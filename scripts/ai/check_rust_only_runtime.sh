#!/usr/bin/env bash
set -euo pipefail

echo "== rust-only-runtime: checking product surface =="

fail=0

for path in python nautilus_trader crates/pyo3 build.py; do
  if [ -e "$path" ]; then
    echo "Rust-only release must not retain product path: $path" >&2
    fail=1
  fi
done

if find crates -path '*/src/python' -type d -print 2>/dev/null | grep -q .; then
  echo "Rust-only release must not retain crates/*/src/python modules:" >&2
  find crates -path '*/src/python' -type d -print >&2
  fail=1
fi

if find . \
  -path './.git' -prune -o \
  -path './.agentflow' -prune -o \
  -path './target' -prune -o \
  -path './target-v2' -prune -o \
  -path './dist' -prune -o \
  -path './build' -prune -o \
  -type f \( -name '*.pyx' -o -name '*.pxd' \) -print | grep -q .; then
  echo "Rust-only release must not retain Cython .pyx/.pxd files:" >&2
  find . \
    -path './.git' -prune -o \
    -path './.agentflow' -prune -o \
    -path './target' -prune -o \
    -path './target-v2' -prune -o \
    -path './dist' -prune -o \
    -path './build' -prune -o \
    -type f \( -name '*.pyx' -o -name '*.pxd' \) -print >&2
  fail=1
fi

active_paths=()
for p in Cargo.toml Cargo.lock crates Makefile pyproject.toml setup.py setup.cfg; do
  [ -e "$p" ] && active_paths+=("$p")
done

if [ "${#active_paths[@]}" -gt 0 ]; then
  if grep -RInE 'pyo3|maturin|Cython|cythonize|\.pyx|\.pxd' "${active_paths[@]}" 2>/dev/null; then
    echo "Rust-only release must not retain Python/PyO3/Cython build/runtime references in active product paths" >&2
    fail=1
  fi
fi

if ! cargo metadata --no-deps --format-version=1 | grep -q '"name":"nautilus-cli"'; then
  echo "Rust-only release requires a nautilus-cli package" >&2
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "== rust-only-runtime: ok =="
