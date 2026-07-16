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

### Start server

```sh
cargo run
```

The server applies pending migrations on startup.

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

```sh
# 1) Start containers
cp .env.template .env.docker
docker compose -f docker-compose-test.yml up -d

# 2) Database setup
docker compose -f docker-compose-test.yml exec -T db psql -U postgres -c "CREATE ROLE bookshelf WITH LOGIN PASSWORD 'password';"
docker compose -f docker-compose-test.yml exec -T db psql -U postgres -c "CREATE DATABASE bookshelf OWNER bookshelf;"

# 3) Start JWKS server (in a separate terminal)
cargo run -p bookshelf-e2e --bin bookshelf-jwks-server

# 4) Start application server (in a separate terminal)
PORT=8080 JWT_AUDIENCE=test-audience JWT_DOMAIN=test-issuer.local \
  JWKS_URL=http://localhost:9999/.well-known/jwks.json \
  DATABASE_URL=postgres://bookshelf:password@localhost:5432/bookshelf ALLOWED_ORIGINS=http://localhost:8080 \
  cargo run

# 5) Run E2E tests
TEST_SERVER_URL=http://localhost:8080 \
  cargo test -p bookshelf-e2e -- --test-threads=1
```

## GraphQL Playground

Run server and access `/graphql/playground`.

## Generate GraphQL schema

```sh
cargo run --bin gen_schema
```

## Deploy to production

Review the Release pull request maintained by
[tagpr](https://github.com/Songmu/tagpr), confirm that `release-pr-ci` has
succeeded, and merge it. The merge creates the matching Git tag and GitHub
Release, tests and publishes that exact version to GHCR, and deploys it to
Render.
