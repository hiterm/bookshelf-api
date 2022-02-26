FROM rust:1.58

RUN cargo new --bin bookshelf-api
WORKDIR /app

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
RUN touch src/main.rs
RUN cargo install --locked --path .

EXPOSE 8080

CMD ["bookshelf-api"]
