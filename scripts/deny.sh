#!/usr/bin/env bash
# Check dependency licenses, security advisories, duplicate versions, and crate
# sources with cargo-deny (configuration in deny.toml).
#
# Wrapped by `just deny` and run by scripts/ci.sh. Extra arguments are forwarded
# to `cargo deny`; with none it runs `cargo deny check`.
set -euo pipefail
exec cargo deny "${@:-check}"
