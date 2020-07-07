//! A wrapper over merging the official `lambda_runtime` and the trustworthy `aws_lambda_event`
//! crate. The idea is provide an easy-to-use api for AWS Serverless Developers, especially those
//! comming from the enterprise world who wants to take advantage of the powerful Rust ecosystem.
//! This API has been designed and optimized to receive Lambda requests from the AWS Application
//! Load Balancer.
//!
//! ## Overview
//! Here is the overview of the required imports.
//! ```no_run
//! // Alb contract signature that has been reexported from `aws_lambda_events::event::alb`
//! use lambda_runtime_rest::{AlbTargetGroupRequest, AlbTargetGroupResponse};
//! // Lambda Context reexported from `lambda::Context`
//! use lambda_runtime_rest::Context;
//! // Convenient trait for to convert Serde Serializable structures into ALB responses
//! use lambda_runtime_rest::{ErrorHandler, ResponseHandler};
//! ```
//! Or, for short
//! ```no_run
//! use lambda_runtime_alb::*;
//! ```
//! To handle requests, you can use the following snippet as blueprint.
//! ```no_run
//! use lambda_runtime_alb::*;
//! use std::default::Default;
//!
//! #[tokio::main]
//! async fn main() {
//!     // A component that holds objects responsible to communicate with the DB
//!     let user_component = UserComponent::default();
//!
//!     lambda_runtime_alb::run_alb_fn(|req: AlbTargetGroupRequest, ctx: Context| -> AlbTargetGroupResponse {
//!         let req: CreateUserRequest = req.body.unwrap().parse().unwrap();
//!         async { user_component.create_user( req ).await.to_handled_response() }
//!     });
//! }
//! ```
//! ## Basic Concepts
//!
//! ### Components
//! On the above example was demonstrated how a component can be used to wrap together all the
//! business logic functions that share the same instrincic dependencies (like DB connection).
//! If you have some BDD background, you've been using the concept of components and services
//! widely on your code base. As Rust imposes a few restrictions on how you can define global
//! constants, it might be tricky to have a Lambda function that relies on a shared state -
//! avoiding the cost of the unnecessarily creation of costy resources on a subsequent request.
//!
//! ### API Contract Consistency
//! Having a consistent API contract for a particular microservice is a mandatory requirement
//! nowadays. It increases the chances of a developer to grasp how it works with little effort,
//! making easy to create clients for it.
//!
//! To approach this problem, the trait `ErrorHandler` and `ResponseHandler` has been created.
//! They are responsible to convert errors (`Debug + Error`) and serializable structures (as long
//! as they are compatible with serde deserialization mechanism) into a valid AWS Application Load
//! Balancer response. Usually, as long as you match their predicates, you don't have to manually
//! implement them.
//!

// Re-exports
pub use lambda::Context;
pub use aws_lambda_events::event::alb::{
    AlbTargetGroupRequest,
    AlbTargetGroupRequestContext,
    AlbTargetGroupResponse,
    ElbContext
};

pub use crate::alb::*;
pub use crate::runtime::*;

// Modules
pub mod runtime;
pub mod alb;

