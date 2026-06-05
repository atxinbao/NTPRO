#!/bin/bash

uv sync --all-groups --no-install-project
uv run --no-sync pytest --ignore=tests/performance_tests --new-first --failed-first
