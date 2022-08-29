FROM rust:1.63 as builder
WORKDIR /with-baby-geo
COPY . .
RUN cargo build --release

FROM ubuntu:latest
WORKDIR /with-baby-geo
COPY --from=builder /with-baby-geo/target/release/with-baby-geo .
CMD ["/with-baby-geo/with-baby-geo"]
