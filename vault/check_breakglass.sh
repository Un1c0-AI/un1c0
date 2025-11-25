#!/usr/bin/env bash
set -euo pipefail

# Check that the break-glass admin token can still read the master key.
# Usage: BREAK_GLASS_TOKEN=<token> VAULT_ADDR=http://127.0.0.1:8200 ./vault/check_breakglass.sh

VAULT_ADDR=${VAULT_ADDR:-http://127.0.0.1:8200}
TOKEN=${BREAK_GLASS_TOKEN:-}

if [ -z "$TOKEN" ]; then
  echo "Provide BREAK_GLASS_TOKEN in environment" >&2
  exit 2
fi

RESP=$(curl -sS -H "X-Vault-Token: $TOKEN" "$VAULT_ADDR/v1/kv/data/master_key" || true)
KEY=$(printf '%s' "$RESP" | jq -r '.data.data.key // empty') || true
if [ -z "$KEY" ]; then
  echo "Break-glass token failed to read master key. Response:" >&2
  printf '%s
'"$RESP"\n' >&2
  exit 1
fi

echo "Break-glass token validated: can read master key (not printing key)."
exit 0
