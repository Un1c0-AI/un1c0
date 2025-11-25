#!/usr/bin/env bash
set -euo pipefail

# Rotate the master key stored in Vault KV v2 at kv/data/master_key
# Usage: VAULT_ADDR and VAULT_TOKEN must be set in the environment (or exported prior to running)

VAULT_ADDR=${VAULT_ADDR:-http://127.0.0.1:8200}
VAULT_TOKEN=${VAULT_TOKEN:-root-token}

if [ -z "${VAULT_ADDR:-}" ] || [ -z "${VAULT_TOKEN:-}" ]; then
  echo "ERROR: VAULT_ADDR and VAULT_TOKEN must be set in the environment" >&2
  exit 2
fi

export VAULT_ADDR VAULT_TOKEN

NEW_KEY=$(openssl rand -hex 32)
EXPIRY=$(date -u -d "+1 day" +"%Y-%m-%dT%H:%M:%SZ")

echo "Rotating MASTER_KEY in Vault (expiry: $EXPIRY)"
if [ "${SKIP_BREAKGLASS_CHECK:-false}" != "true" ]; then
  if [ -z "${BREAK_GLASS_TOKEN:-}" ]; then
    echo "ERROR: BREAK_GLASS_TOKEN not provided. Set BREAK_GLASS_TOKEN or set SKIP_BREAKGLASS_CHECK=true to bypass (not recommended)." >&2
    exit 2
  fi
  echo "Validating break-glass token before rotating..."
  ./vault/check_breakglass.sh
fi

curl -sS -X POST -H "X-Vault-Token: $VAULT_TOKEN" -H "Content-Type: application/json" \
  -d "{\"data\":{\"key\":\"$NEW_KEY\",\"expiry\":\"$EXPIRY\"}}" \
  $VAULT_ADDR/v1/kv/data/master_key

echo "MASTER_KEY rotated. New key is printed below for local PoC — do NOT expose in production logs."
echo "$NEW_KEY"
