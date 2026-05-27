#!/usr/bin/env bash
set -euo pipefail

echo "== cython-removed: checking repository =="

# Final-release check. This is stronger than check_no_cython_runtime.sh.
# It verifies that v1 Cython source/build artifacts are not retained.

if find . \
  -path './.git' -prune -o \
  -path './.venv' -prune -o \
  -path './target' -prune -o \
  -path './dist' -prune -o \
  -type f \( -name '*.pyx' -o -name '*.pxd' \) -print | grep -q .; then
  echo "Cython .pyx/.pxd files remain:" >&2
  find . \
    -path './.git' -prune -o \
    -path './.venv' -prune -o \
    -path './target' -prune -o \
    -path './dist' -prune -o \
    -type f \( -name '*.pyx' -o -name '*.pxd' \) -print >&2
  exit 1
fi

# Active build/runtime paths must not depend on Cython. Documentation and
# migration notes may mention Cython, so this intentionally avoids docs/ and
# the check scripts themselves.
paths_to_check=(pyproject.toml build.py setup.py setup.cfg Makefile python nautilus_trader)
existing=()
for p in "${paths_to_check[@]}"; do
  [ -e "$p" ] && existing+=("$p")
done

if [ "${#existing[@]}" -gt 0 ]; then
  if grep -RInE 'Cython|cythonize|Cython\.Build|\.pyx|\.pxd' "${existing[@]}" 2>/dev/null; then
    echo "Cython build/runtime references remain in active paths" >&2
    exit 1
  fi
fi

echo "== cython-removed: ok =="
