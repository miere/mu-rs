use std::collections::HashMap;
use serde::Serialize;
use crate::alb;

pub mod headers {
    pub const CONTENT_TYPE: &str = "Content-Type";
    pub const LOCATION: &str = "Content-Type";
}

pub mod content_types {
    pub const JSON: &str = "application/json";
    pub const PLAIN_TEXT: &str = "text/plain";
}

/// Creates an ALB-compatible response from an optional Serde-Serializable object.
pub fn create_json_from_optional<T: Serialize>(status: i64, optional: &Option<T>) -> alb::Response {
    match optional {
        Some(object) => create_json_from_obj(status, &object),
        None => create(status, None, Default::default() )
    }
}

/// Creates an ALB-compatible response from a Serde-Serializable object.
pub fn create_json_from_obj<T: Serialize>(status: i64, object: &T) -> alb::Response {
    match serde_json::to_string(object) {
        Ok(serialized) => create_json(
            status, Some(serialized),
        ),
        Err(cause) => create_plain_text(
            500, Some(format!("{}", cause)),
        )
    }
}

/// Creates an ALB-compatible response wrapping a JSON object.
pub fn create_json(status_code: i64, body: Option<String>) -> alb::Response {
    create_with_content_type(
        status_code, body, content_types::JSON.to_string()
    )
}

pub fn create_plain_text(status_code: i64, body: Option<String>) -> alb::Response {
    create_with_content_type(
        status_code, body, content_types::PLAIN_TEXT.to_string()
    )
}

pub fn create_with_content_type(
    status_code: i64,
    body: Option<String>,
    content_type: String
) -> alb::Response {
    create(
        status_code, body,
        create_header_for(headers::CONTENT_TYPE, &content_type)
    )
}

pub fn create_optional_header_for(header_name: &str, optional_value: &Option<String>) -> HashMap<String, String> {
    match optional_value {
        Some(value) => create_header_for(header_name, &value),
        None => Default::default()
    }
}

pub fn create_header_for(header_name: &str, value: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert(header_name.to_string(), value.to_string());
    headers
}

pub fn create(
    status_code: i64,
    body: Option<String>,
    headers: HashMap<String, String>
) -> alb::Response {
    alb::Response {
        status_code, body, headers,
        is_base64_encoded: false,
        status_description: None,
        multi_value_headers: Default::default()
    }
}
