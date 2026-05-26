#!/usr/bin/env python3
from __future__ import annotations

import csv
import re
from pathlib import Path

ROOT = Path.cwd()
OUT = ROOT / "docs" / "rust-cutover" / "inventory" / "cython_inventory.csv"
PATTERNS = ["*.pyx", "*.pxd"]

IMPORT_RE = re.compile(r"^\s*(?:from\s+([\w\.]+)\s+c?import|c?import\s+([\w\.]+))", re.M)


def main() -> None:
    rows = []
    for pattern in PATTERNS:
        for path in sorted(ROOT.rglob(pattern)):
            if any(part in {".venv", "target", "build", ".git"} for part in path.parts):
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            imports = sorted({m.group(1) or m.group(2) for m in IMPORT_RE.finditer(text)})
            rows.append({
                "path": str(path.relative_to(ROOT)),
                "kind": path.suffix,
                "lines": str(text.count("\n") + 1),
                "imports": ";".join(imports),
                "has_cdef_class": str("cdef class" in text),
                "has_cpdef": str("cpdef" in text),
                "has_cimport": str("cimport" in text),
            })
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=["path", "kind", "lines", "imports", "has_cdef_class", "has_cpdef", "has_cimport"])
        writer.writeheader()
        writer.writerows(rows)
    print(f"wrote {OUT} ({len(rows)} files)")


if __name__ == "__main__":
    main()
