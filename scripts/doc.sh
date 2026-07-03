#!/usr/bin/env bash
# Build the API docs with warnings treated as errors (broken intra-doc links and
# the like fail the build), matching the CI documentation gate.
#
# Wrapped by `just doc` and run by scripts/ci.sh. Extra arguments are forwarded
# to `cargo doc`, e.g. `just doc --open`.
set -euo pipefail
exec env RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps --all-features "$@"
