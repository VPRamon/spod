#!/usr/bin/env bash
# Enforce the standalone dependency rules for spod.

set -euo pipefail

cd "$(dirname "$0")/.."

if grep -En '^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=.*\{[^}]*(path|git)[[:space:]]*=' \
  Cargo.toml; then
  echo "Dependency-graph check found a path or git dependency."
  exit 1
fi

source_null_count="$(cargo metadata --locked --format-version 1 | grep -o '"source":null' | wc -l)"
if [[ "$source_null_count" -ne 1 ]]; then
  echo "Dependency-graph check found an unexpected non-registry package."
  exit 1
fi

if ! cargo tree --locked -p spod | grep -q '^spod v'; then
  echo "Dependency-graph check FAILED."
  exit 1
fi
echo "Dependency-graph check passed."
