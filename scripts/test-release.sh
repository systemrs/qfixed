#!/usr/bin/env bash
# Run the whole test suite in release mode.
#
# debug_assertions are off here, so this exercises the optimized, assertion-free
# arithmetic path. Wrapped by `just test-release` and run by scripts/ci.sh. Extra
# arguments are forwarded to `cargo test`.
set -euo pipefail
exec cargo test --locked --workspace --all-features --release "$@"
