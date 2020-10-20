use crate::alb;
use crate::lambda;
use crate::lambda::LambdaError;

pub trait AlbDeserialize<T> {
    fn from_alb_request(req: alb::Request, ctx: lambda::Context) -> Result<T, LambdaError>;
}

impl AlbDeserialize<alb::Request> for alb::Request {
    fn from_alb_request(req: alb::Request, _: lambda::Context) -> Result<alb::Request, LambdaError> {
        Ok(req)
    }
}