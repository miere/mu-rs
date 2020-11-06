# μ-rs
A robust, but lightweight, AWS Lambda Runtime for Rust.

## Overview
A wrapper merging the official `lambda_runtime` and the trustworthy `aws_lambda_event`
crate. The idea behind μ-rs is to provide an easy-to-use api for AWS Serverless Developers,
leveraging enterprise-grade semantics in the powerful Rust ecosystem.

The current state of the official AWS Lambda Runtime for Rust
provides a good base for developers to runtime simple lambda functions, but when it comes to
maintaining a bigger code base, the development experience is not so pleasant. It is especially
true if you need to share state between sub-sequent Lambda requests (e.g. Database Connection
Pools). μ-rs takes advantage of the official `lambda_runtime`, providing a consistent API
to leverage Lambda development, with especial attention to HTTP Requests. 

## Usage
> **Disclaimer**:
This crate has been developed on the best effort from its authors. Although quite useful as being
> presented, it still on its early stages of development.

As this crate relies on the most recent version of the `lambda_runtime` it hasn't being published
to crates.io yet, you need to use Git reference to set up a project using μ-rs.

```toml
[dependencies.mu]
git = "https://github.com/miere/mu-rs"
branch = "0.1.0"
```

## Documentation
`TODO`

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
