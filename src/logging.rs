use crate::types::{AppState, Logging, Res};
use crate::responses::error_t::InternalError;
use axum::extract::{FromRequest, FromRef, Request};
use axum::http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct Header {
    pub key: String,
    pub val: String,
}

#[derive(Serialize)]
pub struct Req {
    pub uri: String,
    pub method: String,
    pub headers: Vec<Header>,
    pub body: String,
}
pub struct Logs<O> 
where O: serde::Serialize
{
    pub id: Option<i64>,
    pub request: Req,
    pub response: Res<O>,
}

impl<T, O> FromRequest<T> for Logs<O> 
where T: Sync + Send, AppState: FromRef<T>,
O: serde::Serialize
{
    type Rejection = InternalError;

    async fn from_request(
        req: Request,
        _state: &T,
    ) -> Result<Self, Self::Rejection>
    {
        let uri_var = req.uri().to_string();
        let meth = req.method().to_string();
        let mut hdrs: Vec<Header> = vec![];

        for (key, val) in  req.headers().iter() {
            hdrs.push(Header { key: key.to_string(), val: val.to_str().unwrap_or("failed to extract header val").to_string() });
        }

        let bdy: String = if uri_var.contains("upload") {
            "body is not provided for uploads".to_string()
        } else {
            req.body()
        };
        
        Ok( Logs{
            id: None,
            request: Req { uri: uri_var, method: meth, headers: hdrs, body: bdy },
            response: Res {status: StatusCode::OK, success: true, msg: String::new(), data: None},
        } )
    }
}

impl<O, T, E> Logging<O> for Result<T, E> 
where O: serde::Serialize
{
    async fn log_if_err(self, logs: &mut Logs<O>) -> Self {
        if self.is_err() {
            
        }
        return self;
    }
}
impl<O> Logs<O> 
where O: serde::Serialize
{
    async fn write_log(mut self, func: impl FnOnce(&Self)) -> Self {
        self
    }
}