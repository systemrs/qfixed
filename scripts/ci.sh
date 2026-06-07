#!/usr/bin/env bash
#
# Full verification suite for qfixed.
#
# This is the single source of truth for "what CI runs". The GitHub Actions
# workflow (.github/workflows/ci.yml) builds the dev container from
# .devcontainer/Dockerfile and runs THIS script inside it, so CI and local
# development execute exactly the same checks against the same pinned toolchain
# (rust-toolchain.toml). Run it locally any time with:
#
#     bash scripts/ci.sh
#
# It mirrors the build-verification sequence from the claude-skill-rust skill,
# plus the cargo-deny / cargo-audit supply-chain gates.
set -euo pipefail

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
cargo fmt --all -- --check
endgroup

# Lint the DEFAULT feature set (serde off) — this is the configuration most
# downstream users get, so it must compile cleanly on its own.
group "clippy (default features)"
cargo clippy --locked --workspace --all-targets -- -D warnings
endgroup

group "clippy (all features, all targets)"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
endgroup

group "test (all features, debug)"
cargo test --locked --workspace --all-features
endgroup

# Re-run tests in release mode: debug_assertions are off, so this exercises the
# optimized, assertion-free arithmetic path.
group "test (all features, release)"
cargo test --locked --workspace --all-features --release
endgroup

group "build (release)"
cargo build --locked --workspace --all-features --release
endgroup

# The crate's strict lints (deny clippy::pedantic / missing_docs) are gated to
# release, non-test builds, so this pass is what actually enforces them.
group "clippy (release, enforces crate's deny(pedantic))"
cargo clippy --locked --workspace --all-targets --all-features --release -- -D warnings
endgroup

group "doc (warnings as errors)"
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps --all-features
endgroup

group "cargo-deny (advisories, licenses, bans, sources)"
cargo deny check
endgroup

# cargo-audit reads (and never rewrites) the committed Cargo.lock, so it always
# audits the checked-in dependency graph. It has no --locked flag of its own.
group "cargo-audit (RustSec advisories)"
cargo audit
endgroup

echo "All checks passed."
