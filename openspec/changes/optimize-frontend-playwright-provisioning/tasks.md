## 1. Frontend browser provisioning

- [ ] 1.1 Remove the Playwright browser cache step from `test-integration-bookshelf` without adding a replacement cache.
- [ ] 1.2 Add `--only-shell` to the existing Playwright Chromium install command while preserving `--with-deps` and timing output.

## 2. Validation

- [ ] 2.1 Validate workflow syntax and run actionlint and zizmor.
- [ ] 2.2 Validate the OpenSpec change.
- [ ] 2.3 Verify the pull request CI logs show Headless Shell-only provisioning and successful frontend integration tests without a material test-time regression.
