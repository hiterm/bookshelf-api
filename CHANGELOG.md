# Changelog

## [2.10.0](https://github.com/hiterm/bookshelf-api/compare/2.9.0...2.10.0) - 2026-07-20

### Other Changes
- Expose author yomi in GraphQL by @hiterm in https://github.com/hiterm/bookshelf-api/pull/282
- Harden release validation by @hiterm in https://github.com/hiterm/bookshelf-api/pull/286
- Fix release PR head resolution by @hiterm in https://github.com/hiterm/bookshelf-api/pull/289

## [2.9.0](https://github.com/hiterm/bookshelf-api/compare/2.8.2...2.9.0) - 2026-07-17

### Other Changes
- Use cargo set-version in tagpr by @hiterm in https://github.com/hiterm/bookshelf-api/pull/279
- Sync Rust versions in Renovate by @hiterm in https://github.com/hiterm/bookshelf-api/pull/281
- Document Codex gh sandbox behavior by @hiterm in https://github.com/hiterm/bookshelf-api/pull/283
- Unify mutation payloads by @hiterm in https://github.com/hiterm/bookshelf-api/pull/284

## [2.8.2](https://github.com/hiterm/bookshelf-api/compare/2.8.1...2.8.2) - 2026-07-16

### Other Changes
- Adopt tagpr release workflow by @hiterm in https://github.com/hiterm/bookshelf-api/pull/277

## [2.8.1](https://github.com/hiterm/bookshelf-api/compare/2.8.0...2.8.1) - 2026-07-15

- chore: bump version to 2.8.0 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/274
- chore: bump version to 2.8.1 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/275
- Restore bookshelf integration target by @hiterm in https://github.com/hiterm/bookshelf-api/pull/276

## [2.8.0](https://github.com/hiterm/bookshelf-api/compare/2.7.2...2.8.0) - 2026-07-15

- Remove Actions concurrency limits by @hiterm in https://github.com/hiterm/bookshelf-api/pull/270
- Update taiki-e/install-action action to v2.83.1 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/267
- Update Rust crate mockall to 0.15.0 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/259
- Update Rust crate tower-http to 0.7.0 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/241
- Update debian:trixie-slim Docker digest to 020c0d2 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/244
- Update postgres:latest Docker digest to b913fd5 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/192
- chore: bump version to 2.7.3 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/271
- Update Rust crate sqlx to 0.9 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/227
- Return event set IDs from mutations by @hiterm in https://github.com/hiterm/bookshelf-api/pull/269

## [2.7.2](https://github.com/hiterm/bookshelf-api/compare/2.7.1...2.7.2) - 2026-07-09

- [codex] Prepare integration test in parallel by @hiterm in https://github.com/hiterm/bookshelf-api/pull/254
- chore: bump version to 2.7.2 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/255
- Lock file maintenance by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/216
- Update actions/cache action to v6 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/245
- Update actions/checkout action to v7 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/243
- Transaction-owned user_id for mutating repos by @hiterm in https://github.com/hiterm/bookshelf-api/pull/261
- Split E2E tests by workflow and extract shared helpers by @hiterm in https://github.com/hiterm/bookshelf-api/pull/260
- Update cargo non-major by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/221
- Exclude Rust from release age gate by @hiterm in https://github.com/hiterm/bookshelf-api/pull/263
- Fix duplicate Rust Renovate updates by @hiterm in https://github.com/hiterm/bookshelf-api/pull/265
- Update Rust to v1.96.1 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/242
- Update github-actions non-major by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/240

## [2.7.1](https://github.com/hiterm/bookshelf-api/compare/2.7.0...2.7.1) - 2026-07-04

- Speed up GitHub Actions: add concurrency, drop redundant check job by @hiterm in https://github.com/hiterm/bookshelf-api/pull/239
- Update debian:trixie-slim Docker digest to 4e401d9 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/191
- Update github-actions non-major by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/158
- chore: bump version to 2.7.1 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/223
- Update rust-toolchain non-major to v1.96.0 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/226
- Update rust version to v1.96.0 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/193
- [codex] Add Rust toolchain components by @hiterm in https://github.com/hiterm/bookshelf-api/pull/247
- Add transactional book lookup and use it in UpdateBook interactor by @hiterm in https://github.com/hiterm/bookshelf-api/pull/246
- [codex] Update clippy pre-commit check by @hiterm in https://github.com/hiterm/bookshelf-api/pull/250
- [codex] Remove stale sqlx-cli docs by @hiterm in https://github.com/hiterm/bookshelf-api/pull/248
- Refactor Book updates to domain operation by @hiterm in https://github.com/hiterm/bookshelf-api/pull/249
- Speed up test image workflow by @hiterm in https://github.com/hiterm/bookshelf-api/pull/252
- Add bulk import E2E coverage by @hiterm in https://github.com/hiterm/bookshelf-api/pull/253
- Align author update with book by @hiterm in https://github.com/hiterm/bookshelf-api/pull/251

## [2.7.0](https://github.com/hiterm/bookshelf-api/compare/2.6.1...2.7.0) - 2026-06-13

- Move transaction control to use-case layer; remove ImportBooksRepository by @hiterm in https://github.com/hiterm/bookshelf-api/pull/232
- Move event recording design to docs/architecture by @hiterm in https://github.com/hiterm/bookshelf-api/pull/234
- Add eventSets and eventSet GraphQL queries by @hiterm in https://github.com/hiterm/bookshelf-api/pull/235
- Refactor E2E tests: extract helper functions and improve cleanup by @hiterm in https://github.com/hiterm/bookshelf-api/pull/236
- Refactor EventSet operation field to use typed enum by @hiterm in https://github.com/hiterm/bookshelf-api/pull/237
- Remove sqlx-cli installation and database migration steps by @hiterm in https://github.com/hiterm/bookshelf-api/pull/238

## [2.6.1](https://github.com/hiterm/bookshelf-api/compare/2.6.0...2.6.1) - 2026-06-09

- Add minimumReleaseAge to Renovate config by @hiterm in https://github.com/hiterm/bookshelf-api/pull/228
- Migrate CI workflows from npm to pnpm by @hiterm in https://github.com/hiterm/bookshelf-api/pull/231

## [2.6.0](https://github.com/hiterm/bookshelf-api/compare/2.5.0...2.6.0) - 2026-05-31

- Remove Claude Code GitHub Actions by @hiterm in https://github.com/hiterm/bookshelf-api/pull/222
- openspec init by @hiterm in https://github.com/hiterm/bookshelf-api/pull/224
- Add importBooks mutation by @hiterm in https://github.com/hiterm/bookshelf-api/pull/225

## [2.5.0](https://github.com/hiterm/bookshelf-api/compare/2.4.2...2.5.0) - 2026-04-30

- Add change history tracking for books and authors by @hiterm in https://github.com/hiterm/bookshelf-api/pull/220
- chore: bump version to 2.5.0 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/219

## [2.4.2](https://github.com/hiterm/bookshelf-api/compare/2.4.1...2.4.2) - 2026-04-29

- Revert "Add devcontainer config and startup script for OpenCode Web" by @hiterm in https://github.com/hiterm/bookshelf-api/pull/218

## [2.4.1](https://github.com/hiterm/bookshelf-api/compare/2.4.0...2.4.1) - 2026-04-23

- Add bookshelf frontend e2e-integration job to CI by @hiterm in https://github.com/hiterm/bookshelf-api/pull/213
- Run e2e-integration tests before image publish by @hiterm in https://github.com/hiterm/bookshelf-api/pull/215
- chore: bump version to 2.4.1 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/214

## [2.4.0](https://github.com/hiterm/bookshelf-api/compare/2.3.3...2.4.0) - 2026-04-22

- Replace once_cell with std::sync::LazyLock and dotenv with dotenvy by @hiterm in https://github.com/hiterm/bookshelf-api/pull/202
- Replace panic with Result in production code by @hiterm in https://github.com/hiterm/bookshelf-api/pull/204
- Link AGENTS.md to CLAUDE.md by @hiterm in https://github.com/hiterm/bookshelf-api/pull/206
- Add author update and delete functionality by @hiterm in https://github.com/hiterm/bookshelf-api/pull/207
- Add --all-targets to check and clippy CI steps by @hiterm in https://github.com/hiterm/bookshelf-api/pull/209
- Update cargo non-major by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/190
- Lock file maintenance by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/189
- Add schema.graphql and CI check by @hiterm in https://github.com/hiterm/bookshelf-api/pull/211
- chore: bump version to 2.4.0 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/203

## [2.3.3](https://github.com/hiterm/bookshelf-api/compare/2.3.2...2.3.3) - 2026-04-10

- Fix user isolation bug and add isolation tests by @hiterm in https://github.com/hiterm/bookshelf-api/pull/174
- chore: bump version to 2.3.3 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/201

## [2.3.2](https://github.com/hiterm/bookshelf-api/compare/2.3.1...2.3.2) - 2026-04-09

- Increase JWKS server startup wait retries by @hiterm in https://github.com/hiterm/bookshelf-api/pull/194
- Add TLS/HTTPS connectivity regression test for CA certificates by @hiterm in https://github.com/hiterm/bookshelf-api/pull/196
- Add JWKS caching with key rotation support by @hiterm in https://github.com/hiterm/bookshelf-api/pull/197
- chore: bump version to 2.3.2 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/195

## [2.3.1](https://github.com/hiterm/bookshelf-api/compare/2.3.0...2.3.1) - 2026-04-05

- Revert to 2.2.0 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/184
- Fix reqwest CA issue by @hiterm in https://github.com/hiterm/bookshelf-api/pull/187
- Revert "Revert to 2.2.0" by @hiterm in https://github.com/hiterm/bookshelf-api/pull/186
- chore: bump version to 2.3.1 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/185

## [2.3.0](https://github.com/hiterm/bookshelf-api/compare/2.2.0...2.3.0) - 2026-04-04

- chore(deps): lock file maintenance by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/129
- fix(deps): update rust crate mockall to 0.14.0 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/145
- Add minimumReleaseAge for cargo updates in renovate config by @hiterm in https://github.com/hiterm/bookshelf-api/pull/147
- fix(deps): update cargo non-major to v7.2.1 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/138
- Update Dockerfile to use latest tag instead of digest by @hiterm in https://github.com/hiterm/bookshelf-api/pull/152
- fix(deps): update rust crate validator to 0.20.0 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/150
- fix(deps): update rust crate axum-extra to 0.12 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/144
- chore(deps): update taiki-e/install-action action to v2.70.3 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/151
- fix(deps): update rust crate derive_more to v2 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/131
- fix(deps): update rust crate reqwest to 0.13 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/149
- Enable version pinning in Renovate config by @hiterm in https://github.com/hiterm/bookshelf-api/pull/153
- Unpin by @hiterm in https://github.com/hiterm/bookshelf-api/pull/154
- chore(deps): update taiki-e/install-action action to v2.70.4 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/155
- chore: exclude self-image and Rust dockerfile from renovate by @hiterm in https://github.com/hiterm/bookshelf-api/pull/156
- Add agent guidelines and CLAUDE.md by @hiterm in https://github.com/hiterm/bookshelf-api/pull/157
- Add unit tests for use case layer by @hiterm in https://github.com/hiterm/bookshelf-api/pull/159
- Update Rust edition from 2021 to 2024 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/160
- Update AGENTS.md by @hiterm in https://github.com/hiterm/bookshelf-api/pull/162
- Add workflow to auto-update Rust version by @hiterm in https://github.com/hiterm/bookshelf-api/pull/161
- fix: add workflows permission to update-rust workflow by @hiterm in https://github.com/hiterm/bookshelf-api/pull/163
- Remove update rust by @hiterm in https://github.com/hiterm/bookshelf-api/pull/166
- Add actionlint workflow for GitHub Actions validation by @hiterm in https://github.com/hiterm/bookshelf-api/pull/167
- Add Renovate config validation workflow and documentation by @hiterm in https://github.com/hiterm/bookshelf-api/pull/165
- Enhance GitHub Actions security and add workflow linting by @hiterm in https://github.com/hiterm/bookshelf-api/pull/169
- Read Rust toolchain version from rust-toolchain.toml by @hiterm in https://github.com/hiterm/bookshelf-api/pull/170
- Fix renovate config validation command by @hiterm in https://github.com/hiterm/bookshelf-api/pull/172
- feat(renovate): group rust-toolchain.toml and Dockerfile rust version updates by @hiterm in https://github.com/hiterm/bookshelf-api/pull/173
- Improve Renovate config validation workflow by @hiterm in https://github.com/hiterm/bookshelf-api/pull/177
- Rename fileMatch to managerFilePatterns by @hiterm in https://github.com/hiterm/bookshelf-api/pull/178
- Add Claude Code GitHub Workflow by @hiterm in https://github.com/hiterm/bookshelf-api/pull/179
- Fix Renovate not detecting rust-toolchain.toml by @hiterm in https://github.com/hiterm/bookshelf-api/pull/181
- chore(deps): update dependency rust to v1.94.1 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/175
- Replace Auth0 with self-signed JWT tokens for E2E tests by @hiterm in https://github.com/hiterm/bookshelf-api/pull/182
- chore: bump version to 2.3.0 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/146

## [2.2.0](https://github.com/hiterm/bookshelf-api/compare/2.1.2...2.2.0) - 2026-03-30

- chore: Update GitHub Actions workflows by @hiterm in https://github.com/hiterm/bookshelf-api/pull/107
- chore: Update release-drafter to v7.1.1 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/118
- chore(deps): Bump docker/login-action from 3.7.0 to 4.0.0 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/117
- chore(deps): Bump actions/cache from 4.3.0 to 5.0.4 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/116
- chore(deps): Bump docker/build-push-action from 6.19.2 to 7.0.0 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/115
- Pin GitHub Actions by SHA by @hiterm in https://github.com/hiterm/bookshelf-api/pull/119
- chore(deps): Bump reproducible-containers/buildkit-cache-dance from 63c59a3e6c6bd7fe55abecb23afd8e1121451230 to 5de31fc1534ed8789e63d41ea933c5df9944a261 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/113
- chore(deps): Bump docker/metadata-action from 5.10.0 to 6.0.0 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/112
- chore(deps): Bump stefanzweifel/git-auto-commit-action from 5.2.0 to 7.1.0 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/111
- chore(deps): Bump docker/setup-buildx-action from 3.12.0 to 4.0.0 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/110
- chore(deps): Bump actions/checkout from 4.3.1 to 6.0.2 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/108
- Downgrade version to 2.2.0 in Cargo.toml files by @hiterm in https://github.com/hiterm/bookshelf-api/pull/120
- V2.1.3 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/121
- Update dependabot config to add Cargo updates by @hiterm in https://github.com/hiterm/bookshelf-api/pull/122
- Introduce Renovate Bot by @hiterm in https://github.com/hiterm/bookshelf-api/pull/125
- chore(deps): Bump taiki-e/install-action from 2.69.14 to 2.70.0 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/123
- chore(deps): update swatinem/rust-cache digest to c193711 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/127
- chore(deps): pin dependencies by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/126
- Update Renovate config to pin versions and group updates by manager by @hiterm in https://github.com/hiterm/bookshelf-api/pull/133
- Dsiable pin temporarily by @hiterm in https://github.com/hiterm/bookshelf-api/pull/136
- chore(deps): update taiki-e/install-action action to v2.70.2 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/135
- fix(deps): update rust crate thiserror to v2 by @renovate[bot] in https://github.com/hiterm/bookshelf-api/pull/132
- Pin async-graphql version by @hiterm in https://github.com/hiterm/bookshelf-api/pull/137
- Upgrade async-graphql to 7.2.0 and axum to 0.8 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/140
- Switch to create-pull-request for automated version bumps by @hiterm in https://github.com/hiterm/bookshelf-api/pull/141
- Update create-pull-request action to use pinned commit SHA by @hiterm in https://github.com/hiterm/bookshelf-api/pull/142
- chore: bump version to 2.2.0 by @github-actions[bot] in https://github.com/hiterm/bookshelf-api/pull/143

## [2.1.2](https://github.com/hiterm/bookshelf-api/compare/2.1.1...2.1.2) - 2026-02-28

- Add health check validation to deploy workflow by @hiterm in https://github.com/hiterm/bookshelf-api/pull/106

## [2.1.1](https://github.com/hiterm/bookshelf-api/compare/2.1.0...2.1.1) - 2026-02-28

- Add devcontainer config and startup script for OpenCode Web by @hiterm in https://github.com/hiterm/bookshelf-api/pull/104
- CIでビルドしたイメージの起動確認とヘルスチェックを追加 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/105

## [2.1.0](https://github.com/hiterm/bookshelf-api/compare/2.0.20...2.1.0) - 2026-02-28

- chore(deps): Bump rsa from 0.9.7 to 0.9.10 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/93
- chore(deps): Bump tracing-subscriber from 0.3.19 to 0.3.20 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/92
- Add E2E tests with Postgres and GitHub Actions by @hiterm in https://github.com/hiterm/bookshelf-api/pull/94
- Increase server wait retries and interval in e2e tests by @hiterm in https://github.com/hiterm/bookshelf-api/pull/96
- Add /me endpoint by @hiterm in https://github.com/hiterm/bookshelf-api/pull/99
- Add e2e test cases by @hiterm in https://github.com/hiterm/bookshelf-api/pull/100
- chore(deps): Bump bytes from 1.9.0 to 1.11.1 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/98
- Update Rust to 1.93.1 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/101
- Update jsonwebtoken to 10.3.0 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/102
- Update time to 0.3.47 by @hiterm in https://github.com/hiterm/bookshelf-api/pull/103

## [2.0.20](https://github.com/hiterm/bookshelf-api/compare/2.0.19...2.0.20) - 2025-04-08

- chore(deps): Bump ring from 0.17.8 to 0.17.13 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/90
- chore(deps): Bump tokio from 1.42.0 to 1.43.1 by @dependabot[bot] in https://github.com/hiterm/bookshelf-api/pull/91

## [2.0.19](https://github.com/hiterm/bookshelf-api/compare/2.0.18...2.0.19) - 2024-12-17

- Update validator by @hiterm in https://github.com/hiterm/bookshelf-api/pull/89
