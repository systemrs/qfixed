#!/usr/bin/env bash
# Audit the committed Cargo.lock against the RustSec advisory database with
# cargo-audit.
#
# cargo-audit reads (and never rewrites) Cargo.lock, so it always audits the
# checked-in dependency graph; it has no --locked flag of its own. Wrapped by
# `just audit` and run by scripts/ci.sh. Extra arguments are forwarded.
set -euo pipefail
exec cargo audit "$@"
