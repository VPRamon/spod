#!/usr/bin/env bash
# Check that legacy project names are limited to documented history and
# explicitly supported REST environment aliases.

set -euo pipefail

cd "$(dirname "$0")/.."

matches="$(git grep -n -I -E 'siderust-pod|siderust_pod|SIDERUST_POD' -- ':!target/**' || true)"
unexpected=()
historical_files=(
    CHANGELOG.md
    docs/adrs/ADR-0001-workspace-reset.md
    docs/adr/0002-no-anyhow.md
    docs/adr/0005-lisa-poc-scope.md
    docs/adr/0006-finite-diff-stm-allow.md
    docs/adrs/ADR-0002-faer-backend.md
    docs/adrs/ADR-0004-provider-traits.md
    docs/design/plan.md
    docs/design/research-requirements-tests-cases.md
    docs/design/spod_detailed_design_document.md
)

is_historical_file() {
    local candidate
    for candidate in "${historical_files[@]}"; do
        [[ "$1" == "$candidate" ]] && return 0
    done
    return 1
}

is_allowed_historical_command() {
    [[ "$1" == "docs/design/research-requirements-tests-cases.md" &&
       "$2" =~ ^cargo[[:space:]]+(test|bench)[[:space:]]+-p[[:space:]]+siderust-pod- ]]
}

while IFS=: read -r file line text; do
    [[ -z "${file:-}" ]] && continue

    case "$file" in
        *)
            if is_historical_file "$file"; then
                if [[ "$text" =~ (github\.com|git[[:space:]]+clone|cargo[[:space:]]+(run|build|test|install)|docker[[:space:]]+(build|run)) ]] &&
                   ! is_allowed_historical_command "$file" "$text"; then
                    unexpected+=("$file:$line:$text")
                fi
            elif [[ "$file" == "scripts/check_legacy_name_refs.sh" ]]; then
                :
            elif [[ "$file" == "README.md" || "$file" == "src/bin/spod-rest.rs" || "$file" == ".github/workflows/ci.yml" ]]; then
                if [[ "$text" != *SIDERUST_POD_REST_BIND* &&
                      "$text" != *SIDERUST_POD_REST_OUT* ]]; then
                    unexpected+=("$file:$line:$text")
                fi
            else
                unexpected+=("$file:$line:$text")
            fi
            ;;
    esac
done <<< "$matches"

if ((${#unexpected[@]} > 0)); then
    printf 'Unexpected legacy project-name references found:\n'
    printf '  %s\n' "${unexpected[@]}"
    exit 1
fi

echo "Legacy-name reference check passed."
