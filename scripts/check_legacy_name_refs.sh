#!/usr/bin/env bash
# Reject obsolete project namespaces and compatibility aliases from active code
# and current documentation. A single pre-rename ADR is retained as an explicit
# historical record.

set -euo pipefail

cd "$(dirname "$0")/.."

pattern='SIDERUST_POD_REST_(BIND|OUT)|siderust[-_]pod|spod::(core|dynamics|estimation|observations|products|qc)'
matches="$(git grep -n -I -E "$pattern" -- ':!target/**' || true)"
unexpected=()

while IFS=: read -r file line text; do
    [[ -z "${file:-}" ]] && continue

    case "$file" in
        scripts/check_legacy_name_refs.sh)
            ;;
        docs/adrs/ADR-0001-workspace-reset.md)
            # Explicitly marked historical record of the pre-rename workspace.
            ;;
        *)
            unexpected+=("$file:$line:$text")
            ;;
    esac
done <<< "$matches"

if ((${#unexpected[@]} > 0)); then
    printf 'Unexpected obsolete project namespace references found:\n'
    printf '  %s\n' "${unexpected[@]}"
    exit 1
fi

echo "Legacy-name reference check passed."
