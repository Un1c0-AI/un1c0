#!/bin/bash
set -e

# GitHub Secrets Configuration Helper
# Easily configure required secrets from local environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERTS_DIR="$SCRIPT_DIR/certs"

echo "GitHub Secrets Configuration Helper"
echo "===================================="
echo ""

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed"
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "Error: Not authenticated with GitHub CLI"
    echo "Run: gh auth login"
    exit 1
fi

echo "Authenticated with GitHub CLI ✅"
echo ""

# Option to generate new certificates
read -p "Generate new client certificates? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Generating certificates..."
    "$SCRIPT_DIR/generate_certs.sh"
    echo "✅ Certificates generated"
    echo ""
fi

# Check if certificates exist
if [ ! -f "$CERTS_DIR/client.crt" ] || [ ! -f "$CERTS_DIR/client.key" ]; then
    echo "Error: Client certificates not found"
    echo "Run ./generate_certs.sh first or answer 'y' when prompted"
    exit 1
fi

echo "Setting repository secrets..."
echo ""

# 1. CLIENT_CERT_BASE64
echo "1. Setting CLIENT_CERT_BASE64..."
CLIENT_CERT_B64=$(base64 -w0 "$CERTS_DIR/client.crt")
gh secret set CLIENT_CERT_BASE64 --body "$CLIENT_CERT_B64"
echo "   ✅ CLIENT_CERT_BASE64 set"

# 2. CLIENT_KEY_BASE64
echo "2. Setting CLIENT_KEY_BASE64..."
CLIENT_KEY_B64=$(base64 -w0 "$CERTS_DIR/client.key")
gh secret set CLIENT_KEY_BASE64 --body "$CLIENT_KEY_B64"
echo "   ✅ CLIENT_KEY_BASE64 set"

# 3. VAULT_ROLE_ID (optional - from init_vault.sh output)
echo ""
read -p "Set VAULT_ROLE_ID? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Run ./init_vault.sh and copy the ROLE_ID value"
    read -p "Enter VAULT_ROLE_ID: " ROLE_ID
    if [ -n "$ROLE_ID" ]; then
        gh secret set VAULT_ROLE_ID --body "$ROLE_ID"
        echo "   ✅ VAULT_ROLE_ID set"
    fi
fi

# 4. BREAK_GLASS_TOKEN (optional - for master key rotation)
echo ""
read -p "Set BREAK_GLASS_TOKEN for master key rotation? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Generate a secure break-glass token (store it offline!)"
    BREAK_GLASS=$(openssl rand -hex 32)
    echo "Generated token: $BREAK_GLASS"
    echo ""
    echo "⚠️  IMPORTANT: Save this token securely offline (encrypted USB, printed backup)"
    echo "   This token is required for master key rotation and revocation operations"
    echo ""
    read -p "Press Enter to set this token in GitHub secrets..."
    gh secret set BREAK_GLASS_TOKEN --body "$BREAK_GLASS"
    echo "   ✅ BREAK_GLASS_TOKEN set"
    
    # Save to local file (should be gitignored)
    echo "$BREAK_GLASS" > "$SCRIPT_DIR/.break_glass_token"
    chmod 600 "$SCRIPT_DIR/.break_glass_token"
    echo "   📄 Token also saved to vault/.break_glass_token (gitignored)"
fi

# 5. KEY_ADMIN_TOKEN (optional - GitHub PAT for setting MASTER_KEY)
echo ""
read -p "Set KEY_ADMIN_TOKEN (GitHub PAT with secrets:write)? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Create a GitHub Personal Access Token with 'secrets:write' permission"
    echo "Go to: https://github.com/settings/tokens/new"
    echo ""
    read -p "Enter GitHub PAT: " -s KEY_ADMIN_TOKEN
    echo ""
    if [ -n "$KEY_ADMIN_TOKEN" ]; then
        gh secret set KEY_ADMIN_TOKEN --body "$KEY_ADMIN_TOKEN"
        echo "   ✅ KEY_ADMIN_TOKEN set"
    fi
fi

echo ""
echo "✅ Configuration complete!"
echo ""
echo "Summary of configured secrets:"
gh secret list

echo ""
echo "Next steps:"
echo "1. Run the E2E workflow to test: gh workflow run e2e_wrapped_flow.yml"
echo "2. View workflow results: gh run list --workflow=e2e_wrapped_flow.yml"
echo "3. Check that MASTER_KEY was set: gh secret list | grep MASTER_KEY"
echo ""
echo "Optional workflows:"
echo "- Certificate rotation: gh workflow run cert_rotation.yml -f force_rotation=true"
echo "- AppRole cleanup: gh workflow run approle_cleanup.yml"
echo "- Master key rotation: gh workflow run rotate_master_key.yml -f break_glass_token=<token>"
