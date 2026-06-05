#!/bin/bash

uv sync --all-groups --no-install-project
uv run --no-sync pytest tests/performance_tests --benchmark-disable-gc --codspeed
