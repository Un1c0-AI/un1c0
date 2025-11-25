#!/usr/bin/env bash
#set -euo pipefail
# Requests a short-lived client certificate from Vault PKI and writes cert/key to files.
# Usage: VAULT_ADDR and VAULT_TOKEN must be set. Outputs: client.crt and client.key

VAULT_ADDR=${VAULT_ADDR:-http://127.0.0.1:8200}
VAULT_TOKEN=${VAULT_TOKEN:-root-token}
ROLE=${1:-ci-client}
OUT_DIR=${2:-./vault/certs}
CN=${3:-ci-runner-$(date +%s)}

if [ -z "$VAULT_ADDR" ] || [ -z "$VAULT_TOKEN" ]; then
  echo "Set VAULT_ADDR and VAULT_TOKEN in environment" >&2
  exit 2
fi

mkdir -p "$OUT_DIR"

RESP=$(curl -sS -X POST -H "X-Vault-Token: $VAULT_TOKEN" -H "Content-Type: application/json" \
  -d "{\"common_name\": \"$CN\", \"ttl\": \"1h\"}" "$VAULT_ADDR/v1/pki/issue/$ROLE")

CERT=$(echo "$RESP" | jq -r '.data.certificate')
KEY=$(echo "$RESP" | jq -r '.data.private_key')

if [ -z "$CERT" ] || [ -z "$KEY" ]; then
  echo "Failed to issue client cert: $RESP" >&2
  exit 1
fi

echo "$CERT" > "$OUT_DIR/client.crt"
echo "$KEY" > "$OUT_DIR/client.key"
chmod 600 "$OUT_DIR/client.key"

echo "Wrote client.crt and client.key to $OUT_DIR"
