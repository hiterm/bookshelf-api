![ci](https://github.com/hiterm/bookshelf-api/actions/workflows/ci.yml/badge.svg)
![deploy](https://github.com/hiterm/bookshelf-api/actions/workflows/deploy.yml/badge.svg)

# Bookshelf API

Backend server for [Bookshelf](https://github.com/hiterm/bookshelf/).

## How to run server

### Set up Auth0

Set up auth0 by following:

https://auth0.com/developers/hub/code-samples/api/actix-web-rust/basic-authorization

### Setup .env

```sh
mv .env.template .env
vim .env  # Fill your value
```

### Run migration

```sh
cargo install sqlx-cli
sqlx migrate run
```

### Start server

```sh
cargo run
```

### Run via Docker Compose

```sh
cp .env.template .env.docker
vim .env.docker  # Fill your value
```

```sh
docker-compose up --build
```

## Test

```sh
cargo test
```

## E2E test

```
# 1) 起動
cp .env.template .env.docker
docker compose -f docker-compose-test.yml up -d
```

# 2) データベース準備（必要なら sqlx をインストール）
cargo install sqlx-cli --no-default-features --features postgres,rustls
sqlx database create
sqlx migrate run
docker compose -f docker-compose-test.yml exec -T db psql -U postgres -c "CREATE ROLE bookshelf WITH LOGIN PASSWORD 'password';"
docker compose -f docker-compose-test.yml exec -T db psql -U postgres -c "CREATE DATABASE bookshelf OWNER bookshelf;"

# 3) E2E 実行
PORT=8080 AUTH0_AUDIENCE=test-audience AUTH0_DOMAIN=example.com DATABASE_URL=postgres://bookshelf:password@localhost:5432/bookshelf \
  cargo test -p bookshelf-e2e -- --test-threads=1
```

## GraphQL Playground

Run server and access `/graphql/playground`.

## Generate GraphQL schema

```
cargo run --bin gen_schema
```

## Deploy to production

Publish a new release.
