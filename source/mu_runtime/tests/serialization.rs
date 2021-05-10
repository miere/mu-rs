use aws_lambda_events::event::alb::AlbTargetGroupRequest;
use std::panic::panic_any;

#[test]
#[ignore]
// This test fails with the 0.4.0 version of aws_lambda_events crate.
fn should_serialize_alb_request_as_expected() {

    let request_bytes = include_str!("./sample_alb_request.json");
    let request = serde_json::from_str::<AlbTargetGroupRequest>(request_bytes);

    match request {
        Err(cause) => panic!("Unexpected: {}", cause),
        Ok(request) => println!("{:?}", request)
    }

}