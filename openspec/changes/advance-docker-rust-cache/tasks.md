## 1. Workflow cache configuration

- [ ] 1.1 Update the CI Docker Cargo cache to use a commit-specific primary key and a Dockerfile/Cargo.lock-scoped restore prefix
- [ ] 1.2 Apply the same cache key and restore prefix to release image validation without changing BuildKit GHA layer caching

## 2. Verification

- [ ] 2.1 Run workflow syntax and static checks and inspect the final workflow diff
- [ ] 2.2 Open a pull request and verify CI cache restore/save behavior and measured Docker build time
- [ ] 2.3 Request CodeRabbit review, address valid findings, and obtain approval

## 3. Specification lifecycle

- [ ] 3.1 Sync the docker-rust-build-cache delta spec into the main specifications
- [ ] 3.2 Archive the completed OpenSpec change
