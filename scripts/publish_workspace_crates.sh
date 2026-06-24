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
#   circuits                      -> (no local deps)
#   circuits-stark-verifier       -> circuits
#   circuit-common                -> circuits, circuits-stark-verifier
#   circuit-verifier              -> circuits, circuits-stark-verifier, circuit-common
#   circuit-serialize             -> circuits, circuits-stark-verifier
#   circuit-prover                -> circuits, circuit-verifier, circuit-common, circuits-stark-verifier
#   circuit-cairo-verifier        -> circuits, circuit-common, circuits-stark-verifier
#   circuit-multiverifier         -> circuits, circuits-stark-verifier, circuit-verifier, circuit-common
#   circuit-cairo-serialize       -> circuit-verifier
#
# Not published (publish = false): circuits-stark-verifier-examples (examples/test utilities).
CRATES_TO_PUBLISH=(
  circuits
  circuits-stark-verifier
  circuit-common
  circuit-verifier
  circuit-serialize
  circuit-prover
  circuit-cairo-verifier
  circuit-multiverifier
  circuit-cairo-serialize
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
