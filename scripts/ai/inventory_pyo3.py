#!/usr/bin/env python3
from __future__ import annotations

import csv
import re
from pathlib import Path

ROOT = Path.cwd()
OUT = ROOT / "docs" / "rust-cutover" / "inventory" / "pyo3_export_inventory.csv"

PYMODULE_RE = re.compile(r"#\[pymodule\][\s\S]{0,400}?pub\s+fn\s+([A-Za-z0-9_]+)")
WRAP_RE = re.compile(r"wrap_pymodule!\(([^)]+)\)")
ADD_CLASS_RE = re.compile(r"add_class::<([^>]+)>")
PYCLASS_RE = re.compile(r"#\[pyclass")
PYFUNCTION_RE = re.compile(r"#\[pyfunction")


def main() -> None:
    rows = []
    for path in sorted((ROOT / "crates").rglob("*.rs")) if (ROOT / "crates").exists() else []:
        if any(part in {"target", ".git"} for part in path.parts):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        if "pyo3" not in text and "#[py" not in text and "wrap_pymodule" not in text:
            continue
        rows.append({
            "path": str(path.relative_to(ROOT)),
            "pymodules": ";".join(PYMODULE_RE.findall(text)),
            "wrapped_modules": ";".join(WRAP_RE.findall(text)),
            "add_classes": ";".join(ADD_CLASS_RE.findall(text)),
            "pyclass_count": str(len(PYCLASS_RE.findall(text))),
            "pyfunction_count": str(len(PYFUNCTION_RE.findall(text))),
        })
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=["path", "pymodules", "wrapped_modules", "add_classes", "pyclass_count", "pyfunction_count"])
        writer.writeheader()
        writer.writerows(rows)
    print(f"wrote {OUT} ({len(rows)} files)")


if __name__ == "__main__":
    main()
