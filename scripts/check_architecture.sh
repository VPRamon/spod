#!/usr/bin/env bash
# Enforce the service -> Siderust dependency direction.
#
# This is deliberately a source-pattern guard, not a filename blacklist:
# service orchestration may mention or compose upstream scientific types.
# New local generic scientific implementations must either move upstream or
# be added to a documented exception. The only current exception is src/sgp4.

set -euo pipefail

repository_root="${SPOD_ARCH_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$repository_root"

exception_dir="src/sgp4"
if [[ ! -d "$exception_dir" ]]; then
  echo "Architecture check FAILED: expected SGP4 exception directory '$exception_dir'." >&2
  exit 1
fi

mapfile -t source_files < <(
  find src tests -type f -name '*.rs' ! -path "$exception_dir/*" -print
)

if grep -InE \
    '^[[:space:]]*(pub[[:space:]]+)?(struct|trait)[[:space:]]+(ForceModel|MeasurementModel|Propagator|Integrator|KalmanFilter|GaussNewton|LeastSquares|OrbitComparator)([[:space:]]|<|{|$)' \
    "${source_files[@]}"; then
  grep_status=0
else
  grep_status=$?
fi

case "$grep_status" in
  0)
    cat >&2 <<'EOF'
Architecture source-hygiene check FAILED.
New local generic scientific implementations were found outside the
documented SGP4 exception in src/sgp4/. Reuse Siderust APIs or document a
temporary upstream-gap exception before adding local science.
EOF
    exit 1
    ;;
  1)
      echo "Architecture source-hygiene check passed."
      exit 0
    ;;
  *)
    echo "Architecture source-hygiene check FAILED: grep exited with status $grep_status." >&2
    exit "$grep_status"
    ;;
esac
