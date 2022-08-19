FROM rust:1.63.0-alpine3.16 as builder
RUN apk update
RUN apk add cmake
RUN apk add musl-dev
WORKDIR /with-baby-geo
COPY . .
WORKDIR /with-baby-geo/h3-3.7.2/build
RUN rm -rf *
RUN cmake ..
RUN make
RUN cargo build --release

FROM alpine:latest
WORKDIR /with-baby-geo
COPY --from=builder /with-baby-geo/target/release/with-baby-geo ./
CMD ["/with-baby-geo/with-baby-geo"]