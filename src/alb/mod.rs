//! An AWS Application Load Balancer abstraction to handle Http requests with AWS Lambda.
//! Being designed with correctness and robustness in mind, it covers more complex scenarios
//! by default - like Http Error handling and customizable serialization mechanism.
//!
//! Let's say hello world?
//! ```no_run
//! use mu::{alb, lambda};
//!
//! #[tokio::main]
//! async fn main() -> lambda::RuntimeResult {
//!   alb::listen_events(|req: alb::Request| say_hello()).await
//! }
//!
//! async fn say_hello() -> alb::Response {
//!   alb::response::create_plain_text(200, Some("Hello, mate".to_string()))
//! }
//! ```
//!
//! ## Centralised Serialization Mechanism
//! One of the common questions raised when designing a serverless application is how to have
//! a consistent response serialization that can shared across different lambda functions. This
//! is especially true when developing HTTP endpoints, where you might have different successful
//! response types, but shared error types.
//!
//! By creating your own `mu::alb::Serialize` implementation, as below exemplified, you can
//! globally define how a given object will be sent as a response to the AWS Application Load Balancer.
//!
//! ```no_run
//! use mu::alb;
//! use serde::Serialize;
//!
//! enum MyResponses<T> {
//!     Success(T),
//!     NoContent,
//!     BadRequest
//! }
//!
//! impl<T> alb::Serialize for MyResponses<T>
//!     where T: Serialize {
//!
//!     fn to_alb_response(&self) -> alb::Response {
//!         match self {
//!             MyResponses::Success(msg) => alb::response::create_json_from_obj(200, msg),
//!             MyResponses::NoContent => alb::response::create_plain_text(204, None),
//!             MyResponses::BadRequest => alb::response::create_plain_text(400, None)
//!         }
//!     }
//! }
//! ```
//!
//! ## Custom Request Deserialization
//! It is also possible to replace the __mu::alb::Request__ type by our custom type in the listener
//! function, significantly reducing the size of your Lambda functions, mimicking the RPC request-style.
//!
//! ```no_run
//! use mu::{alb, lambda};
//!
//! struct EmptyPayload {}
//!
//! impl alb::Deserialize<EmptyPayload> for EmptyPayload {
//!     fn from_alb_request(req: alb::Request, ctx: lambda::Context) -> Result<EmptyPayload, lambda::LambdaError> {
//!         match req.body {
//!             None => Ok(EmptyPayload {}),
//!             Some(_) => Err("Unexpected payload".into())
//!         }
//!     }
//! }
//! ```
//! Certainly, implementing __mu::alb::Deserialize__ for several structs might become a burden when
//! the software gets bigger. If the structs derives __serde::Deserialize__, though, the number of
//! lines of code required to create a deserializer can be significantly reduced.
//!
//! ```no_run
//! use mu::{alb, lambda};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct EmptyPayload {}
//!
//! impl alb::RpcRequest for EmptyPayload {}
//!
//! #[tokio::main]
//! async fn main() -> lambda::RuntimeResult {
//!   alb::listen_events(|empty: EmptyPayload|
//!     say_hello()
//!   ).await
//! }
//!
//! async fn say_hello() -> alb::Response {
//!   alb::response::create_plain_text(200, Some("Hello, mate".to_string()))
//! }
//! ```

pub mod deserializer;
pub mod response;
pub mod runtime;
pub mod serializer;

// Stable, long-term API

pub use aws_lambda_events::event::alb::{
    AlbTargetGroupRequest as Request, AlbTargetGroupRequestContext as RequestContext,
    AlbTargetGroupResponse as Response, ElbContext,
};

pub use crate::alb::deserializer::AlbDeserialize as Deserialize;
pub use crate::alb::deserializer::RpcRequest;
pub use crate::alb::runtime::listen_events;
pub use crate::alb::serializer::AlbSerialize as Serialize;
