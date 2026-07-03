#!/usr/bin/env bash
# Check formatting without modifying files — the CI formatting gate.
#
# Wrapped by `just fmt-check` and run first by scripts/ci.sh. Extra arguments are
# forwarded to rustfmt (after the `--` separator).
set -euo pipefail
exec cargo fmt --all -- --check "$@"
