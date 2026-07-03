#!/usr/bin/env bash
# Build and test on the crate's Minimum Supported Rust Version.
#
# The MSRV is read from `rust-version` in the workspace Cargo.toml (currently
# 1.90), so this stays in lockstep with the declared floor — bump that field and
# this check follows. fmt/clippy are intentionally NOT run here: lint and format
# output drift between compiler versions, so they are only enforced on the pinned
# toolchain (rust-toolchain.toml, via scripts/ci.sh).
#
# Installs the MSRV toolchain if it is missing. Wrapped by `just msrv`.
set -euo pipefail

# Bind-mounts (devcontainer/CI) can trip git's "dubious ownership" guard.
git config --global --add safe.directory "$(pwd)" 2>/dev/null || true

# `rust-version` may be "1.90" or "1.90.0"; rustup wants a full x.y.z triple.
MSRV="$(sed -n 's/^rust-version *= *"\([0-9.]*\)".*/\1/p' Cargo.toml | head -n1)"
if [ -z "${MSRV}" ]; then
    echo "error: could not read rust-version from Cargo.toml" >&2
    exit 1
fi
case "${MSRV}" in
    *.*.*) : ;;              # already x.y.z
    *.*)   MSRV="${MSRV}.0" ;;
esac

echo "MSRV = ${MSRV}"
rustup toolchain install "${MSRV}" --profile minimal
cargo "+${MSRV}" build --locked --workspace --all-features
cargo "+${MSRV}" test --locked --workspace --all-features
