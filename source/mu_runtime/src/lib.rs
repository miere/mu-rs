//! An alternative AWS Lambda runtime for Rust. It was written based on the official
//! `lambda_runtime` and was designed to be used along with the trustworthy
//! `aws_lambda_event` crate.
//!
//! The idea behind this crate is to provide an easy-to-use api for AWS Serverless Developers,
//! leveraging enterprise-grade semantics in the powerful Rust ecosystem.

pub use runtime::*;
pub use model::Context;
pub use error::Error;

// Modules
pub mod runtime;
pub mod model;
pub mod lambda_api;
pub mod error;

