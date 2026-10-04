#!/usr/bin/env bash
# Check that legacy project names are limited to documented history and
# explicitly supported REST environment aliases.

set -euo pipefail

cd "$(dirname "$0")/.."

matches="$(git grep -n -I -E 'siderust-pod|siderust_pod|SIDERUST_POD' -- ':!target/**' || true)"
unexpected=()

while IFS=: read -r file line text; do
    [[ -z "${file:-}" ]] && continue

    case "$file" in
        CHANGELOG.md|docs/adr/*|docs/adrs/*|docs/design/*)
            ;;
        scripts/check_legacy_name_refs.sh)
            ;;
        README.md|src/bin/spod-rest.rs|.github/workflows/ci.yml)
            if [[ "$text" != *SIDERUST_POD_REST_BIND* &&
                  "$text" != *SIDERUST_POD_REST_OUT* ]]; then
                unexpected+=("$file:$line:$text")
            fi
            ;;
        *)
            unexpected+=("$file:$line:$text")
            ;;
    esac
done <<< "$matches"

if ((${#unexpected[@]} > 0)); then
    printf 'Unexpected legacy project-name references found:\n'
    printf '  %s\n' "${unexpected[@]}"
    exit 1
fi

echo "Legacy-name reference check passed."
