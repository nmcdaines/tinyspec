#!/usr/bin/env bash
set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
ln -sf "${REPO_ROOT}/scripts/pre-commit" "${REPO_ROOT}/.git/hooks/pre-commit"
echo "Pre-commit hook installed."
