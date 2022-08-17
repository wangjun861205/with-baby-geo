FROM rust:latest as builder
WORKDIR /with-baby-geo
COPY . .
RUN cargo build --release

FROM alpine:latest
WORKDIR /with-baby-geo
COPY --from=builder /with-baby-geo/target/release/with-baby-geo ./
ENTRYPOINT [ "/bin/sh", "-l", "-c" ]
CMD ["/with-baby-geo/with-baby-geo"]