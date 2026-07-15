# httpbin

[![Rust](https://github.com/DoumanAsh/httpbin/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/httpbin/actions/workflows/rust.yml)

Simple HTTP echo server to provide endpoint that echoes back request data.

## Echo request headers

Behavior of server can be configured via `x-echo-*` headers:

- `x-echo-delay` - Specifies the delay before sending response. Expects [grpc-timeout value](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md)

## Environment variables

- `PORT` - Specifies HTTP port to serve server. Defaults to 8080.
