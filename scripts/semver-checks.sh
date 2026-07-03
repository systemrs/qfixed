#!/usr/bin/env bash
# Detect SemVer-incompatible public-API changes with cargo-semver-checks.
#
# By default it compares the current public API against the last version
# published on crates.io, so it is a release gate rather than part of `just ci`.
# Until the crate has a published baseline, point it at a git ref instead, e.g.
# `just semver-checks --baseline-rev master`.
#
# Requires cargo-semver-checks (installed in the devcontainer). Wrapped by
# `just semver-checks`; extra arguments are forwarded.
set -euo pipefail

if ! cargo semver-checks --version >/dev/null 2>&1; then
    echo "error: cargo-semver-checks is not installed." >&2
    echo "       install it with: cargo install --locked cargo-semver-checks" >&2
    exit 1
fi

exec cargo semver-checks "$@"
