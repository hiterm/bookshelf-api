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
assert_contains "cargo llvm-cov --all-features --locked --show-missing-lines"
assert_contains "tee coverage.txt"
assert_contains "<details>"
assert_contains 'cat coverage.txt'
assert_contains 'GITHUB_STEP_SUMMARY'

assert_absent "cargo llvm-cov --workspace"
assert_absent "--fail-under"
assert_absent "codecov"
assert_absent "upload-artifact"
assert_absent "--text"
assert_absent "--html"
