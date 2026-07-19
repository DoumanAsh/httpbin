#![doc = include_str!("../README.md")]

#![allow(clippy::style)]

use std::io;
use core::pin::Pin;
use core::future::Future;
use core::{convert, task};

use tokio::time;
use hyper::{Request, Response, Uri, Method, HeaderMap};
use hyper::body::Incoming;
use hyper::service::Service;

pub const ECHO_SERVER_HEADER_PREFIX: &str = "x-echo-";
const APP_JSON: http::HeaderValue = http::HeaderValue::from_static("application/json");

struct HttpParams {
    status: http::StatusCode,
}

pub struct RequestParams {
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    time: time::Instant,
}

impl RequestParams {
    pub fn write(&self, out: &mut impl io::Write) -> io::Result<()> {
        const TAB: &str = "    ";

        let elapsed = self.time.elapsed();
        out.write_all("{\n".as_bytes())?;
        out.write_fmt(format_args!("{TAB}\"method\": \"{}\",\n", self.method))?;
        out.write_fmt(format_args!("{TAB}\"uri\": \"{}\",\n", self.uri.path()))?;

        let headers_len = self.headers.len();

        out.write_fmt(format_args!("{TAB}\"headers\": {{\n"))?;
        for (idx, (name, value)) in self.headers.iter().enumerate() {
            if let Ok(value) = value.to_str() {
                out.write_fmt(format_args!("{TAB}{TAB}\"{name}\": \"{value}\""))?;
                if idx < (headers_len - 1) {
                    out.write_all(",\n".as_bytes())?;
                } else {
                    out.write_all("\n".as_bytes())?;
                }
            }
        }
        out.write_all(TAB.as_bytes())?;
        out.write_all("},\n".as_bytes())?;

        out.write_fmt(format_args!("{TAB}\"elapsed\": \"{:?}\"\n", elapsed))?;
        out.write_all("}".as_bytes())?;
        Ok(())
    }
}

pub struct EchoServer;

pub struct ResponseFuture {
    delay: Option<time::Sleep>,
    body: RequestParams,
    http_params: HttpParams,
}

impl Future for ResponseFuture {
    type Output = Result<Response<String>, convert::Infallible>;

    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let this = unsafe {
            self.get_unchecked_mut()
        };

        if let Some(delay) = this.delay.as_mut() {
            let delay_ptr = unsafe {
                Pin::new_unchecked(delay)
            };

            match Future::poll(delay_ptr, ctx) {
                task::Poll::Pending => return task::Poll::Pending,
                task::Poll::Ready(_) => {
                    this.delay = None;
                }
            }
        }

        let mut body = Vec::<u8>::new();
        let _ = this.body.write(&mut body);
        let body = unsafe {
            String::from_utf8_unchecked(body)
        };
        let mut response = Response::new(body);
        *response.status_mut() = this.http_params.status;
        response.headers_mut().insert(http::header::CONTENT_TYPE, APP_JSON);
        task::Poll::Ready(Ok(response))
    }
}

impl Service<Request<Incoming>> for EchoServer {
    type Response = Response<String>;
    type Error = core::convert::Infallible;
    type Future = ResponseFuture;

    #[inline(always)]
    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let (http::request::Parts { method, uri, headers, .. }, _) = req.into_parts();

        let mut delay = None;
        let time = time::Instant::now();
        let mut status = http::StatusCode::OK;
        for (name, value) in headers.iter() {
            if let Some(param) = name.as_str().strip_prefix(ECHO_SERVER_HEADER_PREFIX) {
                if param.eq_ignore_ascii_case("delay") {
                    if let Ok(duration) = timeout_context::try_parse_timeout(value.as_bytes()) {
                        delay = Some(tokio::time::sleep_until(time + duration));
                    }
                } else if param.eq_ignore_ascii_case("status") {
                    if let Ok(new_status) = http::StatusCode::from_bytes(value.as_bytes()) {
                        status = new_status;
                    }
                }
            }
        }
        let body = RequestParams {
            uri,
            method,
            headers,
            time,
        };

        ResponseFuture {
            delay,
            body,
            http_params: HttpParams {
                status
            }
        }
    }
}
