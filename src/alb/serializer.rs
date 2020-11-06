use crate::alb;
use crate::alb::response;
use crate::lambda::LambdaError;
use aws_lambda_events::event::alb::AlbTargetGroupResponse;

/// Serialize ordinary structures and enums into an ALB valid response.
pub trait AlbSerialize {
    fn to_alb_response(&self) -> alb::Response;
}

impl AlbSerialize for alb::Response {
    fn to_alb_response(&self) -> AlbTargetGroupResponse {
        self.clone()
    }
}

impl AlbSerialize for LambdaError {
    fn to_alb_response(&self) -> alb::Response {
        let body = format!("{}", self);
        response::create_plain_text(500, Some(body))
    }
}

#[cfg(test)]
mod custom_serializer_tests {
    use super::*;
    use crate::alb::response::*;
    use serde::Serialize;

    impl AlbSerialize for User
    {
        fn to_alb_response(&self) -> alb::Response {
            response::create_json_from_obj(200, self)
        }
    }

    #[derive(Serialize)]
    struct User {
        name: String
    }

    #[test]
    fn should_convert_into_alb_response() {
        let serializable = User { name: String::from("John") };

        let response = serializable.to_alb_response();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body.unwrap(), "{\"name\":\"John\"}".to_string());

        let header = response.multi_value_headers.get(headers::CONTENT_TYPE);
        assert_ne!(None, header);
        assert_eq!(&content_types::JSON.to_string(), header.unwrap().get(0).unwrap());
    }
}


