#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
workflow="${script_dir}/../workflows/deploy.yml"

assert_contains() {
  local expected=$1

  if ! grep --fixed-strings --quiet -- "${expected}" "${workflow}"; then
    echo "Expected deploy workflow to contain: ${expected}" >&2
    exit 1
  fi
}

assert_contains "id: meta"
assert_contains 'uses: docker/metadata-action@'
assert_contains 'images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}'
assert_contains 'tags: type=raw,value=${{ inputs.release_tag }}'
assert_contains 'labels: ${{ steps.meta.outputs.labels }}'
