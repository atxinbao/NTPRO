#!/usr/bin/env bash
set -euo pipefail

# Resolve Cargo.toml relative to this script's location.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_TOML_FILE="${SCRIPT_DIR}/../Cargo.toml"

# Check that Cargo.toml exists.
if [[ ! -f "$CARGO_TOML_FILE" ]]; then
  echo "Error: Cargo.toml not found at $CARGO_TOML_FILE" >&2
  exit 1
fi

# Detect available Python interpreter (honor PYTHON env var, then probe common names)
detect_python() {
  if [[ -n "${PYTHON:-}" ]] && command -v "$PYTHON" &> /dev/null; then
    echo "$PYTHON"
  elif command -v python3 &> /dev/null; then
    echo "python3"
  elif command -v python &> /dev/null; then
    echo "python"
  elif command -v py &> /dev/null; then
    # Windows py launcher
    echo "py -3"
  else
    return 1
  fi
}

# Try to parse using Python's `tomllib` (Python 3.11+) if available
PYTHON_CMD=$(detect_python 2> /dev/null) || PYTHON_CMD=""

if [[ -n "$PYTHON_CMD" ]] && $PYTHON_CMD -c "import tomllib" &> /dev/null; then
  VERSION=$($PYTHON_CMD -c "
import tomllib
with open('$CARGO_TOML_FILE', 'rb') as f:
    data = tomllib.load(f)
print(data['workspace']['package']['version'])
" | tr -d '\n\r ')
else
  # Fallback: extract the version only from the [workspace.package] table.
  VERSION=$(awk '
    /^\[workspace\.package\]/ { in_section=1; next }
    /^\[/ { in_section=0 }
    in_section && /^version[[:space:]]*=/ {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' "$CARGO_TOML_FILE" | tr -d '\n\r ')
fi

# Validate that we got a version
if [[ -z "$VERSION" ]]; then
  echo "Error: Could not extract workspace package version from $CARGO_TOML_FILE" >&2
  exit 1
fi

# Output version (without trailing newline for consistency)
echo -n "$VERSION"
