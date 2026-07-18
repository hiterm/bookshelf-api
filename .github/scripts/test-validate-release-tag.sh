#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
validator="${script_dir}/validate-release-tag.sh"

assert_valid() {
  local tag=$1

  if ! "${validator}" "${tag}"; then
    echo "Expected a valid release tag: ${tag}" >&2
    exit 1
  fi
}

assert_invalid() {
  local tag=$1

  if "${validator}" "${tag}" >/dev/null 2>&1; then
    echo "Expected an invalid release tag: ${tag}" >&2
    exit 1
  fi
}

assert_valid "1.2.3"
assert_valid "1.2.3-rc.1"

assert_invalid ""
assert_invalid "v1.2.3"
assert_invalid "1.2.3+build"
assert_invalid "1.2.3-rc.1+build"
