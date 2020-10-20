use aws_lambda_events::event::alb::AlbTargetGroupResponse;
use serde::Serialize;

use crate::alb::response;
use crate::lambda::LambdaError;

/// Serialize ordinary structures and enums into an ALB valid response.
pub trait AlbSerialize {
    fn to_alb_response(&self) -> AlbTargetGroupResponse;
}

// Converts structures and enums marked with Serialize into a valid ALB response.
impl<T> AlbSerialize for T
    where T: Serialize
{
    fn to_alb_response(&self) -> AlbTargetGroupResponse {
        response::create_json_from_obj(200, self)
    }
}

impl AlbSerialize for LambdaError {
    fn to_alb_response(&self) -> AlbTargetGroupResponse {
        let body = format!("{}", self);
        response::create_plain_text(500, Some(body))
    }
}

#[cfg(test)]
mod handled_response_tests {
    use super::*;
    use crate::alb::response::*;

    #[derive(Serialize)]
    struct User {
        name: String
    }

    #[test]
    fn should_convert_serializable_into_response() {
        let serializable = User { name: String::from("John") };

        let response = serializable.to_alb_response();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body.unwrap(), "{\"name\":\"John\"}".to_string());

        let header = response.headers.get(headers::CONTENT_TYPE);
        assert_ne!(None, header);
        assert_eq!(Some(&content_types::JSON.to_string()), header);
    }
}


