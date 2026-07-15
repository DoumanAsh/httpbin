FROM douman/rust-musl:latest as build

WORKDIR /src

COPY . /src

RUN cargo build --release

FROM scratch
COPY --from=build /src/target/release/httpbin /httpbin
CMD ["/httpbin"]
