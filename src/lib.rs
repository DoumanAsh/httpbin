#![doc = include_str!("../README.md")]

#![allow(clippy::style)]

use core::{fmt, convert, task};
use core::future::Future;
use core::pin::Pin;

use tokio::time;
use hyper::{Request, Response, Uri, Method, HeaderMap};
use hyper::body::Incoming;
use hyper::service::Service;

pub const ECHO_SERVER_HEADER_PREFIX: &str = "x-echo-";

pub struct RequestParams {
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    time: time::Instant,
}

impl fmt::Debug for RequestParams {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elapsed = self.time.elapsed();
        fmt.debug_map()
           .entry(&"method", &self.method)
           .entry(&"uri", &self.uri)
           .entry(&"headers", &self.headers)
           .entry(&"elapsed", &elapsed)
           .finish()
    }
}

pub struct EchoServer;

pub struct ResponseFuture {
    delay: Option<time::Sleep>,
    body: RequestParams,
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

        task::Poll::Ready(Ok(Response::new(format!("{:#?}", this.body))))
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
        for (name, value) in headers.iter() {
            if let Some(param) = name.as_str().strip_prefix(ECHO_SERVER_HEADER_PREFIX) {
                if param.eq_ignore_ascii_case("delay") {
                    if let Ok(duration) = timeout_context::try_parse_timeout(value.as_bytes()) {
                        delay = Some(tokio::time::sleep_until(time + duration));
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
        }
    }
}
