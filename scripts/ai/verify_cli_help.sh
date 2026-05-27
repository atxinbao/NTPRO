#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

export PATH="/opt/homebrew/opt/rustup/bin:${PATH}"

run_cli() {
  local label="$1"
  shift
  echo "== verify_cli_help: ${label} =="
  cargo run -q -p nautilus-cli -- "$@"
}

run_cli "top-level help" --help
run_cli "version" --version
run_cli "database help" database --help

echo "== verify_cli_help complete =="
