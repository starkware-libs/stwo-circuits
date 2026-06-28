#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Publish workspace crates in dependency order.

Usage:
  scripts/publish_workspace_crates.sh --order-only
  scripts/publish_workspace_crates.sh --dry-run
  scripts/publish_workspace_crates.sh --skip-first 2
  scripts/publish_workspace_crates.sh -- --registry crates-io
EOF
}

dry_run=0
order_only=0
skip_first=0
publish_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      dry_run=1
      shift
      ;;
    --order-only)
      order_only=1
      shift
      ;;
    --skip-first|-s)
      if [[ -z "${2:-}" ]]; then
        echo "Error: --skip-first requires a numeric argument." >&2
        exit 1
      fi
      skip_first="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    --)
      shift
      publish_args+=("$@")
      break
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="${SCRIPT_DIR}/.."

# Hard-coded publish order. Keep in sync with workspace crates.
# Crates are listed in topological dependency order: every crate appears
# after all of the workspace crates it depends on.
#   stwo-circuits-core            -> (no local deps)
#   stwo-circuits-stark-verifier  -> stwo-circuits-core
#   stwo-circuit-common           -> stwo-circuits-core, stwo-circuits-stark-verifier
#   stwo-circuit-verifier         -> stwo-circuits-core, stwo-circuits-stark-verifier, stwo-circuit-common
#   stwo-circuit-serialize        -> stwo-circuits-core, stwo-circuits-stark-verifier
#   stwo-circuit-prover           -> stwo-circuits-core, stwo-circuit-verifier, stwo-circuit-common, stwo-circuits-stark-verifier
#   stwo-circuit-cairo-verifier   -> stwo-circuits-core, stwo-circuit-common, stwo-circuits-stark-verifier
#   stwo-circuit-multiverifier    -> stwo-circuits-core, stwo-circuits-stark-verifier, stwo-circuit-verifier, stwo-circuit-common
#   stwo-circuit-cairo-serialize  -> stwo-circuit-verifier
#
# Not published (publish = false): stwo-circuits-stark-verifier-examples (examples/test utilities).
CRATES_TO_PUBLISH=(
  stwo-circuits-core
  stwo-circuits-stark-verifier
  stwo-circuit-common
  stwo-circuit-verifier
  stwo-circuit-serialize
  stwo-circuit-prover
  stwo-circuit-cairo-verifier
  stwo-circuit-multiverifier
  stwo-circuit-cairo-serialize
)

echo "Publish order:"
for name in "${CRATES_TO_PUBLISH[@]}"; do
  echo "- ${name}"
done

if [[ "${order_only}" -eq 1 ]]; then
  exit 0
fi

for name in "${CRATES_TO_PUBLISH[@]:${skip_first}}"; do
  echo ""
  echo "==> Publishing ${name}"
  cmd=(cargo publish -p "${name}")
  if [[ "${dry_run}" -eq 1 ]]; then
    cmd+=(--dry-run)
  fi
  if [[ "${#publish_args[@]}" -gt 0 ]]; then
    cmd+=("${publish_args[@]}")
  fi
  (cd "${WORKSPACE_ROOT}" && "${cmd[@]}")
done
