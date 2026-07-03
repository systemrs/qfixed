#!/usr/bin/env bash
# Lint the workspace with clippy; warnings are errors.
#
# Runs the same three passes as CI, because the crate gates its strict lints
# (deny(clippy::pedantic), deny(missing_docs)) to release, non-test builds
# (see crates/qfixed/src/lib.rs), so each feature/profile combination has to be
# linted on its own:
#   1. default features (serde off) — the configuration most downstream users get
#   2. all features                 — exercises the optional serde code paths
#   3. release, all features        — the pass that actually enforces deny(pedantic)
#
# Wrapped by `just clippy`. Extra arguments are forwarded to every pass (before
# the `--` separator), e.g. `just clippy -p qfixed` or `just clippy --fix`.
set -euo pipefail

cargo clippy --locked --workspace --all-targets "$@" -- -D warnings
cargo clippy --locked --workspace --all-targets --all-features "$@" -- -D warnings
cargo clippy --locked --workspace --all-targets --all-features --release "$@" -- -D warnings
