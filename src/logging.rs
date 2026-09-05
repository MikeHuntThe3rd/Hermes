#![allow(warnings)]
use crate::types::{AppState, Logging};
use crate::responses::error_t::InternalError;
use axum::extract::{FromRef, FromRequestParts};
use serde::Serialize;

#[derive(Serialize)]
pub struct Header {
    pub key: String,
    pub val: String,
}

#[derive(Serialize)]
pub struct Req<T> {
    pub uri: String,
    pub method: String,
    pub headers: Vec<Header>,
    pub body: Option<T>,
}

#[derive(Serialize)]
pub struct Resp<T> 
where T: serde::Serialize
{
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Option<T>,
}
pub struct Logs<I, O> 
where I: serde::Serialize,
O: serde::Serialize
{
    pub id: Option<i64>,
    pub request: Req<I>,
    pub response: Resp<O>,
    pub err_msg: Option<String>,
}

impl<T, I, O> FromRequestParts<T> for Logs<I, O> 
where T: Sync + Send
, AppState: FromRef<T>,
O: serde::Serialize,
I: serde::Serialize
{
    type Rejection = InternalError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &T,
    ) -> Result<Self, Self::Rejection>
    {
        let uri_var = parts.uri.to_string();
        let meth = parts.method.to_string();
        let mut hdrs: Vec<Header> = vec![];

        for (key, val) in  parts.headers.iter() {
            hdrs.push(Header { key: key.to_string(), val: val.to_str().unwrap_or("failed to extract header val").to_string() });
        }

        Ok( Logs{
            id: None,
            request: Req { uri: uri_var, method: meth, headers: hdrs, body: None },
            response: Resp { status: 0, headers: vec![], body: None },
            err_msg: None
        } )   
    }
}

impl<I, O, T, E> Logging<I, O> for Result<T, E> 
where O: serde::Serialize,
I: serde::Serialize,
E: ToString
{
    fn log_if_err(self, logs: &mut Logs<I, O>) -> Self {
        if let Err(e) = &self {
            logs.err_msg = Some(e.to_string());
        }
        return self;
    }
}

impl<I, O> Logs<I, O> 
where O: serde::Serialize,
I: serde::Serialize
{
    async fn write_log(mut self, func: impl FnOnce(&Self)) -> Self {
        self
    }
}