use std::error::Error;
use std::fmt::Debug;
use std::future::Future;

use serde::{Deserialize, Serialize};

use crate::ErrorHandler;

/// Runs a closure as a AWS Lambda endpoint. This method is basically
/// a syntax sugar for the traditional (non-macro) lambda syntax.
/// ```no_run
///  let handler_fn = lambda::handler_fn(handler);
///  lambda::run(handler_fn).await;
/// ```
pub async fn run_fn<F, Fut, A, B, E>(handler: F)
    where
        F: Fn(A, lambda::Context) -> Fut + Sync,
        Fut: Future<Output = Result<B, E>> + Send,
        A: for<'de> Deserialize<'de>,
        B: Serialize,
        E: Debug + Error
{
    let handler_fn = lambda::handler_fn(handler);
    let result = lambda::run(handler_fn).await;
    if let Err(cause) = result {
        eprintln!("Lambda function finished with error: {:?}", cause)
    }
}

/// Runs a closure as a AWS Lambda endpoint. Unlike the `run_fn` function,
/// this one will expect a valid AWS Application Load Balancer as a contract.
/// ```no_run
/// // Alb contract signature that has been reexported from `aws_lambda_events::event::alb`
/// use lambda_runtime_alb::*;
/// // Convenient trait for
/// // Lambda Context reexported from `lambda::Context`
/// use lambda_runtime_alb::Context;
///
/// lambda_runtime_alb::run_alb_fn(|req: AlbTargetGroupRequest, ctx: Context| -> AlbTargetGroupResponse {
///     todo!()
/// });
/// ```
pub async fn run_alb_fn<F, Fut, A, B, E>(handler: F)
    where
        F: Fn(A, lambda::Context) -> Fut + Sync + Send,
        Fut: Future<Output = Result<B, E>> + Send,
        A: for<'de> Deserialize<'de> + Send,
        B: Serialize,
        E: ErrorHandler<E> + Debug + Error
{
    let borrowed_handler = handler;
    let handler_fn = lambda::handler_fn(|req: A, ctx: lambda::Context| {
        async {
            (borrowed_handler)(req, ctx).await.to_handled_response()
        }
    });
    let result = lambda::run(handler_fn).await;
    if let Err(cause) = result {
        eprintln!("Lambda function finished with error: {:?}", cause)
    }
}