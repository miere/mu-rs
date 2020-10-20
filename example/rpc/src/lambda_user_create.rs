mod rpc_api;

use lambda_runtime_ext::alb;
use lambda_runtime_ext::lambda;
use crate::rpc_api::UserRpcController;

#[tokio::main]
async fn main() -> lambda::RuntimeResult {
    // instantiate your app/component
    let controller = UserRpcController {};
    // wrap it into a Fn
    alb::initialize_rpc_fn(|req| controller.create(req)).await
}
