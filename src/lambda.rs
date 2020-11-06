//! A decorator module that provides consistent API for further Lambda development.

pub use lambda::{Context, Handler};

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FormatterResult};
use std::future::Future;

/// A custom result type that assumes __LambdaError__ as error type.
pub type LambdaResult<T> = Result<T, LambdaError>;

/// The default lambda runtime result type. It will be returned by
/// all listener functions, and can be used as return type for your
/// __main__ function.
pub type RuntimeResult = Result<(), LambdaError>;

/// A struct to hold errors resulting from the Lambda initialization.
/// As the original lambda codebase made the type Error private to its crate
/// it was required to create a wrapper to hold the received error message
/// received from the original Lambda. Otherwise, we might have to create
/// our own Error type (that matches the original contract).
#[derive(Debug)]
pub struct LambdaError(String);

impl LambdaError {
    pub fn new(msg: &str) -> LambdaError {
        LambdaError(msg.to_string())
    }
}

impl Into<LambdaError> for &str {
    fn into(self) -> LambdaError {
        LambdaError(self.to_string())
    }
}

impl Into<LambdaError> for String {
    fn into(self) -> LambdaError {
        LambdaError(self)
    }
}

impl StdError for LambdaError {}

impl Display for LambdaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatterResult {
        write!(f, "{}", self.0)
    }
}

/// Listen for AWS Lambda requests of a given __A__ type. This method doesn't
/// enforce any contract aside of the fact it expects a __std::result::Result<B, Error>__
/// as a return value. It also expects the __handler__ function to be __async__.
///
/// It's important to notice that both __A__ and __B__ types are expected to be
/// Serde deserializable and serializable, respectively.
///
/// ```no_run
/// use aws_lambda_events::event::sqs::SqsEvent;
/// use mu::lambda;
///
/// #[tokio::main]
/// async fn main() -> lambda::RuntimeResult {
///   lambda::listen_events(|sqs_events, ctx| {
///     handle_sqs_messages(sqs_events)
///   }).await
/// }
///
/// async fn handle_sqs_messages(sqs_events: SqsEvent) -> lambda::LambdaResult<()> {
///   println!("Received {} events", sqs_events.records.len());
///   Ok(())
/// }
/// ```
pub async fn listen_events<F, Fut, A, B, E>(handler: F) -> RuntimeResult
where
    F: Fn(A, Context) -> Fut + Sync + Send,
    Fut: Future<Output = Result<B, E>> + Send,
    A: for<'de> Deserialize<'de> + Send,
    B: Serialize,
    E: Debug + StdError,
{
    let wrapped = lambda::handler_fn(handler);
    let result = lambda::run(wrapped).await;
    match result {
        Ok(_) => Ok(()),
        Err(cause) => Err(LambdaError(format!(
            "Lambda unexpectedly finished: {:?}",
            cause
        ))),
    }
}
