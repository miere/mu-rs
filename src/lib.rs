//! A wrapper merging the official `lambda_runtime` and the trustworthy `aws_lambda_event`
//! crate. The idea is provide an easy-to-use api for AWS Serverless Developers, especially those
//! coming from the enterprise world who wants to take advantage of the powerful Rust ecosystem.
//! This API has been designed and optimized to receive Lambda requests from the AWS Application
//! Load Balancer.
//!
//! Note: It could be possible to provide AWS API Gateway compatibility layer as well, but this
//! away from its current scope.
//!
//! ## Basic Concepts
//! ### Components
//! On the above example has demonstrated how a component can be used to wrap together all the
//! business logic functions that share the same intrinsic dependencies (like DB connection).
//! If you have some BDD background, you've been using the concept of components and services
//! widely on your code base. As Rust imposes a few restrictions on how you can define global
//! constants, it might be tricky to have a Lambda function that relies on a shared state -
//! avoiding the cost of the unnecessarily creation of expensive resources on a subsequent request.
//!
//! ### API Contract Consistency
//! Having a consistent API contract for a particular microservice is a mandatory requirement
//! nowadays. It increases the chances of a developer to grasp how it works with little effort,
//! making easy to create clients for it.
//!
//! To approach this problem, the trait `ResponseHandler` has been created.
//! It's responsible to convert errors (`Debug + Error`) and (serde) serializable structures
//! into a valid AWS Application Load Balancer response.
//!

// Modules
pub mod alb;
pub mod lambda;
