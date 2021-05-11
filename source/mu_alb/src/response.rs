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

    #[cfg(not(feature = "multi_header"))]
    pub type HeaderMap = HashMap<String, String>;

    #[cfg(feature = "multi_header")]
    pub type HeaderMap = HashMap<String, Vec<String>>;

    /// Creates a single entry header for the given __header_name__ and __value__ arguments.
    #[cfg(not(feature = "multi_header"))]
    pub fn create_for(header_name: &str, value: &str) -> HeaderMap {
        let mut headers = HashMap::new();
        headers.insert(header_name.to_string(), value.to_string());
        headers
    }

    /// Creates a single entry header for the given __header_name__ and __value__ arguments.
    #[cfg(feature = "multi_header")]
    pub fn create_for(header_name: &str, value: &str) -> HeaderMap {
        let mut headers = HashMap::new();
        headers.insert(header_name.to_string(), vec![value.to_string()]);
        headers
    }
}

/// Creates an ALB-compatible response wrapping a Serde-Serializable object as Json.
pub fn create_json_from_obj<T: Serialize>(status: i64, object: &T) -> Response {
    match serde_json::to_string(object) {
        Ok(serialized) => create_as_json(status, Some(serialized)),
        Err(cause) => create_as_plain_text(500, Some(format!("{}", cause))),
    }
}

/// Creates an ALB-compatible response wrapping an optional object as JSON.
pub fn create_as_json(status_code: i64, body: Option<String>) -> Response {
    create_with_content_type(status_code, body, content_types::JSON.to_string())
}

/// Creates an ALB-compatible response wrapping an optional String.
pub fn create_as_plain_text(status_code: i64, body: Option<String>) -> Response {
    create_with_content_type(status_code, body, content_types::PLAIN_TEXT.to_string())
}

///
pub fn create_with_content_type(
    status_code: i64,
    body: Option<String>,
    content_type: String,
) -> Response {
    create( status_code, body,
        headers::create_for(headers::CONTENT_TYPE, &content_type) )
}

/// Creates a normalised [aws_lambda_events::event::alb::AlbTargetGroupResponse], taking care of
/// a few details that might lead to 502 errors on the Application Load Balancer.
#[cfg(not(feature = "multi_header"))]
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

    create_with_header(status_code, body, adapted_headers)
}

#[cfg(not(feature = "multi_header"))]
fn create_with_header(
    status_code: i64,
    body: Option<String>,
    headers: HeaderMap
) -> Response {
    Response {
        status_code,
        headers,
        multi_value_headers: HeaderMap::new(),
        is_base64_encoded: false,
        status_description: Some(format!("{} Response", status_code)),
        body: match body {
            None => Some(Body::Empty),
            Some(content) => Some(Body::Text(content)),
        }
    }
}

/// Creates a normalised [aws_lambda_events::event::alb::AlbTargetGroupResponse], taking care of
/// a few details that might lead to 502 errors on the Application Load Balancer.
#[cfg(feature = "multi_header")]
pub fn create(
    status_code: i64,
    body: Option<String>,
    headers: headers::HeaderMap,
) -> Response {
    let mut adapted_headers = HeaderMap::with_capacity(headers.len());

    for (key, values) in headers.iter() {
        for value in values {
            adapted_headers.append(
                HeaderName::from_str(key).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
    }

    create_with_header(status_code, body, adapted_headers)
}

#[cfg(feature = "multi_header")]
fn create_with_header(
    status_code: i64,
    body: Option<String>,
    headers: HeaderMap
) -> Response {
    Response {
        status_code,
        multi_value_headers: headers,
        headers: HeaderMap::new(),
        is_base64_encoded: false,
        status_description: Some(format!("{} Response", status_code)),
        body: match body {
            None => Some(Body::Empty),
            Some(content) => Some(Body::Text(content)),
        }
    }
}

#[cfg(test)]
mod tests {

    mod response_creation {
        use crate::response;
        use http::{HeaderMap, HeaderValue};
        use http::header::HeaderName;
        use std::str::FromStr;

        #[test]
        #[cfg(not(feature = "multi_header"))]
        fn should_create_response_with_headers() {
            let mut headers = response::headers::HeaderMap::new();
            headers.insert("Content-Type".to_string(), "text/plain".to_string());

            let alb_response = response::create(200, Some("hello".to_string()), headers);

            let mut expected_headers = HeaderMap::with_capacity(1);
            expected_headers.insert(
                HeaderName::from_str("Content-Type").unwrap(),
                HeaderValue::from_str("text/plain").unwrap()
            );

            assert_eq!(expected_headers, alb_response.headers)
        }

        #[test]
        #[cfg(feature = "multi_header")]
        fn should_create_response_with_multi_headers() {
            let mut headers = response::headers::HeaderMap::new();
            headers.insert("Content-Type".to_string(), vec!["text/plain".to_string()]);

            let alb_response = response::create(200, Some("hello".to_string()), headers);

            let mut expected_headers = HeaderMap::with_capacity(1);
            expected_headers.insert(
                HeaderName::from_str("Content-Type").unwrap(),
                HeaderValue::from_str("text/plain").unwrap()
            );

            assert_eq!(expected_headers, alb_response.multi_value_headers)
        }
    }

}