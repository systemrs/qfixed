#!/usr/bin/env bash
#
# Runs once, the first time the QFixed devcontainer is created.
# Keep everything here idempotent and non-fatal — a failed optional step should
# not block the container from coming up.
set -euo pipefail

WORKSPACE="/workspaces/qfixed"

echo "==> QFixed devcontainer: post-create"

# Devcontainers frequently see the bind-mounted repo as "dubious ownership".
git config --global --add safe.directory "${WORKSPACE}" || true

# QFixed vendors the shared Rust skill as a git submodule
# (.claude/skills/claude-skill-rust). Make sure it is checked out.
if [ -f "${WORKSPACE}/.gitmodules" ]; then
    echo "==> Initialising git submodules"
    git -C "${WORKSPACE}" submodule update --init --recursive || true
fi

# Ensure the components the Rust skill's checks need are present (the image
# installs them, but a toolchain update could drop them).
rustup component add rustfmt clippy >/dev/null 2>&1 || true

# The cargo registry/git caches are backed by named volumes (see
# devcontainer.json). A named volume's ownership is fixed when it is first
# created and is NOT re-derived from the image on later rebuilds, so the
# Dockerfile's build-time chown only covers a first-ever launch. Reconcile it
# here (the dev user has passwordless sudo) so cargo can always write the caches
# even after a UID remap or a volume reused from an older image.
for cache in /opt/cargo/registry /opt/cargo/git; do
    if [ -d "${cache}" ] && [ "$(stat -c %U "${cache}")" != "$(id -un)" ]; then
        echo "==> Reconciling ownership of ${cache}"
        sudo chown -R "$(id -un):$(id -gn)" "${cache}"
    fi
done

# Warm the crate cache once a Cargo manifest is present at the repo root.
if [ -f "${WORKSPACE}/Cargo.toml" ]; then
    echo "==> Fetching crate dependencies (cargo fetch)"
    cargo fetch --manifest-path "${WORKSPACE}/Cargo.toml" || true
else
    echo "==> No workspace Cargo.toml at the repo root yet — skipping cargo fetch."
    echo "    The crates live under crates/; add a root Cargo.toml with a"
    echo "    [workspace] table to tie them together."
fi

echo "==> post-create complete"
