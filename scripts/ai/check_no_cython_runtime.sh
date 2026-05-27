#!/usr/bin/env bash
set -euo pipefail

# This check verifies that no Cython source/build/test runtime path remains.
# It does not approve Python or PyO3 as a product surface; the stronger final
# check is scripts/ai/check_rust_only_runtime.sh.

echo "== no-cython-runtime: checking for retained .pyx/.pxd files =="
retained_sources="$({ find . \
  \( -path './.git' -o -path './.agentflow' -o -path './.venv' -o -path './venv' -o -path './target' -o -path './target-v2' -o -path './build' -o -path './dist' -o -path './.mypy_cache' -o -path './.pytest_cache' -o -path './.ruff_cache' \) -prune \
  -o \( -name '*.pyx' -o -name '*.pxd' \) -type f -print; } | sort)"
if [ -n "$retained_sources" ]; then
  echo "Retained Cython source/interface files found:" >&2
  echo "$retained_sources" >&2
  exit 1
fi

echo "== no-cython-runtime: checking active build config for Cython dependencies =="
check_paths=()
for p in pyproject.toml build.py setup.py setup.cfg Makefile Cargo.toml crates; do
  [ -e "$p" ] && check_paths+=("$p")
done

if [ ${#check_paths[@]} -gt 0 ] && grep -RInE 'Cython|cythonize|Cython\.Build|\.pyx|\.pxd' "${check_paths[@]}" 2>/dev/null; then
  echo "Potential Cython build/runtime reference found in active source paths" >&2
  exit 1
fi

echo "== no-cython-runtime: ok =="
