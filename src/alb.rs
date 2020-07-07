use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

use aws_lambda_events::event::alb::AlbTargetGroupResponse;
use serde::Serialize;
use serde_json;

/// A response handler that allows ordinary structures and enums
/// to be converted into an ALB valid response.
pub trait ResponseHandler {
    fn to_alb_response(&self) -> AlbTargetGroupResponse;
}

/// Allow errors to be handled and translated into a valid ALB response
pub trait ErrorHandler<E>
    where E: Debug + Error
{
    fn to_handled_response(&self) -> Result<AlbTargetGroupResponse, E>;
}

/// Ensure `Result`s will be translated into valid ALB responses.
impl <A, E> ErrorHandler<E> for Result<A, E>
    where A: Serialize,
          E: ErrorHandler<E> + Debug + Error
{
    fn to_handled_response(&self) -> Result<AlbTargetGroupResponse, E> {
        match &self {
            Ok(value) => Ok(value.to_alb_response()),
            Err(cause) => cause.to_handled_response()
        }
    }
}

/// Converts structs and enums marked with `Serialize` into a valid ALB response.
impl<T> ResponseHandler for T
    where T: Serialize
{
    fn to_alb_response(&self) -> AlbTargetGroupResponse {
        match serde_json::to_string(self) {
            Ok(serialized) => response::create_json(
                200, Some(serialized),
            ),
            Err(cause) => response::create_json(
                500, Some(format!("{}", cause)),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct User {
        name: String
    }

    #[test]
    fn should_convert_serializable_into_response() {
        let serializable = User{ name: String::from("John") };

        let response = serializable.to_alb_response();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body.unwrap(), "{\"name\":\"John\"}".to_string());

        let header = response.headers.get(CONTENT_TYPE);
        assert_ne!(None, header);
        assert_eq!(Some(&CONTENT_TYPE_JSON.to_string()), header);
    }
}

pub mod response {
    use super::*;

    const CONTENT_TYPE: &str = "Content-Type";
    const CONTENT_TYPE_JSON: &str = "application/json";
    const CONTENT_TYPE_PLAIN_TEXT: &str = "text/plain";

    pub fn create_json(status_code: i64, body: Option<String>) -> AlbTargetGroupResponse {
        create_with_content_type(
            status_code, body, CONTENT_TYPE_JSON.to_string()
        )
    }

    pub fn create_plain_text(status_code: i64, body: Option<String>) -> AlbTargetGroupResponse {
        create_with_content_type(
            status_code, body, CONTENT_TYPE_PLAIN_TEXT.to_string()
        )
    }

    pub fn create_with_content_type(
        status_code: i64,
        body: Option<String>,
        content_type: String
    ) -> AlbTargetGroupResponse {
        AlbTargetGroupResponse {
            status_code, body,
            headers: create_content_type_headers(&content_type),
            is_base64_encoded: false,
            status_description: None,
            multi_value_headers: Default::default()
        }
    }

    fn create_content_type_headers(value: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(CONTENT_TYPE.to_string(), value.to_string());
        headers
    }
}