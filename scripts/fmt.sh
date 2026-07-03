#!/usr/bin/env bash
# Format the whole workspace in place with rustfmt.
#
# Wrapped by `just fmt`. The read-only counterpart that CI gates on is
# `just fmt-check` (scripts/fmt-check.sh). Extra arguments are forwarded to
# `cargo fmt`.
set -euo pipefail
exec cargo fmt --all "$@"
