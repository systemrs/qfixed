#!/usr/bin/env bash
# Check that commit subjects follow Conventional Commits, so release automation
# (e.g. release-plz) can derive the version bump. A release gate — not part of
# `just ci`.
#
# Validates every non-merge commit on the current branch that is not on the base
# branch. BASE defaults to `origin/master` if that ref exists, else `master`;
# override it explicitly, e.g. `BASE=main just commit-lint`. On a branch with no
# commits beyond the base (e.g. master itself) there is nothing to check.
#
# This is a self-contained regex check — no external commit-lint tool required.
set -euo pipefail

BASE="${BASE:-}"
if [ -z "${BASE}" ]; then
    if git rev-parse --verify --quiet origin/master >/dev/null; then
        BASE="origin/master"
    else
        BASE="master"
    fi
fi

# Conventional Commits: <type>(<optional scope>)<optional !>: <description>
type_re='(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)'
subject_re="^${type_re}(\([a-z0-9,._/ -]+\))?!?: .+"

range="${BASE}..HEAD"

fail=0
found=0
while IFS= read -r sha; do
    [ -z "${sha}" ] && continue
    found=1
    subject="$(git log -1 --format=%s "${sha}")"
    # Skip autosquash markers; they collapse into their target on rebase.
    case "${subject}" in
        fixup!*|squash!*) continue ;;
    esac
    if [[ "${subject}" =~ $subject_re ]]; then
        echo "✓ ${sha:0:12}  ${subject}"
    else
        echo "✗ ${sha:0:12}  ${subject}"
        fail=1
    fi
done < <(git rev-list --no-merges "${range}")

if [ "${found}" -eq 0 ]; then
    echo "No commits to lint in range ${range}."
    exit 0
fi

if [ "${fail}" -ne 0 ]; then
    {
        echo
        echo "Some commit subjects do not follow Conventional Commits:"
        echo "  <type>(<optional scope>): <description>"
        echo "  e.g. 'feat(q): add checked_mul' or 'fix: correct rounding overflow'"
    } >&2
    exit 1
fi

echo "All commit subjects follow Conventional Commits."
