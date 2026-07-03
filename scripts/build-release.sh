#!/usr/bin/env bash
# Compile the whole workspace in release mode — the optimized artifact CI gates
# on.
#
# Wrapped by `just build-release` and run by scripts/ci.sh. Extra arguments are
# forwarded to `cargo build`.
set -euo pipefail
exec cargo build --locked --workspace --all-features --release "$@"
