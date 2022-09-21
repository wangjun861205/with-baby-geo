FROM rust:1.63 AS builder
WORKDIR /with-baby-geo
COPY . .
RUN cargo build --release


FROM ubuntu:20.04
WORKDIR /app
COPY --from=builder /with-baby-geo/target/release/with-baby-geo ./
CMD ["/app/with-baby-geo"]
