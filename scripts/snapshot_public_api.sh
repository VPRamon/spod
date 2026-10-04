#!/usr/bin/env bash
# Generate or compare the public API snapshot for the single spod crate.
#
#   scripts/snapshot_public_api.sh --update   # refresh api.snapshot
#   scripts/snapshot_public_api.sh            # compare when a baseline exists
#
# Requires cargo-public-api and a nightly toolchain.

set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo-public-api >/dev/null 2>&1; then
  echo "cargo-public-api not installed; skipping snapshot." >&2
  echo "Install with: cargo install --locked cargo-public-api" >&2
  exit 0
fi

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

cargo public-api --simplified > "$tmp"

if [[ "${1-}" == "--update" ]]; then
  mv "$tmp" api.snapshot
  trap - EXIT
  echo "updated api.snapshot"
  exit 0
fi

if [[ ! -f api.snapshot ]]; then
  echo "api.snapshot does not exist yet; run with --update to create a baseline."
  exit 0
fi

if ! diff -u api.snapshot "$tmp"; then
  echo
  echo "Public API drift detected. Re-run with --update after reviewing the change."
  exit 1
fi

echo "Public API snapshot is up to date."
