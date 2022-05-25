![ci](https://github.com/hiterm/bookshelf-api/actions/workflows/ci.yml/badge.svg)

# Bookshelf API

Backend server for [Bookshelf](https://github.com/hiterm/bookshelf/).

## How to run server

### Set up Auth0

Set up auth0 by following:

https://auth0.com/developers/hub/code-samples/api/actix-web-rust/basic-authorization

### Setup .env

```sh
$ mv .env.template .env
$ vim .env  # Fill your value
```

### Run migration

```
$ cargo install sqlx-cli
$ sqlx migrate run
```

### Start server

```
$ cargo run
```

### Run via Docker Compose

```sh
$ mv .env.template .env.docker
$ vim .env.docker  # Fill your value
```

```
$ docker-compose up --build
```

## Test

```
$ cargo test
```

With DB

```
$ docker-compose -f docker-compose-test.yml up -d
$ cargo test -- --include-ignored
```

## GraphQL Playground

Run server and access `/graphql/playground`.

## Generate GraphQL schema

```
$ cargo run --bin gen_schema
```
