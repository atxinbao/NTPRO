#!/usr/bin/env bash
set -euo pipefail

# V080-005 synthetic secret leak scanner.
# Scans generated stdout/stderr, artifacts, dashboard snapshots, logs, and
# evidence snippets for synthetic Binance testnet credential markers. This
# script never uses real credentials and never opens a network connection.

if (( $# == 0 )); then
  echo "usage: $0 <path> [<path> ...]" >&2
  exit 2
fi

SYNTHETIC_API_KEY="${NTPRO_V08_SYNTHETIC_API_KEY:-FAKE_BINANCE_TESTNET_API_KEY_SHOULD_NOT_APPEAR}"
SYNTHETIC_API_SECRET="${NTPRO_V08_SYNTHETIC_API_SECRET:-FAKE_BINANCE_TESTNET_API_SECRET_SHOULD_NOT_APPEAR}"
SYNTHETIC_SIGNATURE="${NTPRO_V08_SYNTHETIC_SIGNATURE:-FAKE_BINANCE_SIGNATURE_SHOULD_NOT_APPEAR}"

scan_roots() {
  python3 - "$SYNTHETIC_API_KEY" "$SYNTHETIC_API_SECRET" "$SYNTHETIC_SIGNATURE" "$@" <<'PY'
import sys
from pathlib import Path

tokens = [token for token in sys.argv[1:4] if token]
roots = [Path(path) for path in sys.argv[4:]]
skip_dirs = {".git", "target", "target-v2", "dist", "build"}
matches = []
files_scanned = 0


def iter_files(root: Path):
    if root.is_file():
        yield root
        return
    if not root.exists():
        raise SystemExit(f"scan root does not exist: {root}")
    for path in root.rglob("*"):
        if any(part in skip_dirs for part in path.parts):
            continue
        if path.is_file():
            yield path


for root in roots:
    for path in iter_files(root):
        files_scanned += 1
        try:
            text = path.read_text(errors="replace")
        except OSError as exc:
            raise SystemExit(f"failed to read {path}: {exc}") from exc
        for token in tokens:
            if token in text:
                matches.append((str(path), token))

if matches:
    print("v08_synthetic_secret_leak_scan status=fail", file=sys.stderr)
    for path, token in matches:
        print(f"leaked synthetic secret token in {path}: {token}", file=sys.stderr)
    raise SystemExit(1)

print(
    "v08_synthetic_secret_leak_scan status=ok "
    f"files_scanned={files_scanned} tokens_scanned={len(tokens)}"
)
PY
}

if [[ "${NTPRO_V08_SECRET_SCANNER_SELF_TEST:-1}" == "1" ]]; then
  self_test_root="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v08-secret-scanner-self-test.XXXXXX")"
  printf 'leaked=%s\n' "$SYNTHETIC_API_KEY" >"$self_test_root/leak.txt"
  if NTPRO_V08_SECRET_SCANNER_SELF_TEST=0 scan_roots "$self_test_root" >/dev/null 2>&1; then
    echo "v08 synthetic secret scanner self-test failed to catch a leak" >&2
    exit 1
  fi
  rm -rf "$self_test_root"
  echo "v08_synthetic_secret_leak_scan self_test=ok"
fi

scan_roots "$@"
