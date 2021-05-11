use std::env;

use hyper::{Body, HeaderMap, Request};
use hyper::body::Bytes;
use hyper::client::{Client, HttpConnector};
use serde::{Deserialize, Serialize};

use crate::error::Error as LambdaApiError;
use crate::model::{Config, Context};

/// The Lambda Api Client. Abstracts the communication with the internal
/// Lambda Runtime rest API, as documented here:
/// https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
pub struct LambdaApiClient {
    client: Client<HttpConnector, Body>,
    config: Config
}

impl Default for LambdaApiClient {
    fn default() -> Self {
        let config = Config {
            endpoint: env::var("AWS_LAMBDA_RUNTIME_API").unwrap(),
            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME").unwrap(),
            memory: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").unwrap()
                .parse::<i32>().unwrap(),
            version: env::var("AWS_LAMBDA_FUNCTION_VERSION").unwrap(),
            log_stream: env::var("AWS_LAMBDA_LOG_STREAM_NAME").unwrap(),
            log_group: env::var("AWS_LAMBDA_LOG_GROUP_NAME").unwrap(),
        };

        LambdaApiClient {
            client: Client::new(),
            config
        }
    }
}

impl LambdaApiClient {

    /// Creates a new instance of this client. Make sure to pass
    /// a valid [Config] instance, as it holds sensitive attributes that
    /// break the communication with the intended endpoint in case
    /// of misconfiguration.
    pub fn create(config: Config) -> Self {
        LambdaApiClient {
            client: Client::new(),
            config
        }
    }

    /// Fetches the next message to be processed.
    pub async fn fetch_next_message(&self) -> Result<(Bytes, Context), LambdaApiError> {
        let uri = format!("http://{}/2018-06-01/runtime/invocation/next", &self.config.endpoint);
        let uri = uri.parse()?;
        let resp = self.client.get(uri).await?;
        let (parts, body) = resp.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        let context = self.create_execution_context_from(parts.headers);

        if !parts.status.is_success() {
            let error_msg = String::from_utf8(body.to_vec())?;
            return Err(LambdaApiError::from(error_msg))
        }

        Ok((body, context))
    }

    fn create_execution_context_from(&self, headers: HeaderMap) -> Context {
        Context {
            request_id: headers["lambda-runtime-aws-request-id"]
                .to_str()
                .expect("Missing Request ID")
                .to_owned(),
            deadline: headers["lambda-runtime-deadline-ms"]
                .to_str()
                .expect("Missing deadline")
                .parse()
                .expect("Missing deadline"),
            invoked_function_arn: headers["lambda-runtime-invoked-function-arn"]
                .to_str()
                .expect("Missing arn; this is a bug")
                .to_owned(),
            xray_trace_id: headers["lambda-runtime-trace-id"]
                .to_str()
                .expect("Invalid XRayTraceID sent by Lambda; this is a bug")
                .to_owned(),
            client_context: headers.get("lambda-runtime-client-context")
                .map(|h| h.to_str().expect("Invalid ClientContext sent by lambda"))
                .map(|s| serde_json::from_str(s).expect("Invalid ClientContext sent by lambda")),
            identity: headers.get("lambda-runtime-cognito-identity")
                .map(|h| h.to_str().expect("Invalid CognitoIdentity sent by lambda"))
                .map(|s| serde_json::from_str(s).expect("Invalid CognitoIdentity sent by lambda")),
            env_config: self.config.clone(),
        }
    }

    /// Publish a response in case of successful execution.
    pub async fn publish_response<T>(&self, request_id: String, payload: T) -> Result<(), LambdaApiError>
        where T: Serialize
    {
        self.post_message(request_id, "response", payload).await
    }

    /// Publish an error response.
    pub async fn publish_error(&self, request_id: String, payload: PublishErrorRequest) -> Result<(), LambdaApiError>
    {
        self.post_message(request_id, "error", payload).await
    }

    async fn post_message<T>(&self, request_id: String, path: &str, payload: T) -> Result<(), LambdaApiError>
        where T: Serialize
    {
        let payload = serde_json::to_string(&payload)?;

        let uri = format!(
            "http://{}/2018-06-01/runtime/invocation/{}/{}",
            &self.config.endpoint, request_id, path);

        let req = Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(payload))?;

        let resp = self.client.request(req).await?;
        let (parts, body) = resp.into_parts();

        if !parts.status.is_success() {
            let body = hyper::body::to_bytes(body).await?;
            let error_msg = String::from_utf8(body.to_vec())?;
            return Err(LambdaApiError::from(error_msg))
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishErrorRequest {
    pub error_type: String,
    pub error_message: String,
}

#[cfg(test)]
mod tests {

    use crate::lambda_api::LambdaApiClient;
    use crate::model::Config;
    use httpmock::MockServer;
    use maplit::hashmap;

    mod fetching_next_message {

        use super::*;
        use crate::model::{ClientContext, ClientApplication, CognitoIdentity};

        // taken from the C# SDK
        // Check this commit for details: https://git.io/J3FGH
        const COGNITO_CLIENT_CONTEXT: &str = include_str!("../tests/sample_cognito_context.json");

        // taken from the C# SDK
        // Check this commit for details: https://git.io/J3FZ7
        const COGNITO_IDENTITY: &str = include_str!("../tests/sample_cognito_identity.json");

        #[tokio::test]
        async fn should_handle_failures() {
            let unreachable_host = "localhost:4312";

            let api = LambdaApiClient::create(Config {
                endpoint: unreachable_host.to_string(),
                ..Config::default()
            });

            let result = api.fetch_next_message().await;
            match result {
                Ok(_) => panic!("Should not succeed in case of failures"),
                Err(cause) => {
                    let error_msg = format!("{}", cause);
                    assert!(error_msg.starts_with("error trying to connect: tcp connect error: Connection refused"))
                }
            }
        }

        #[tokio::test]
        async fn should_handle_successful_requests() {
            let mock_server = MockServer::start();

            let next_endpoint = mock_server.mock(|when, then| {
                when.path("/2018-06-01/runtime/invocation/next");

                let lambda_request_payload = r#"{ "body": "hello" }"#;
                then.status(200)
                    .header("lambda-runtime-aws-request-id", "0000-0001")
                    .header("lambda-runtime-deadline-ms", "1000")
                    .header("lambda-runtime-invoked-function-arn", "arn::something")
                    .header("lambda-runtime-trace-id", "0001-0001")
                    .body(lambda_request_payload);
            });

            let api = LambdaApiClient::create(Config {
                endpoint: format!("localhost:{}", mock_server.port()),
                ..Config::default()
            });

            let result = api.fetch_next_message().await;
            match result {
                Err(cause) => panic!("Unexpected: {}", cause),
                Ok((bytes, _ctx)) => {
                    let body = String::from_utf8(bytes.to_vec()).unwrap();
                    let expected = r#"{ "body": "hello" }"#.to_string();
                    assert_eq!(expected, body);
                    next_endpoint.assert();
                }
            }
        }

        #[tokio::test]
        async fn should_serialize_client_context() {
            let mock_server = MockServer::start();

            mock_server.mock(|when, then| {
                when.path("/2018-06-01/runtime/invocation/next");

                let lambda_request_payload = r#"{ "body": "hello" }"#;
                then.status(200)
                    .header("lambda-runtime-aws-request-id", "0000-0001")
                    .header("lambda-runtime-deadline-ms", "1000")
                    .header("lambda-runtime-invoked-function-arn", "arn::something")
                    .header("lambda-runtime-trace-id", "0001-0001")
                    .header("lambda-runtime-client-context", &COGNITO_CLIENT_CONTEXT
                        .replace("\r", "").replace("\n", ""))
                    .body(lambda_request_payload);
            });

            let api = LambdaApiClient::create(Config {
                endpoint: format!("localhost:{}", mock_server.port()),
                ..Config::default()
            });
            let result = api.fetch_next_message().await;

            match result {
                Err(cause) => panic!("Unexpected: {}", cause),
                Ok((_bytes, ctx)) => {
                    let client_context = ClientContext {
                        client: ClientApplication {
                            installation_id: "InstallationId1".to_string(),
                            app_title: "AppTitle1".to_string(),
                            app_version_name: "AppVersionName1".to_string(),
                            app_version_code: "AppVersionCode1".to_string(),
                            app_package_name: "AppPackageName1".to_string()
                        },
                        custom: hashmap!(
                            "CustomKey1".to_string() => "CustomValue1".to_string(),
                            "CustomKey2".to_string() => "CustomValue2".to_string()
                        ),
                        environment: hashmap!(
                            "EnvironmentKey1".to_string() => "EnvironmentValue1".to_string(),
                            "EnvironmentKey2".to_string() => "EnvironmentValue2".to_string()
                        )
                    };

                    let expected = Some(client_context);
                    assert_eq!(expected, ctx.client_context);
                }
            }
        }

        #[tokio::test]
        async fn should_serialize_cognito_identity() {
            let mock_server = MockServer::start();

            mock_server.mock(|when, then| {
                when.path("/2018-06-01/runtime/invocation/next");

                let lambda_request_payload = r#"{ "body": "hello" }"#;
                then.status(200)
                    .header("lambda-runtime-aws-request-id", "0000-0001")
                    .header("lambda-runtime-deadline-ms", "1000")
                    .header("lambda-runtime-invoked-function-arn", "arn::something")
                    .header("lambda-runtime-trace-id", "0001-0001")
                    .header("lambda-runtime-Cognito-Identity", &COGNITO_IDENTITY
                        .replace("\r", "").replace("\n", ""))
                    .body(lambda_request_payload);
            });

            let api = LambdaApiClient::create(Config {
                endpoint: format!("localhost:{}", mock_server.port()),
                ..Config::default()
            });
            let result = api.fetch_next_message().await;

            match result {
                Err(cause) => panic!("Unexpected: {}", cause),
                Ok((_bytes, ctx)) => {
                    let cognito_identity = CognitoIdentity {
                        identity_id: "Id1".to_string(),
                        identity_pool_id: "Pool1".to_string()
                    };

                    let expected = Some(cognito_identity);
                    assert_eq!(expected, ctx.identity);
                }
            }
        }
    }

    mod publish_successful_response {

        use super::*;

        #[tokio::test]
        async fn should_handle_failures() {
            let unreachable_host = "localhost:4312";

            let api = LambdaApiClient::create(Config {
                endpoint: unreachable_host.to_string(),
                ..Config::default()
            });

            let result = api.publish_response("0000-0001".to_string(), "42".to_string()).await;
            match result {
                Ok(_) => panic!("Should not succeed in case of failures"),
                Err(cause) => {
                    let error_msg = format!("{}", cause);
                    assert!(error_msg.starts_with("error trying to connect: tcp connect error: Connection refused"))
                }
            }
        }

        #[tokio::test]
        async fn should_be_able_to_publish_response() {
            let mock_server = MockServer::start();

            let success_endpoint = mock_server.mock(|when, then| {
                when.path("/2018-06-01/runtime/invocation/0000-0001/response")
                    .body("\"42\"")
                    .method("POST");

                then.status(200);
            });

            let api = LambdaApiClient::create(Config {
                endpoint: format!("localhost:{}", mock_server.port()),
                ..Config::default()
            });

            let result = api.publish_response("0000-0001".to_string(), "42".to_string()).await;
            if let Err(cause) = result {
                panic!("Returned unsuccessful result: {}", cause)
            }

            success_endpoint.assert();
        }
    }

    mod publish_error_response {
        use super::*;
        use crate::lambda_api::PublishErrorRequest;

        #[tokio::test]
        async fn should_handle_failures() {
            let unreachable_host = "localhost:4312";

            let api = LambdaApiClient::create(Config {
                endpoint: unreachable_host.to_string(),
                ..Config::default()
            });

            let result = api.publish_error(
                "0000-0001".to_string(),
                PublishErrorRequest {
                    error_type: "CompileError".to_string(),
                    error_message: "Not implemented".to_string()
                }).await;

            match result {
                Ok(_) => panic!("Should not succeed in case of failures"),
                Err(cause) => {
                    let error_msg = format!("{}", cause);
                    assert!(error_msg.starts_with("error trying to connect: tcp connect error: Connection refused"))
                }
            }
        }

        #[tokio::test]
        async fn should_publish_error_messages() {
            let mock_server = MockServer::start();

            let error_endpoint = mock_server.mock(|when, then| {
                when.path("/2018-06-01/runtime/invocation/0000-0001/error")
                    .body(r#"{"errorType":"CompileError","errorMessage":"Not implemented"}"#)
                    .method("POST");

                then.status(200);
            });

            let api = LambdaApiClient::create(Config {
                endpoint: format!("localhost:{}", mock_server.port()),
                ..Config::default()
            });

            let result = api.publish_error(
                "0000-0001".to_string(),
                PublishErrorRequest {
                    error_type: "CompileError".to_string(),
                    error_message: "Not implemented".to_string()
                }).await;

            if let Err(cause) = result {
                panic!("Returned unsuccessful result: {}", cause)
            }

            error_endpoint.assert();
        }
    }
}