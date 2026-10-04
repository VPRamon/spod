#!/usr/bin/env bash
# Regression tests for check_architecture.sh using temporary source trees.

set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
checker="$script_dir/check_architecture.sh"
root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

mkdir -p "$root/src/service" "$root/src/sgp4" "$root/tests/sgp4" "$root/scripts"
cp "$checker" "$root/scripts/check_architecture.sh"
chmod +x "$root/scripts/check_architecture.sh"

expect_failure() {
  local expected="$1"
  if SPOD_ARCH_ROOT="$root" "$root/scripts/check_architecture.sh" \
      >"$root/stdout" 2>"$root/stderr"; then
    echo "architecture regression expected failure: $expected" >&2
    exit 1
  fi
  grep -q "$expected" "$root/stderr"
}

cat >"$root/src/service/forbidden.rs" <<'EOF'
struct ForceModel;
pub struct MeasurementModel;
pub(crate) struct Integrator;
pub(super) trait Propagator {}
pub(self) struct KalmanFilter;
pub(in crate::foo) struct GaussNewton;
struct LeastSquares;
pub struct OrbitComparator;
EOF
expect_failure "New local generic scientific implementations"
rm "$root/src/service/forbidden.rs"

cat >"$root/src/sgp4/exception.rs" <<'EOF'
pub(crate) struct ForceModel;
pub(super) trait Propagator {}
EOF
cat >"$root/tests/sgp4/other.rs" <<'EOF'
pub struct ForceModel;
EOF
expect_failure "New local generic scientific implementations"
rm "$root/tests/sgp4/other.rs"
SPOD_ARCH_ROOT="$root" "$root/scripts/check_architecture.sh"

mock_bin="$root/mock-bin"
mkdir "$mock_bin"
cat >"$mock_bin/grep" <<'EOF'
#!/usr/bin/env bash
exit 2
EOF
chmod +x "$mock_bin/grep"
if PATH="$mock_bin:$PATH" SPOD_ARCH_ROOT="$root" \
    "$root/scripts/check_architecture.sh" >"$root/stdout" 2>"$root/stderr"; then
  echo "architecture regression expected checker error to fail" >&2
  exit 1
fi
grep -q "grep exited with status 2" "$root/stderr"

echo "Architecture guard regression tests passed."
