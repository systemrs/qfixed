#!/usr/bin/env bash
# Run the whole test suite in debug mode (debug_assertions on).
#
# Wrapped by `just test`. The release counterpart, which exercises the optimized
# assertion-free arithmetic path, is `just test-release` (scripts/test-release.sh).
# Extra arguments are forwarded to `cargo test`, e.g. `just test packed` to
# filter by name.
set -euo pipefail
exec cargo test --locked --workspace --all-features "$@"
