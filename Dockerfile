FROM ubuntu:latest
WORKDIR /with-baby-geo
COPY target/release/with-baby-geo .
WORKDIR /with-baby-geo/h3-3.7.2/build
CMD ["/with-baby-geo/with-baby-geo"]
