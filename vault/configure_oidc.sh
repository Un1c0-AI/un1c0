#!/usr/bin/env bash
set -euo pipefail

# This script shows example commands to configure Vault to accept GitHub OIDC tokens.
# This is illustrative — you must supply correct values for your Vault and GitHub OIDC setup.

VAULT_ADDR=${VAULT_ADDR:-http://127.0.0.1:8200}
VAULT_TOKEN=${VAULT_TOKEN:-root-token}

GH_OIDC_ISSUER='https://token.actions.githubusercontent.com'
GH_AUDIENCE='api://GitHubActions'
VAULT_ROLE_NAME='github-oidc-role'

echo "Enabling JWT auth method at auth/jwt"
curl -sS -X POST -H "X-Vault-Token: $VAULT_TOKEN" -d '{"type":"jwt"}' $VAULT_ADDR/v1/sys/auth/jwt || true

echo "Configuring JWT/OIDC provider for GitHub Actions"
curl -sS -X POST -H "X-Vault-Token: $VAULT_TOKEN" -H "Content-Type: application/json" \
  -d "{\"jwt_validation_pubkeys\":[], \"oidc_discovery_url\": \"$GH_OIDC_ISSUER\"}" \
  $VAULT_ADDR/v1/auth/jwt/config || true

echo "Creating role that maps GitHub OIDC claims to policies"
cat > /tmp/role.json <<EOF
{
  "role_type": "jwt",
  "bound_audiences": ["$GH_AUDIENCE"],
  "user_claim": "sub",
  "policies": ["default", "read-master-key"],
  "claim_mappings": {}
}
EOF

curl -sS -X POST -H "X-Vault-Token: $VAULT_TOKEN" -H "Content-Type: application/json" -d @/tmp/role.json $VAULT_ADDR/v1/auth/jwt/role/$VAULT_ROLE_NAME || true

echo "Vault is configured (PoC). To use GitHub OIDC, modify the GH workflow to request an OIDC token and use it to login to Vault's JWT auth method."
