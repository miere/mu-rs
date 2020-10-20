//! A decorator module that provides consistent API for further Lambda development.

pub use lambda::{Context, Handler};

use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter, Result as FormatterResult};
use std::error::Error as StdError;
use std::future::Future;

pub type Succeeded = ();
pub type RuntimeResult = Result<Succeeded, LambdaError>;

/// A struct to hold errors resulting from the Lambda initialization.
/// As the original lambda codebase made the type Error private to its crate
/// it was required to create a wrapper to hold the received error message
/// received from the original Lambda. Otherwise, we might have to create
/// our own Error type (that matches the original contract).
#[derive(Debug)]
pub struct LambdaError(String);

impl StdError for LambdaError {}

impl Display for LambdaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatterResult {
        write!(f, "{}", self.0)
    }
}

/// Initializes a function as a handler for AWS Lambda requests. It is expected
/// that the handler function respect the contract `async fn(A,Context) -> B`, as
/// long as both A and B are Serde serializable structures.
pub async fn initialize<F, Fut, A, B, E>(handler: F) -> RuntimeResult
    where
        F: Fn(A, Context) -> Fut + Sync + Send,
        Fut: Future<Output = Result<B, E>> + Send,
        A: for<'de> Deserialize<'de> + Send,
        B: Serialize,
        E: Debug + StdError
{
    let wrapped = lambda::handler_fn(handler);
    let result = lambda::run(wrapped).await;
    match result {
        Ok(empty) => Ok(empty),
        Err(cause) => Err(LambdaError(
            format!("Lambda unexpectedly finished: {:?}", cause)
        ))
    }
}
