#!/usr/bin/env bash
#
# Full verification suite for qfixed.
#
# This is the single source of truth for "what CI runs". The GitHub Actions
# workflow (.github/workflows/ci.yml) builds the dev container from
# .devcontainer/Dockerfile and runs `just ci` (which runs THIS script) inside it,
# so CI and local development execute exactly the same checks against the same
# pinned toolchain (rust-toolchain.toml). Run it locally any time with:
#
#     just ci          # or, without just installed: bash scripts/ci.sh
#
# Each step below delegates to the same scripts/<step>.sh that the matching
# `just` recipe wraps, so `just fmt-check`, `just clippy`, `just test`, ... each
# run byte for byte what this suite runs. It mirrors the build-verification
# sequence from the claude-skill-rust skill, plus the cargo-deny / cargo-audit
# supply-chain gates.
set -euo pipefail

# Resolve the directory this script lives in, so the per-step scripts are found
# no matter what the caller's working directory is.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# `::group::`/`::endgroup::` fold the output into collapsible sections on GitHub
# Actions; locally they are just printed as plain lines.
group() { echo "::group::$1"; }
endgroup() { echo "::endgroup::"; }

# Devcontainer and CI bind-mounts often trip git's "dubious ownership" guard
# (the checkout is owned by a different uid than the container user).
git config --global --add safe.directory "$(pwd)" 2>/dev/null || true

echo "Toolchain:"
rustc --version
cargo --version
cargo clippy --version

group "rustfmt (check)"
"${here}/fmt-check.sh"
endgroup

# Lints the default feature set, the all-features set, and the release pass that
# enforces the crate's deny(pedantic)/missing_docs — see scripts/clippy.sh.
group "clippy (default + all-features + release)"
"${here}/clippy.sh"
endgroup

group "test (all features, debug)"
"${here}/test.sh"
endgroup

group "test (all features, release)"
"${here}/test-release.sh"
endgroup

group "build (release)"
"${here}/build-release.sh"
endgroup

group "doc (warnings as errors)"
"${here}/doc.sh"
endgroup

group "cargo-deny (advisories, licenses, bans, sources)"
"${here}/deny.sh"
endgroup

group "cargo-audit (RustSec advisories)"
"${here}/audit.sh"
endgroup

echo "All checks passed."
