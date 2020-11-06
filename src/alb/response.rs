use crate::alb;
use serde::Serialize;
use std::collections::HashMap;

/// Known content types.
pub mod content_types {
    pub const JSON: &str = "application/json";
    pub const PLAIN_TEXT: &str = "text/plain";
}

/// Creates an ALB-compatible response wrapping an optional Serde-Serializable object as Json.
pub fn create_json_from_optional<T: Serialize>(status: i64, optional: &Option<T>) -> alb::Response {
    match optional {
        Some(object) => create_json_from_obj(status, &object),
        None => create(status, None, Default::default()),
    }
}

/// Creates an ALB-compatible response wrapping a Serde-Serializable object as Json.
pub fn create_json_from_obj<T: Serialize>(status: i64, object: &T) -> alb::Response {
    match serde_json::to_string(object) {
        Ok(serialized) => create_json(status, Some(serialized)),
        Err(cause) => create_plain_text(500, Some(format!("{}", cause))),
    }
}

/// Creates an ALB-compatible response wrapping an optional object as JSON.
pub fn create_json(status_code: i64, body: Option<String>) -> alb::Response {
    create_with_content_type(status_code, body, content_types::JSON.to_string())
}

/// Creates an ALB-compatible response wrapping an optional String.
pub fn create_plain_text(status_code: i64, body: Option<String>) -> alb::Response {
    create_with_content_type(status_code, body, content_types::PLAIN_TEXT.to_string())
}

///
pub fn create_with_content_type(
    status_code: i64,
    body: Option<String>,
    content_type: String,
) -> alb::Response {
    create(
        status_code,
        body,
        headers::create_for(headers::CONTENT_TYPE, &content_type),
    )
}

/// Creates a normalised __mu::alb::Response__, taking care of a few details
/// that might lead to 502 errors on the Application Load Balancer.
pub fn create(
    status_code: i64,
    body: Option<String>,
    multi_value_headers: headers::Map,
) -> alb::Response {
    alb::Response {
        status_code,
        multi_value_headers,
        headers: HashMap::new(),
        is_base64_encoded: false,
        status_description: Some(format!("{} Response", status_code)),
        body: match body {
            None => Some("".to_string()),
            _ => body,
        },
    }
}

pub mod headers {
    use std::collections::HashMap;

    pub type Map = HashMap<String, Vec<String>>;

    pub const CONTENT_TYPE: &str = "Content-Type";
    pub const LOCATION: &str = "Content-Type";

    /// Creates a single entry header for the given __header_name__ and the optional __value__ arguments.
    pub fn create_for_optional(header_name: &str, optional_value: &Option<String>) -> Map {
        match optional_value {
            Some(value) => create_for(header_name, &value),
            None => Default::default(),
        }
    }

    /// Creates a single entry header for the given __header_name__ and __value__ arguments.
    pub fn create_for(header_name: &str, value: &str) -> Map {
        let values = vec![value.to_string()];
        let mut headers = HashMap::new();
        headers.insert(header_name.to_string(), values);
        headers
    }
}
