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

expect_forbidden_declaration() {
  local description="$1"
  local declaration="$2"

  printf '%s\n' "$declaration" >"$root/src/service/forbidden.rs"
  if SPOD_ARCH_ROOT="$root" "$root/scripts/check_architecture.sh" \
      >"$root/stdout" 2>"$root/stderr"; then
    echo "architecture regression expected failure: $description" >&2
    exit 1
  fi
  grep -q "New local generic scientific implementations" "$root/stderr"
  rm "$root/src/service/forbidden.rs"
}

for case in \
  "private|struct ForceModel;" \
  "public|pub struct MeasurementModel;" \
  "pub(crate)|pub(crate) struct Integrator;" \
  "pub(super)|pub(super) trait Propagator {}" \
  "pub(self)|pub(self) struct KalmanFilter;" \
  "pub(in path)|pub(in crate::foo) struct GaussNewton;" \
  "private second name|struct LeastSquares;" \
  "public second name|pub struct OrbitComparator;"
do
  IFS='|' read -r description declaration <<<"$case"
  expect_forbidden_declaration "$description" "$declaration"
done

cat >"$root/src/sgp4/exception.rs" <<'EOF'
pub(crate) struct ForceModel;
pub(super) trait Propagator {}
EOF
cat >"$root/tests/sgp4/other.rs" <<'EOF'
pub struct ForceModel;
EOF
if SPOD_ARCH_ROOT="$root" "$root/scripts/check_architecture.sh" \
    >"$root/stdout" 2>"$root/stderr"; then
  echo "architecture regression expected tests/sgp4 declaration to fail" >&2
  exit 1
fi
grep -q "New local generic scientific implementations" "$root/stderr"
rm "$root/tests/sgp4/other.rs"
SPOD_ARCH_ROOT="$root" "$root/scripts/check_architecture.sh" \
  >"$root/stdout" 2>"$root/stderr"
grep -qx "Architecture source-hygiene check passed." "$root/stdout"
rm "$root/src/sgp4/exception.rs"

mock_bin="$root/mock-bin"
mkdir "$mock_bin"
cat >"$mock_bin/grep" <<'EOF'
#!/usr/bin/env bash
echo "grep must not run without inspectable Rust files" >&2
exit 99
EOF
chmod +x "$mock_bin/grep"
PATH="$mock_bin:$PATH" SPOD_ARCH_ROOT="$root" \
  "$root/scripts/check_architecture.sh" >"$root/stdout" 2>"$root/stderr"
grep -qx "Architecture source-hygiene check passed." "$root/stdout"

cat >"$root/src/service/valid.rs" <<'EOF'
pub(crate) struct ServiceState;
EOF
SPOD_ARCH_ROOT="$root" "$root/scripts/check_architecture.sh" \
  >"$root/stdout" 2>"$root/stderr"
grep -qx "Architecture source-hygiene check passed." "$root/stdout"

cat >"$mock_bin/grep" <<'EOF'
#!/usr/bin/env bash
exit 2
EOF
if PATH="$mock_bin:$PATH" SPOD_ARCH_ROOT="$root" \
    "$root/scripts/check_architecture.sh" >"$root/stdout" 2>"$root/stderr"; then
  echo "architecture regression expected checker error to fail" >&2
  exit 1
fi
grep -q "grep exited with status 2" "$root/stderr"

echo "Architecture guard regression tests passed."
