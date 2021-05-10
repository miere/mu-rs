use serde::Serialize;
use aws_lambda_events::event::alb::AlbTargetGroupResponse as Response;
use http::{HeaderMap, HeaderValue};
use aws_lambda_events::encodings::Body;
use http::header::HeaderName;
use std::str::FromStr;

/// Known content types.
pub mod content_types {
    pub const JSON: &str = "application/json";
    pub const PLAIN_TEXT: &str = "text/plain";
}

/// Common header utilities.
pub mod headers {
    use std::collections::HashMap;

    pub const CONTENT_TYPE: &str = "Content-Type";
    pub const LOCATION: &str = "Content-Type";
    pub type HeaderMap = HashMap<String, String>;

    /// Creates a single entry header for the given __header_name__ and the optional __value__ arguments.
    pub fn create_for_optional(header_name: &str, optional_value: &Option<String>) -> HeaderMap {
        match optional_value {
            Some(value) => create_for(header_name, &value),
            None => Default::default(),
        }
    }

    /// Creates a single entry header for the given __header_name__ and __value__ arguments.
    pub fn create_for(header_name: &str, value: &str) -> HeaderMap {
        let mut headers = HashMap::new();
        headers.insert(header_name.to_string(), value.to_string());
        headers
    }
}

/// Creates an ALB-compatible response wrapping an optional Serde-Serializable object as Json.
pub fn create_json_from_optional<T: Serialize>(status: i64, optional: &Option<T>) -> Response {
    match optional {
        Some(object) => create_json_from_obj(status, &object),
        None => create(status, None, Default::default()),
    }
}

/// Creates an ALB-compatible response wrapping a Serde-Serializable object as Json.
pub fn create_json_from_obj<T: Serialize>(status: i64, object: &T) -> Response {
    match serde_json::to_string(object) {
        Ok(serialized) => create_json(status, Some(serialized)),
        Err(cause) => create_plain_text(500, Some(format!("{}", cause))),
    }
}

/// Creates an ALB-compatible response wrapping an optional object as JSON.
pub fn create_json(status_code: i64, body: Option<String>) -> Response {
    create_with_content_type(status_code, body, content_types::JSON.to_string())
}

/// Creates an ALB-compatible response wrapping an optional String.
pub fn create_plain_text(status_code: i64, body: Option<String>) -> Response {
    create_with_content_type(status_code, body, content_types::PLAIN_TEXT.to_string())
}

///
pub fn create_with_content_type(
    status_code: i64,
    body: Option<String>,
    content_type: String,
) -> Response {
    create(
        status_code,
        body,
        headers::create_for(headers::CONTENT_TYPE, &content_type),
    )
}

/// Creates a normalised [aws_lambda_events::event::alb::AlbTargetGroupResponse], taking care of
/// a few details that might lead to 502 errors on the Application Load Balancer.
pub fn create(
    status_code: i64,
    body: Option<String>,
    headers: headers::HeaderMap,
) -> Response {
    let mut adapted_headers = HeaderMap::with_capacity(headers.len());

    for (key, value) in headers.iter() {
        adapted_headers.insert(
            HeaderName::from_str(key).unwrap(),
            HeaderValue::from_str(value).unwrap()
        );
    }

    Response {
        status_code,
        multi_value_headers: adapted_headers,
        headers: HeaderMap::new(),
        is_base64_encoded: false,
        status_description: Some(format!("{} Response", status_code)),
        body: match body {
            None => Some(Body::Empty),
            Some(content) => Some(Body::Text(content)),
        },
    }
}
