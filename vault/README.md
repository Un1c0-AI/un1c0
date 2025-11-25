Vault PoC
=========

This folder contains a small proof-of-concept to issue a `MASTER_KEY` via HashiCorp Vault.

WARNING: This PoC is for demonstration and local testing only. Do NOT use the dev server or
root tokens in production.

# Secure Vault Integration for CI/CD

Production-ready HashiCorp Vault integration with mTLS-enforced secret management for GitHub Actions workflows.

## 🎯 Overview

This demonstrates a secure, automated approach to managing secrets in CI/CD pipelines using:

- **HashiCorp Vault** - Centralized secrets management
- **mTLS Authentication** - Certificate-based security (no API keys)
- **Response Wrapping** - Single-use secret delivery
- **Automated Operations** - Certificate rotation, cleanup, key rotation
- **Production Ready** - Rate limiting, monitoring, comprehensive deployment guides

## 🚀 Quick Start

### Prerequisites

- Docker & Docker Compose
- GitHub CLI (`gh`) - For secret management
- OpenSSL - For certificate generation

### Local Setup

```bash
# 1. Start Vault stack
docker compose up -d

# 2. Initialize Vault and create certificates
./init_vault.sh
./generate_certs.sh

# 3. Configure GitHub repository secrets
./configure_secrets.sh

# 4. Test the setup
curl -k --cert certs/client.crt --key certs/client.key \
  -H "Content-Type: application/json" -d '{}' \
  https://localhost:8443/issue-wrapped-secret-id
```

### Running Workflows

```bash
# Trigger E2E workflow to provision MASTER_KEY
gh workflow run e2e_wrapped_flow.yml

# Check if MASTER_KEY was created
gh secret list | grep MASTER_KEY

# Use gated CI example as a template
gh workflow run gated_ci_example.yml
```

## 📋 Architecture

```
┌─────────────────┐
│ GitHub Actions  │
│   Workflows     │
└────────┬────────┘
         │ mTLS (client cert)
         ▼
┌─────────────────┐
│  nginx (8443)   │
│  Rate Limiting  │
│  Cert Verify    │
└────────┬────────┘
         │ HTTP (internal)
         ▼
┌─────────────────┐
│  Admin Service  │
│  Flask (5000)   │
└────────┬────────┘
         │ Vault API
         ▼
┌─────────────────┐
│  Vault (8200)   │
│  KV v2 Secrets  │
│  AppRole Auth   │
└─────────────────┘
```

## 🔐 Security Features

### mTLS Enforcement
All endpoints (except `/health`) require valid client certificates:

```bash
# ✅ With client cert - Success
curl -k --cert certs/client.crt --key certs/client.key \
  https://localhost:8443/metrics

# ❌ Without client cert - Unauthorized
curl -k https://localhost:8443/metrics
# Returns: 401 Unauthorized
```

### Rate Limiting
- **30 requests/minute** per client DN
- **10 request burst** allowance
- Returns `429 Too Many Requests` when exceeded

### Response Wrapping
All secrets delivered as single-use wrapped tokens:

```json
{
  "wrap_token": "hvs.CAESI...",
  "accessor": "kWeTvdFmXq..."
}
```

Unwrap with:
```bash
VAULT_TOKEN="$wrap_token" vault unwrap
```

## 🛠️ Endpoints

| Endpoint | Method | Purpose | Auth |
|----------|--------|---------|------|
| `/health` | GET | Health check | None |
| `/issue-secret-id` | POST | Plain secret_id | mTLS |
| `/issue-wrapped-secret-id` | POST | Wrapped secret_id | mTLS |
| `/issue-job-secret` | POST | Job-specific secret | mTLS |
| `/revoke-accessors` | POST | Revoke secrets | mTLS + Break-glass |
| `/rotate-master-key` | POST | Rotate master key | mTLS |
| `/metrics` | GET | Monitoring data | mTLS |

### Example: Issue Job-Specific Secret

```bash
curl -k --cert certs/client.crt --key certs/client.key \
  -H "Content-Type: application/json" \
  -d '{"job_id":"'$GITHUB_RUN_ID-$GITHUB_JOB'"}' \
  https://localhost:8443/issue-job-secret
```

Response:
```json
{
  "accessor": "wfURIn40SNrL0Mz2aExkB3hx",
  "job_id": "12345678-build",
  "wrap_token": "hvs.CAESI..."
}
```

## 🤖 Automated Workflows

### Core Workflows

**E2E Wrapped Flow** (`../.github/workflows/e2e_wrapped_flow.yml`)
- Provisions master secret_id for CI/CD
- Stores as `MASTER_KEY` secret
- Required by: All gated workflows

**Certificate Rotation** (`../.github/workflows/cert_rotation.yml`)
- Runs: Monthly (1st of month, 2 AM UTC)
- Checks: Certificate expiry < 30 days
- Auto-rotates: Client certificates
- Updates: `CLIENT_CERT_BASE64`, `CLIENT_KEY_BASE64` secrets

**AppRole Cleanup** (`../.github/workflows/approle_cleanup.yml`)
- Runs: Daily (3 AM UTC)
- Removes: Old secret_id accessors (keeps last 10)
- Prunes: Issuance events (keeps last 100)
- Configurable via workflow inputs

**Master Key Rotation** (`../.github/workflows/rotate_master_key.yml`)
- Runs: Quarterly (1st day, 4 AM UTC)
- Requires: Break-glass token validation
- Rotates: Master secret_id
- Expiry: Configurable (default 90 days)

### Gated CI Example

**Template** (`../.github/workflows/gated_ci_example.yml`)

Shows how to gate builds/deployments with `MASTER_KEY`:

```yaml
jobs:
  provision-master-key:
    uses: ./.github/workflows/e2e_wrapped_flow.yml

  run-tests:
    needs: provision-master-key
    steps:
      - name: Unwrap Master Key
        run: |
          SECRET_ID=$(VAULT_TOKEN="${{ secrets.MASTER_KEY }}" \
            vault unwrap -field=secret_id)
```

## 📊 Monitoring

### Metrics Endpoint

```bash
curl -k --cert certs/client.crt --key certs/client.key \
  https://localhost:8443/metrics
```

Response:
```json
{
  "accessor_count": 8,
  "issuance_event_count": 15,
  "cert_expiry_days": -1,
  "status": "ok"
}
```

**Metrics:**
- `accessor_count` - Active secret_id accessors
- `issuance_event_count` - Total issuance operations
- `cert_expiry_days` - Days until cert expires
- `status` - Service health

## 🔧 Helper Scripts

### Configuration
**`configure_secrets.sh`** - Interactive GitHub secrets setup
```bash
./configure_secrets.sh
# Sets: CLIENT_CERT_BASE64, CLIENT_KEY_BASE64, BREAK_GLASS_TOKEN, etc.
```

### Operations
**`cleanup_accessors.sh`** - Revoke old accessors and prune events
```bash
./cleanup_accessors.sh 5 50
# Keep: 5 newest accessors, 50 newest events
```

**`rotate_master_key.sh`** - Manual master key rotation
```bash
./rotate_master_key.sh 90
# Rotates master key with 90-day expiry
```

**`check_breakglass.sh`** - Validate break-glass token
```bash
./check_breakglass.sh <token>
# Returns: valid or invalid
```

### Setup
**`init_vault.sh`** - Initialize Vault with policies and AppRole
**`generate_certs.sh`** - Create local CA and client certificates

## 📖 Documentation

- **[COMPLETE_IMPLEMENTATION.md](COMPLETE_IMPLEMENTATION.md)** - Full implementation summary
- **[PRODUCTION_DEPLOYMENT.md](PRODUCTION_DEPLOYMENT.md)** - Production deployment guide
  - Kubernetes (Helm) setup
  - VM/Docker deployment
  - Auto-unseal (AWS KMS, Azure, GCP)
  - HA configuration with Raft
  - Monitoring & alerting
  - Backup & disaster recovery
- **[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)** - Original PoC status

## 🔒 Break-Glass Procedures

### Emergency Access

When automated workflows fail or `MASTER_KEY` is compromised:

1. **Retrieve break-glass token** (stored offline during setup)
2. **Revoke all accessors:**
   ```bash
   curl -k --cert certs/client.crt --key certs/client.key \
     -H "Content-Type: application/json" \
     -H "X-Break-Glass-Token: <token>" \
     -d '{"accessor_ids":["*"]}' \
     https://localhost:8443/revoke-accessors
   ```
3. **Rotate master key:**
   ```bash
   gh workflow run rotate_master_key.yml \
     -f break_glass_token=<token>
   ```
4. **Audit logs** for unauthorized access
5. **Rotate break-glass token** after use

## 🚢 Production Deployment

### Quick Checklist

- [ ] Review `PRODUCTION_DEPLOYMENT.md`
- [ ] Deploy HA Vault cluster (3+ nodes)
- [ ] Configure auto-unseal (AWS KMS recommended)
- [ ] Use organization PKI for certificates
- [ ] Set up monitoring (Prometheus/Grafana)
- [ ] Configure audit logging to SIEM
- [ ] Enable backup automation
- [ ] Document break-glass procedures
- [ ] Test disaster recovery process
- [ ] Set up alerting for cert expiry
- [ ] Configure rate limiting per environment
- [ ] Review security hardening checklist

### Kubernetes Deployment

```bash
# Install Vault with Helm
helm repo add hashicorp https://helm.releases.hashicorp.com
helm install vault hashicorp/vault -f k8s/values.yaml
```

See `PRODUCTION_DEPLOYMENT.md` for complete guide.

## 🧪 Testing

### Local Testing

```bash
# Start services
docker compose up -d

# Initialize Vault
./init_vault.sh

# Test wrapped secret issuance
curl -k --cert certs/client.crt --key certs/client.key \
  -H "Content-Type: application/json" -d '{}' \
  https://localhost:8443/issue-wrapped-secret-id

# Test metrics
curl -k --cert certs/client.crt --key certs/client.key \
  https://localhost:8443/metrics

# Test rate limiting (should get 429 after 30 requests)
for i in {1..35}; do
  curl -k -w "%{http_code}\n" --cert certs/client.crt --key certs/client.key \
    https://localhost:8443/health
done
```

### Workflow Testing

```bash
# Test E2E flow
gh workflow run e2e_wrapped_flow.yml
gh run watch

# Test cleanup
gh workflow run approle_cleanup.yml -f max_accessors=5 -f max_events=20

# Test cert rotation (manual)
gh workflow run cert_rotation.yml
```

## 📁 Directory Structure

```
vault/
├── admin_service/         # Flask admin service
│   ├── app.py
│   ├── Dockerfile
│   └── requirements.txt
├── nginx/                 # mTLS termination
│   ├── mutual_tls.conf
│   └── Dockerfile
├── policies/              # Vault policies
│   └── approle_reader.hcl
├── certs/                 # Local certificates (gitignored)
├── scripts/               # Helper scripts
│   ├── init_vault.sh
│   ├── generate_certs.sh
│   ├── cleanup_accessors.sh
│   ├── configure_secrets.sh
│   ├── rotate_master_key.sh
│   └── check_breakglass.sh
├── docker-compose.yml
├── PRODUCTION_DEPLOYMENT.md
├── COMPLETE_IMPLEMENTATION.md
├── IMPLEMENTATION_STATUS.md
└── README.md (this file)
```

## 🆘 Troubleshooting

### Common Issues

**"401 Unauthorized" on all requests**
- Check client certificate is provided
- Verify cert is signed by correct CA
- Check nginx logs: `docker compose logs nginx`

**"429 Too Many Requests"**
- Rate limit exceeded (30 req/min)
- Wait 1 minute or adjust limit in `nginx/mutual_tls.conf`

**"Vault sealed" errors**
- Restart Vault: `docker compose restart vault`
- Wait 10 seconds for auto-unseal
- Check Vault status: `vault status`

**Workflows fail with "MASTER_KEY not found"**
- Run E2E workflow first: `gh workflow run e2e_wrapped_flow.yml`
- Verify secret exists: `gh secret list | grep MASTER_KEY`

### Debug Commands

```bash
# Check Vault status
vault status

# List secret_id accessors
vault list auth/approle/role/ci-cd-role/secret-id

# View admin service logs
docker compose logs -f admin-service

# View nginx logs
docker compose logs -f nginx

# Test mTLS from inside container
docker compose exec admin-service curl -k \
  --cert /app/certs/client.crt --key /app/certs/client.key \
  https://nginx:8443/health
```

## 🔗 Resources

- [HashiCorp Vault Documentation](https://www.vaultproject.io/docs)
- [Vault AppRole Auth](https://www.vaultproject.io/docs/auth/approle)
- [Response Wrapping](https://www.vaultproject.io/docs/concepts/response-wrapping)
- [GitHub Actions Security](https://docs.github.com/en/actions/security-guides)

---

**Status:** Production Ready ✅  
**Last Updated:** November 25, 2025
