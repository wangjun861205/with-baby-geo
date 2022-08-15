FROM rust:latest as builder
WORKDIR /with-baby-geo
COPY . .
RUN cargo build

FROM alpine:latest
WORKDIR /with-baby-geo
COPY --from=builder /with-baby-geo/target/release/with-baby-geo .
ENTRYPOINT ["/with-baby-geo/with-baby-geo"]