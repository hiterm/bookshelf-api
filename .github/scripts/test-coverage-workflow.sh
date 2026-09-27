#!/usr/bin/env bash

set -euo pipefail

workflow=.github/workflows/ci.yml

assert_contains() {
  local expected=$1

  if ! grep -F --quiet -- "${expected}" "${workflow}"; then
    echo "Expected CI workflow to contain: ${expected}" >&2
    exit 1
  fi
}

assert_absent() {
  local unexpected=$1

  if grep -F --quiet -- "${unexpected}" "${workflow}"; then
    echo "Expected CI workflow not to contain: ${unexpected}" >&2
    exit 1
  fi
}

assert_contains "tool: cargo-llvm-cov@0.9.0"
assert_contains "cargo llvm-cov --all-features --locked --lcov --output-path lcov.info"
assert_contains "uses: codecov/codecov-action@303a32d7a59b442fa8d48b6a1cc6825c09c847a5 # v7.1.1"
assert_contains "id-token: write"
assert_contains "use_oidc: true"
assert_contains "files: lcov.info"
assert_contains "disable_search: true"
assert_contains "fail_ci_if_error: true"

assert_absent "cargo llvm-cov --workspace"
assert_absent "--fail-under"
assert_absent "upload-artifact"
assert_absent "--text"
assert_absent "--html"
assert_absent "e2e"
assert_absent "CODECOV_TOKEN"
assert_absent "handle_no_reports_found: true"
