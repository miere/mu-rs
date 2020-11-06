use std::future::Future;

use crate::alb;
use crate::alb::*;
use crate::lambda;
use crate::lambda::LambdaError;

/// Listen to ALB events. Unlike __mu::lambda::listen_events__, this method
/// expects you to respect the AWS Application Load Balancer contract by returning
/// the appropriate response (defined by __mu::alb::Response__).
///
/// It's expected the `handler` argument to be an async function, as exemplified below.
///
/// ```no_run
/// use mu::{alb, lambda};
///
/// #[tokio::main]
/// async fn main() -> lambda::RuntimeResult {
///   alb::listen_events(|req: alb::Request| {
///     say_hello()
///   }).await
/// }
///
/// async fn say_hello() -> alb::Response {
///  alb::response::create_plain_text(
///    200, Some("Hello World".to_string()))
/// }
/// ```
pub async fn listen_events<F, Fut, A, B>(handler: F) -> lambda::RuntimeResult
where
    F: Fn(A) -> Fut + Sync + Send,
    Fut: Future<Output = B> + Send,
    A: Deserialize<A> + Send,
    B: Serialize,
{
    lambda::listen_events(|req: Request, ctx: lambda::Context| handle_rpc_req(&handler, req, ctx))
        .await
}

/// Handle the RPC request.
#[inline]
async fn handle_rpc_req<F, Fut, A, B>(
    func: &F,
    req: Request,
    ctx: lambda::Context,
) -> Result<Response, LambdaError>
where
    F: Fn(A) -> Fut + Sync + Send,
    Fut: Future<Output = B> + Send,
    A: Deserialize<A> + Send,
    B: Serialize,
{
    let result: Result<A, LambdaError> = A::from_alb_request(req, ctx);
    Ok(match result {
        Ok(deserialized) => (func)(deserialized).await.to_alb_response(),
        Err(cause) => alb::response::create_plain_text(400, Some(format!("Bad Request {}", cause))),
    })
}
