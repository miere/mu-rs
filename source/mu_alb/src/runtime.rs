use std::future::Future;

use aws_lambda_events::event::alb::{
    AlbTargetGroupRequest, AlbTargetGroupResponse
};

use mu_runtime::{Context, Error};

use crate::deserializer::AlbDeserialize;
use crate::{response, AlbSerialize};

/// Listen to ALB events. Unlike [mu_runtime::listen_events], this method
/// expects you to respect the AWS Application Load Balancer contract by returning
/// the appropriate response (defined by [aws_lambda_events::event::alb::AlbTargetGroupRequest]).
///
/// It's expected the `handler` argument to be an async function, as exemplified below.
///
/// ```no_run
/// use mu_alb::*;
/// use aws_lambda_events::event::alb::{
///     AlbTargetGroupRequest,
///     AlbTargetGroupResponse
/// };
///
/// #[tokio::main]
/// async fn main() -> RuntimeResult {
///   listen_events(|req: AlbTargetGroupRequest| {
///     say_hello()
///   }).await
/// }
///
/// async fn say_hello() -> AlbTargetGroupResponse {
///  response::create_as_plain_text(
///    200, Some("Hello World".to_string()))
/// }
/// ```
pub async fn listen_events<F, Fut, A, B>(handler: F) -> mu_runtime::RuntimeResult
where
    F: Fn(A) -> Fut + Sync + Send,
    Fut: Future<Output = B> + Send,
    A: AlbDeserialize<A> + Send,
    B: AlbSerialize,
{
    mu_runtime::listen_events(
        |req, ctx| handle_rpc_req(&handler, req, ctx)
    ).await
}

/// Handle the RPC request.
#[inline]
async fn handle_rpc_req<F, Fut, A, B>(
    func: &F,
    req: AlbTargetGroupRequest,
    ctx: Context,
) -> Result<AlbTargetGroupResponse, Error>
where
    F: Fn(A) -> Fut + Sync + Send,
    Fut: Future<Output = B> + Send,
    A: AlbDeserialize<A> + Send,
    B: AlbSerialize,
{
    let result: Result<A, Error> = A::from_alb_request(req, ctx);
    Ok(match result {
        Ok(deserialized) => (func)(deserialized).await.to_alb_response(),
        Err(cause) => response::create_as_plain_text(
            400, Some(format!("Bad Request {}", cause))
        ),
    })
}
