FROM rust:1.80.1 AS build-stage

RUN cargo new --bin bookshelf-api
WORKDIR /bookshelf-api

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./migrations ./migrations
COPY ./build.rs ./build.rs
RUN touch src/main.rs
RUN cargo build --release


FROM debian:bullseye-slim
COPY --from=build-stage /bookshelf-api/target/release/bookshelf-api /

RUN apt update
RUN apt install -y libssl-dev

CMD ["/bookshelf-api"]
