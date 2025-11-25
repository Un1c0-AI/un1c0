#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/make_repo_private.sh OWNER/REPO
# Requires `gh` CLI and that the invoking user has admin privileges on the repo.

REPO=${1:-}
if [ -z "$REPO" ]; then
  echo "Usage: $0 OWNER/REPO" >&2
  exit 2
fi

echo "This will attempt to set $REPO to private. Ensure you have admin privileges."
read -p "Proceed? [y/N] " yn
if [[ "$yn" != "y" && "$yn" != "Y" ]]; then
  echo "Aborting."; exit 1
fi

gh repo edit "$REPO" --visibility private
echo "$REPO is now private (if your account has permission)."
