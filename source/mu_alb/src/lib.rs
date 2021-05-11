//! An abstraction to handle Http requests coming from the AWS Application Load Balancer.
//! Being designed with correctness and robustness in mind, it covers more complex scenarios
//! by default - like Http Error handling and customizable serialization mechanism.
//!
//! Let's say hello world?
//! ```no_run
//! use aws_lambda_events::event::alb::{
//!     AlbTargetGroupRequest, AlbTargetGroupResponse
//! };
//! use mu_alb::*;
//!
//! #[tokio::main]
//! async fn main() -> RuntimeResult {
//!   listen_events(|req: AlbTargetGroupRequest| say_hello()).await
//! }
//!
//! async fn say_hello() -> AlbTargetGroupResponse {
//!   response::create_as_plain_text(200, Some("Hello, mate".to_string()))
//! }
//! ```
//!
//! ## Centralised Serialization Mechanism
//! One of the common converns raised when designing a serverless application is how to have
//! a consistent response serialization that can shared across different lambda functions. This
//! is especially true when developing HTTP endpoints where you might have different successful
//! response types, but shared error types.
//!
//! By creating your own [crate::AlbSerialize] implementation, as below exemplified, you can
//! globally define how a given object will be sent as a response to the AWS Application Load Balancer.
//!
//! ```no_run
//! use aws_lambda_events::event::alb::AlbTargetGroupResponse;
//! use mu_alb;
//! use serde::Serialize;
//!
//! enum MyResponses<T> {
//!     Success(T),
//!     NoContent,
//!     BadRequest
//! }
//!
//! impl<T> mu_alb::AlbSerialize for MyResponses<T>
//!     where T: Serialize {
//!
//!     fn to_alb_response(&self) -> AlbTargetGroupResponse {
//!         match self {
//!             MyResponses::Success(msg) => mu_alb::response::create_json_from_obj(200, msg),
//!             MyResponses::NoContent => mu_alb::response::create_as_plain_text(204, None),
//!             MyResponses::BadRequest => mu_alb::response::create_as_plain_text(400, None)
//!         }
//!     }
//! }
//! ```
//!
//! ## Custom Request Deserialization
//! It is also possible to replace the [aws_lambda_events::event::alb::AlbTargetGroupRequest] type
//! with your custom type in the listener function, it might be convenient when desiging RPC
//! request-style APIs.
//!
//! ```no_run
//! use aws_lambda_events::event::alb::AlbTargetGroupRequest;
//! use mu_alb::*;
//!
//! struct EmptyPayload {}
//!
//! impl AlbDeserialize<EmptyPayload> for EmptyPayload {
//!     fn from_alb_request(req: AlbTargetGroupRequest, ctx: Context) -> Result<EmptyPayload, Error> {
//!         match req.body {
//!             None => Ok(EmptyPayload {}),
//!             Some(_) => Err("Unexpected payload".into())
//!         }
//!     }
//! }
//! ```
//! On a much more complex event, one might need to deserialize the received Payload and transform
//! that into the desired entity. If you're using Serde, though, things might be further simplied.
//!
//! ```no_run
//! use mu_alb::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct EmptyPayload {}
//!
//! // TODO: turn this into a derive macro
//! impl RpcRequest for EmptyPayload {}
//! ```
//!
//! ## Features
//! - `multi_header`: enables support to multi-value headers and query strings.
//!    For more on that check the official [AWS documentation about this
//!    topic](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html#multi-value-headers).
//!

// Internal modules are public, so people can use it whenever it makes sense.
pub mod deserializer;
pub mod response;
pub mod runtime;
pub mod serializer;

// Stable, long-term API
pub use crate::{
    deserializer::AlbDeserialize,
    deserializer::RpcRequest,
    runtime::listen_events,
    serializer::AlbSerialize,
};

// Re-exporting a few entries from mu_runtime, for convenience.
pub use mu_runtime::{
    Error,
    RuntimeResult,
    Context,
};