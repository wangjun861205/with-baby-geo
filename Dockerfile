FROM rust:1.63
WORKDIR /with-baby-geo
COPY . .
RUN cargo build --release
CMD ["/with-baby-geo/target/release/with-baby-geo"]
