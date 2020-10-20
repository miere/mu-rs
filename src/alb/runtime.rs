use std::future::Future;

use aws_lambda_events::event::alb::{AlbTargetGroupRequest, AlbTargetGroupResponse};

use crate::alb;
use crate::lambda;
use crate::lambda::LambdaError;
use crate::alb::Serialize;
use crate::alb::Deserialize;

pub async fn initialize<F, Fut, A, B, C>(handler: F) -> lambda::RuntimeResult
    where
        F: Fn(A) -> Fut + Sync + Send,
        Fut: Future<Output = B> + Send,
        A: Deserialize<A> + Send,
        B: Serialize
{
    lambda::initialize(|req: AlbTargetGroupRequest, ctx: lambda::Context| {
        handle_rpc_req(&handler, req, ctx)
    }).await
}

/// Handle the RPC request.
async fn handle_rpc_req<F, Fut, A, B>(func: &F, req: AlbTargetGroupRequest, ctx: lambda::Context)
    -> Result<AlbTargetGroupResponse, LambdaError>
    where
        F: Fn(A) -> Fut + Sync + Send,
        Fut: Future<Output = B> + Send,
        A: Deserialize<A> + Send,
        B: Serialize
{
    let result: Result<A, LambdaError> = A::from_alb_request(req, ctx);
    Ok(match result {
        Ok(deserialized) => (func)(deserialized).await.to_alb_response(),
        Err(cause) => alb::response::create_plain_text(400, Some(
            format!("Bad Request {}", cause)
        ))
    })
}
