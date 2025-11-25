#!/usr/bin/env bash
set -euo pipefail

# Usage: generate_master_key.sh [REPO] [EXPIRY_DAYS]
# Example: REPO=Un1c0-AI/un1c0 EXPIRY_DAYS=1 ./generate_master_key.sh

REPO=${1:-${REPO:-}}
EXPIRY_DAYS=${2:-1}

if [ -z "${KEY_ADMIN_TOKEN:-}" ]; then
  echo "ERROR: set KEY_ADMIN_TOKEN environment variable to a PAT with repo admin scope." >&2
  exit 2
fi

if [ -z "$REPO" ]; then
  echo "ERROR: repository argument missing. Provide owner/repo as first arg or set REPO env var." >&2
  exit 2
fi

KEY=$(openssl rand -hex 32)
EXPIRY=$(date -u -d "+${EXPIRY_DAYS} days" +"%Y-%m-%dT%H:%M:%SZ")

echo "Generating MASTER_KEY for repo=$REPO (expiry: $EXPIRY)"

# Authenticate gh CLI using the provided token on stdin
printf '%s' "$KEY_ADMIN_TOKEN" | gh auth login --with-token

# Set the repository secret MASTER_KEY to the generated key
# Note: this will overwrite any existing MASTER_KEY secret on the target repo
gh secret set MASTER_KEY --body "$KEY" --repo "$REPO"

echo "MASTER_KEY secret set in $REPO."
echo "IMPORTANT: The raw key is NOT printed by this script for security reasons." >&2
echo "Deliver the key securely to the authorized recipient (out-of-band) and record its expiry: $EXPIRY" >&2

exit 0
