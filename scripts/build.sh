#!/usr/bin/env bash
# Compile the whole workspace and all targets in debug mode — a quick "does it
# still build" check.
#
# Wrapped by `just build`. The release build that CI gates on is
# `just build-release` (scripts/build-release.sh). Extra arguments are forwarded
# to `cargo build`.
set -euo pipefail
exec cargo build --locked --workspace --all-targets --all-features "$@"
