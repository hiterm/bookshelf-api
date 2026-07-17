#!/usr/bin/env bash

set -euo pipefail

tag=${1:-}

if [[ ! "${tag}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "::error::Release tag is not a Docker-compatible semantic version: ${tag}" >&2
  exit 1
fi
