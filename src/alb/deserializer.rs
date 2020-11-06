use crate::alb;
use crate::lambda;
use crate::lambda::LambdaError;

pub trait AlbDeserialize<T> {
    fn from_alb_request(req: alb::Request, ctx: lambda::Context) -> Result<T, LambdaError>;
}

impl AlbDeserialize<alb::Request> for alb::Request {
    fn from_alb_request(
        req: alb::Request,
        _: lambda::Context,
    ) -> Result<alb::Request, LambdaError> {
        Ok(req)
    }
}

pub trait RpcRequest {}

impl<T> alb::Deserialize<T> for T
where
    T: for<'de> serde::Deserialize<'de> + RpcRequest,
{
    fn from_alb_request(req: alb::Request, _ctx: lambda::Context) -> Result<T, LambdaError> {
        match &req.body {
            Some(body) => match serde_json::from_str(body) {
                Ok(deserialized) => Ok(deserialized),
                Err(cause) => Err(format!("Failed {:?}", cause).into()),
            },
            None => Err("No payload defined".into()),
        }
    }
}
