//! Provides abstractions for Alb Request serialization.

use std::fmt::Debug;

use aws_lambda_events::event::alb::AlbTargetGroupResponse;
use serde::Serialize;

use crate::response;

/// Serialize ordinary structures and enums into an ALB valid response.
pub trait AlbSerialize {
    fn to_alb_response(&self) -> AlbTargetGroupResponse;
}

impl AlbSerialize for AlbTargetGroupResponse {
    fn to_alb_response(&self) -> AlbTargetGroupResponse {
        self.clone()
    }
}

impl AlbSerialize for mu_runtime::Error {
    fn to_alb_response(&self) -> AlbTargetGroupResponse {
        let body = format!("{}", self);
        response::create_as_plain_text(500, Some(body))
    }
}

impl<T, E> AlbSerialize for Result<T, E>
where
    T: Serialize,
    E: Debug,
{
    fn to_alb_response(&self) -> AlbTargetGroupResponse {
        match self {
            Ok(response) => response::create_json_from_obj(200, response),
            Err(cause) => response::create_as_plain_text(
                500,
                Some(format!("Internal Server Error: {:?}", cause)),
            ),
        }
    }
}

#[cfg(test)]
mod custom_serializer_tests {
    use serde::Serialize;
    use crate::response::*;
    use super::*;
    use aws_lambda_events::encodings::Body;

    impl AlbSerialize for User {
        fn to_alb_response(&self) -> AlbTargetGroupResponse {
            response::create_json_from_obj(200, self)
        }
    }

    #[derive(Serialize)]
    struct User {
        name: String,
    }

    #[test]
    #[cfg(feature = "multi_header")]
    fn should_convert_into_alb_response() {
        let serializable = User {
            name: String::from("John"),
        };

        let response = serializable.to_alb_response();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body.unwrap(), Body::Text("{\"name\":\"John\"}".to_string()));

        let header = response.multi_value_headers.get(headers::CONTENT_TYPE);
        assert_ne!(None, header);
        assert_eq!(content_types::JSON, header.unwrap().to_str().unwrap());
    }

    #[test]
    #[cfg(not(feature = "multi_header"))]
    fn should_convert_into_alb_response() {
        let serializable = User {
            name: String::from("John"),
        };

        let response = serializable.to_alb_response();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body.unwrap(), Body::Text("{\"name\":\"John\"}".to_string()));

        let header = response.headers.get(headers::CONTENT_TYPE);
        assert_ne!(None, header);
        assert_eq!(content_types::JSON, header.unwrap().to_str().unwrap());
    }
}

#[cfg(test)]
mod result_object_serialization_tests {
    use mu_runtime::RuntimeResult;
    use super::*;
    use aws_lambda_events::encodings::Body;

    #[test]
    fn should_serialize_successful_result() {
        let res: RuntimeResult = Ok(());

        let response = res.to_alb_response();
        assert_eq!(200, response.status_code);
        assert_eq!(Body::Text("null".to_string()), response.body.unwrap());
    }

    #[test]
    fn should_serialize_failure_result() {
        let res: RuntimeResult = Err("Unit Test".into());

        let response = res.to_alb_response();
        assert_eq!(500, response.status_code);
        assert_eq!(
            Body::Text("Internal Server Error: Error(\"Unit Test\")".to_string()),
            response.body.unwrap()
        );
    }
}
