//! A wrapper merging the official `lambda_runtime` and the trustworthy `aws_lambda_event`
//! crate. The idea behind μ-rs is to provide an easy-to-use api for AWS Serverless Developers,
//! leveraging enterprise-grade semantics in the powerful Rust ecosystem.
//!
//! The current state of the official AWS Lambda Runtime for Rust
//! provides a good base for developers to runtime simple lambda functions, but when it comes to
//! maintaining a bigger code base, the development experience is not so pleasant. It is especially
//! true if you need to share state between sub-sequent Lambda requests (e.g. Database Connection
//! Pools). μ-rs takes advantage of the official __lambda_runtime__, providing a consistent API
//! to leverage Lambda development, with especial attention to HTTP Requests.

// Modules
#[cfg(feature = "alb")]
pub mod alb;
pub mod lambda;
