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
[tagpr](https://github.com/Songmu/tagpr). Each update immediately creates three
informational commit statuses:

- `release-pr-ci` reports the Rust, image-build, migration, and schema checks.
- `release-pr-api-e2e` reports the API E2E suite.
- `release-pr-frontend-integration` reports compatibility with the Bookshelf
  frontend `main` branch.

These statuses are not configured as branch-protection required checks. Confirm
their results before merging the release pull request.

The merge creates the matching Git tag and GitHub Release. The release workflow
builds the Docker image once and runs API E2E against that image. API E2E is a
release gate: if it fails, the image is not pushed to GHCR and Render deployment
does not start. After API E2E succeeds, the exact validated image is pushed
without rebuilding it.

Render deployment and `Integration tests (bookshelf frontend)` then run
independently. A frontend integration failure makes the release workflow fail
but does not stop or cancel image publication or Render deployment.
