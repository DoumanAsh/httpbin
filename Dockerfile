FROM quay.io/doumanash/rust-musl:latest AS build

WORKDIR /src

COPY . /src

RUN cargo build --release

FROM scratch
COPY --from=build /src/target/release/httpbin /httpbin
CMD ["/httpbin"]
