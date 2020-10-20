//! This is a sample controller that mimics an RPC-Style API, expecting
//! one object representing a request and returning an objecting describing
//! the execution result.

use lambda_runtime_ext::http::*;
use lambda_runtime_ext::alb::*;
use serde::{Serialize, Deserialize};

/// A RPC-style User Crud API
pub struct UserRpcController {}

#[allow(dead_code)]
impl UserRpcController {

    pub async fn create(&self, req: User) -> HttpResponse {
        let location = format!("/users/{}", req.email);
        HttpResponse::Created(Some(location))
    }

    pub async fn retrieve(&self, _: String) -> HttpResponse {
        HttpResponse::NotFound
    }

    pub async fn update(&self, email: String, user: User) -> Response {
        if email.ne(&user.email) {
            response::create_plain_text(422, Some("You can't modify user email".to_string()))
        } else {
            response::create(204, None, Default::default())
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub name: String
}