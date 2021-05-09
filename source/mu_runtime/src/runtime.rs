use std::error::Error as StdError;
use std::future::Future;
use std::result::Result as StdResult;

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::lambda_api::{LambdaApiClient, PublishErrorRequest};
use crate::model::Context;

/// Represents the result of the Lambda runtime execution.
pub type RuntimeResult = StdResult<(), Error>;

/// Listen to AWS Lambda events and delegates the received payload to
/// the [handler] function.
///
/// It is expected that the [handler] functions follows the contract
/// `Fn(A) -> Future<Output=Result<B, E>>` where both [A] and [B] types
/// are expected to be Serde deserializable and serializable, respectively.
/// The [E] type can be any valid error type.
///
/// ```no_run
/// use aws_lambda_events::event::sqs::SqsEvent;
///
/// #[tokio::main]
/// async fn main() -> mu_runtime::RuntimeResult {
///   mu_runtime::listen_events(|sqs_events, ctx| {
///     handle_sqs_messages(sqs_events)
///   }).await
/// }
///
/// async fn handle_sqs_messages(sqs_events: SqsEvent) -> Result<(), mu_runtime::Error> {
///   println!("Received {} events", sqs_events.records.len());
///   Ok(())
/// }
/// ```
pub async fn listen_events<F, Fut, A, B, E>(handler: F) -> RuntimeResult
    where F: Fn(A, Context) -> Fut + Sync + Send,
          Fut: Future<Output=StdResult<B, E>> + Send,
          A: for<'de> Deserialize<'de> + Send,
          B: Serialize,
          E: StdError
{
    let lambda_api = LambdaApiClient::default();
    listen_events_with(lambda_api, handler).await
}

/// Listen to AWS Lambda events and delegates the received payload to
/// the [handler] function. This method allows one to define the LambdaApi
/// instance that will be used in the Lambda-consumption mainloop. This
/// might be desirable for local testing.
#[inline]
pub async fn listen_events_with<F, Fut, A, B, E>(lambda_api: LambdaApiClient, handler: F) -> RuntimeResult
    where F: Fn(A, Context) -> Fut + Sync + Send,
          Fut: Future<Output=StdResult<B, E>> + Send,
          A: for<'de> Deserialize<'de> + Send,
          B: Serialize,
          E: StdError
{
    loop {
        try_invoke_lambda_handler(&lambda_api, &handler).await?;
        // allows one to perform single request tests during the Integration Tests.
        if cfg!(test) {
            return Ok(())
        }
    }
}

/// Performs the actual Lambda Invocation lifecycle.
#[inline]
async fn try_invoke_lambda_handler<F, Fut, A, B, E>(lambda_api: &LambdaApiClient, handler: &F) -> RuntimeResult
    where F: Fn(A, Context) -> Fut + Sync + Send,
          Fut: Future<Output=StdResult<B, E>> + Send,
          A: for<'de> Deserialize<'de> + Send,
          B: Serialize,
          E: StdError
{
    let (bytes, context) = lambda_api.fetch_next_message().await?;
    let request_id = context.request_id.clone();
    let body = serde_json::from_slice(&bytes)?;
    let result = (handler)(body, context).await;

    match result {
        Ok(payload) => lambda_api.publish_response(request_id, payload).await?,
        Err(error) => {
            let payload = PublishErrorRequest {
                error_type: type_name_of_val(&error).to_string(),
                error_message: format!("{}", error)
            };
            lambda_api.publish_error(request_id, payload).await?
        }
    }

    Ok(())
}

fn type_name_of_val<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

// Integration test has been moved to this file because `if cfg(test)` doesn't
// work in integration tests.
//
// See: https://users.rust-lang.org/t/cfg-test-does-not-work-with-integration-tests/36630/7
#[cfg(test)]
mod integration_tests {
    use aws_lambda_events::event::alb::AlbTargetGroupRequest;
    use httpmock::{MockRef, MockServer};
    use rusoto_core::Region;
    use rusoto_dynamodb::DynamoDbClient;

    use crate::Error;
    use crate::lambda_api::LambdaApiClient;
    use crate::listen_events_with;
    use crate::model::Config;

    /// A simulates a complex repository that relies on DynamoDB to
    /// store data. This was designed to mimic real-world scenario
    /// where users has to interact with complex structures that holds
    /// state between sub-sequent lambda requests.
    struct DynamoDbRepository {
        #[allow(dead_code)]
        dynamo_db: DynamoDbClient,
    }

    impl DynamoDbRepository {

        fn create() -> Self {
            let region = Region::default();
            Self { dynamo_db: DynamoDbClient::new(region) }
        }

        async fn a_method_that_will_succeed(&self) -> Result<i32, Error> {
            Ok(42)
        }

        async fn a_method_that_will_fail(&self) -> Result<(), Error> {
            Err("Not implemented".into())
        }
    }

    #[tokio::test]
    async fn should_handle_successful_requests()
    {
        let mock_server = MockServer::start();
        let (next, success, _error) = mock_lambda_runtime_endpoints(&mock_server);

        let lambda_api = create_lambda_api_for_testing(mock_server.port());
        let client = DynamoDbRepository::create();
        let result = listen_events_with(lambda_api, |_req: AlbTargetGroupRequest, _ctx| {
            client.a_method_that_will_succeed()
        }).await;

        if let Err(cause) = result {
            panic!("Unexpected: {}", cause);
        }

        next.assert();
        success.assert();
    }

    #[tokio::test]
    async fn should_handle_failure_requests()
    {
        let mock_server = MockServer::start();
        let (next, _success, error) = mock_lambda_runtime_endpoints(&mock_server);

        let lambda_api = create_lambda_api_for_testing(mock_server.port());
        let client = DynamoDbRepository::create();
        let result = listen_events_with(lambda_api, |_req: AlbTargetGroupRequest, _ctx| {
            client.a_method_that_will_fail()
        }).await;

        if let Err(cause) = result {
            panic!("Unexpected: {}", cause);
        }

        next.assert();
        error.assert();
    }

    fn create_lambda_api_for_testing(port: u16) -> LambdaApiClient {
        LambdaApiClient::create(Config {
            endpoint: format!("127.0.0.1:{}", port),
            ..Default::default()
        })
    }

    fn mock_lambda_runtime_endpoints(server: &MockServer) -> (MockRef, MockRef, MockRef) {
        let next_endpoint = server.mock(|when, then| {
            when.path("/2018-06-01/runtime/invocation/next");

            let alb_request = include_str!("../tests/sample_alb_request.json");
            then.status(200)
                .header("lambda-runtime-aws-request-id", "0000-0001")
                .header("lambda-runtime-deadline-ms", "1000")
                .header("lambda-runtime-invoked-function-arn", "arn::something")
                .header("lambda-runtime-trace-id", "0001-0001")
                .body(alb_request);
        });

        let success_endpoint = server.mock(|when, then| {
            when.path("/2018-06-01/runtime/invocation/0000-0001/response")
                .body("42")
                .method("POST");

            then.status(200);
        });

        let error_endpoint = server.mock(|when, then| {
            when.path("/2018-06-01/runtime/invocation/0000-0001/error")
                .body(r#"{"errorType":"mu_runtime::error::Error","errorMessage":"Not implemented"}"#)
                .method("POST");

            then.status(200);
        });

        (next_endpoint, success_endpoint, error_endpoint)
    }
}
