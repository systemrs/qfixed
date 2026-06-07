#!/usr/bin/env bash
#
# Runs every time a session attaches to the QFixed devcontainer.
# Keep this fast and side-effect-free — it is on the interactive attach path.
set -euo pipefail

echo "QFixed dev environment"
echo "  rustc  : $(rustc --version 2>/dev/null || echo 'not found')"
echo "  cargo  : $(cargo --version 2>/dev/null || echo 'not found')"
echo "  clippy : $(cargo-clippy --version 2>/dev/null || echo 'not found')"
