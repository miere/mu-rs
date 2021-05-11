# μ-rs
A minimalistic AWS Lambda runtime. It was written based on the official `lambda_runtime` and
was designed to be used along with the trustworthy `aws_lambda_event` crate.

## Why another runtime?
The official AWS Lambda Runtime is awesome, very well crafted and has battle-tested by several developers
in the past few years. It is known, though, that it has a [slow development 
pace](https://github.com/awslabs/aws-lambda-rust-runtime/issues/274) - where PR and tickets
being left months until receive proper attention. The so waited release of the 0.3.0 version also
introduced [extra complexities on its default API](https://github.com/awslabs/aws-lambda-rust-runtime/issues/310),
making it more verbose and less developer friendly.

The main goal of μ-rs is to provide an easy-to-use api for AWS Serverless Developers,
leveraging enterprise-grade semantics in the powerful Rust ecosystem.

> **Disclaimer**:
This crate has been developed on the best effort from its authors. Although quite useful as being
> presented, it still on its early stages of development.

## Acknowledgement
Although initially this library used to depend on the (official) `lambda_runtime` crate, ever
since the 0.2.0 it's not being the case anymore. The current codebase was deeply inspired on
the official one though, having its code base deeply simplified. It is still possible to
see a resemblances between these two crates, though. In the case this library proves useful,
it can be back-ported to the upstream repository.

## Usage
Set up the dependency.
```toml
[dependencies]
mu_runtime="0.2.0"
```
Start listen to events on your Lambda functions.
```rust
// Sample lambda function that listen for SQS events.
use aws_lambda_events::event::sqs::SqsEvent;
use mu_runtime;

#[tokio::main]
async fn main() -> mu_runtime::RuntimeResult {
  mu_runtime::listen_events(|sqs_events, ctx| {
    handle_sqs_messages(sqs_events)
  }).await
}

async fn handle_sqs_messages(sqs_events: SqsEvent) -> Result<(), mu_runtime::Error> {
  println!("Received {} events", sqs_events.records.len());
  Ok(())
}
```

## Documentation
- [Crate documentation](https://docs.rs/mu_runtime/)

## Reporting Bugs/Feature Requests
We welcome you to use the GitHub issue tracker to report bugs or suggest features.

When filing an issue, please check existing open, or recently closed, issues to make sure somebody else hasn't already
reported the issue. Please try to include as much information as you can. Details like these are incredibly useful:

* A reproducible test case or series of steps
* The version of our code being used
* Any modifications you've made relevant to the bug
* Anything unusual about your environment or deployment


## Contributing via Pull Requests
Contributions via pull requests are much appreciated. Before sending us a pull request, please ensure that:

1. You are working against the latest source on the *master* branch.
2. You check existing open, and recently merged, pull requests to make sure someone else hasn't addressed the problem already.
3. You open an issue to discuss any significant work - we would hate for your time to be wasted.

To send us a pull request, please:

1. Fork the repository.
2. Modify the source; please focus on the specific change you are contributing. If you also reformat all the code, it will be hard for us to focus on your change.
3. Ensure local tests pass.
4. Commit to your fork using clear commit messages.
5. Send us a pull request, answering any default questions in the pull request interface.
6. Pay attention to any automated CI failures reported in the pull request, and stay involved in the conversation.

GitHub provides additional document on [forking a repository](https://help.github.com/articles/fork-a-repo/) and
[creating a pull request](https://help.github.com/articles/creating-a-pull-request/).

## Finding contributions to work on
Looking at the existing issues is a great way to find something to contribute on. As our projects, by default, use the default GitHub issue labels ((enhancement/bug/duplicate/help wanted/invalid/question/wontfix), looking at any 'help wanted' issues is a great place to start.

## License
This is release under the Apache License 2 terms.
