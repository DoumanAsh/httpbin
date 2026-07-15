use pin_project_lite::pin_project;
use hyper::server::conn::http1;
use tokio::net::TcpListener;

use httpbin::EchoServer;

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::task;
use core::pin::Pin;

//Fucking retarded hyper traits need wrapper
pin_project! {
    struct IoWrapper<T> {
        io: T,
    }
}

impl<T: tokio::io::AsyncRead + Unpin> hyper::rt::Read for IoWrapper<T> {
    #[inline(always)]
    fn poll_read(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, mut buf: hyper::rt::ReadBufCursor<'_>) -> task::Poll<Result<(), std::io::Error>> {
        let n = unsafe {
            let mut tbuf = tokio::io::ReadBuf::uninit(buf.as_mut());
            match tokio::io::AsyncRead::poll_read(Pin::new(self.project().io), ctx, &mut tbuf) {
                task::Poll::Ready(Ok(())) => tbuf.filled().len(),
                other => return other,
            }
        };

        unsafe {
            buf.advance(n);
        }
        task::Poll::Ready(Ok(()))
    }
}

impl<T: tokio::io::AsyncWrite + Unpin> hyper::rt::Write for IoWrapper<T> {
    #[inline(always)]
    fn poll_write(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, buf: &[u8]) -> task::Poll<Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write(Pin::new(self.project().io), ctx, buf)
    }

    #[inline(always)]
    fn poll_flush(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_flush(Pin::new(self.project().io), ctx)
    }

    #[inline(always)]
    fn poll_shutdown(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_shutdown(Pin::new(self.project().io), ctx)
    }

    #[inline(always)]
    fn is_write_vectored(&self) -> bool {
        tokio::io::AsyncWrite::is_write_vectored(&self.io)
    }

    #[inline(always)]
    fn poll_write_vectored(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, bufs: &[std::io::IoSlice<'_>]) -> task::Poll<Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write_vectored(Pin::new(self.project().io), ctx, bufs)
    }
}

async fn run() {
    let port = match env::var("PORT") {
        Err(_) => 8080,
        Ok(port) => port.parse().expect("env::PORT should be valid port value")
    };
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    let listener = TcpListener::bind(addr).await.expect("bind");
    println!("Listening on http://{}", addr);

    loop {
        let (io, _) = listener.accept().await.expect("accept");
        let io = IoWrapper {
            io,
        };

        tokio::task::spawn(async move {
            http1::Builder::new().serve_connection(io, EchoServer).await
        });
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("to create IO runtime");
    rt.block_on(run())
}
