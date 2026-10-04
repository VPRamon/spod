#!/usr/bin/env bash
# Enforce the service -> Siderust dependency direction.
#
# This is deliberately a source-pattern guard, not a filename blacklist:
# service orchestration may mention or compose upstream scientific types.
# New local generic scientific implementations must either move upstream or
# be added to a documented exception. The only current exception is src/sgp4.

set -euo pipefail

cd "$(dirname "$0")/.."

exception_dir="src/sgp4"
if [[ ! -d "$exception_dir" ]]; then
  echo "Architecture check FAILED: expected SGP4 exception directory '$exception_dir'." >&2
  exit 1
fi

if ! grep -RInE \
  --include='*.rs' \
  --exclude-dir=target \
  --exclude-dir="$exception_dir" \
  '^[[:space:]]*(pub[[:space:]]+)?(struct|trait)[[:space:]]+(ForceModel|MeasurementModel|Propagator|Integrator|KalmanFilter|GaussNewton|LeastSquares|OrbitComparator)([[:space:]]|<|{|$)' \
  src tests; then
  echo "Architecture source-hygiene check passed."
  exit 0
fi

cat >&2 <<'EOF'
Architecture source-hygiene check FAILED.
New local generic scientific implementations were found outside the
documented SGP4 exception in src/sgp4/. Reuse Siderust APIs or document a
temporary upstream-gap exception before adding local science.
EOF
exit 1
