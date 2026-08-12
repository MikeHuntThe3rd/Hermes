use crate::types::{Res, AppState};
use crate::responses::error_t::InternalError;
use axum::Json;
use axum::extract::{FromRequest, FromRef, Request};
use axum::http::StatusCode;
use axum::body::Body;

pub struct Header {
    pub key: String,
    pub val: String,
}

pub struct Req {
    pub uri: String,
    pub method: String,
    pub headers: Vec<Header>,
    pub body: Json<Body>,
}
pub struct Log<O> 
where O: serde::Serialize
{
    pub id: Option<i64>,
    pub request: Req,
    pub response: Res<O>,
}

impl<T, O> FromRequest<T> for Log<O> 
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

        let bdy: Json<Body> = if uri_var.contains("upload") {
            Json(Body::empty())
        } else {
            Json(Body::new(req))
        };
        
        Ok( Log{
            id: None,
            request: Req { uri: uri_var, method: meth, headers: hdrs, body: bdy },
            response: Res {status: StatusCode::OK, success: true, msg: String::new(), data: None},
        } )
    }
}

impl<O> Log<O> 
where O: serde::Serialize
{
    async fn write_log() {
        
    }
}