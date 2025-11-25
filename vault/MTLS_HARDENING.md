mTLS hardening and client-cert rotation (PoC guidance)
=====================================================

This document explains options to harden the admin-service with mutual TLS and rotate
client certificates safely.

Overview
--------
- Use nginx (or another TLS terminator) to require client certificates for connections
  to the admin service. The nginx config is in `vault/nginx/mutual_tls.conf`.
- Use a PKI (Vault's PKI secrets engine or an internal CA) to issue short-lived client
  certificates for CI runners or operators. Avoid long-lived client certs.

Client cert issuance (recommended)
---------------------------------
1) Enable Vault PKI (example):

```bash
vault secrets enable pki
vault secrets tune -max-lease-ttl=87600h pki
vault write pki/root/generate/internal common_name="Internal CA" ttl=87600h
vault write pki/config/urls issuing_certificates="$VAULT_ADDR/v1/pki/ca" crl_distribution_points="$VAULT_ADDR/v1/pki/crl"
```

2) Create a role to issue client certs with short TTL (e.g., 1h):

```bash
vault write pki/roles/ci-client allow_any_name=false allowed_domains=example.com allow_subdomains=false max_ttl=1h
```

3) For each GitHub Actions run (or runner), request a signed client cert dynamically:

```bash
vault write pki/issue/ci-client common_name="ci-runner-$(date +%s)" ttl=1h
# Response contains certificate and private_key
```

4) Use the returned cert/key pair for the runner to connect to the admin service mTLS endpoint.

Automating rotation
-------------------
- Issue per-run short TTL certs instead of baking a cert into runner images.
- Store the cert thumbprint or accessor in Vault KV to track issuance and revoke if needed.

Break-glass and master-user recovery
-----------------------------------
- Keep an offline, securely stored admin token or hardware-backed key you can use to recover
  access if AppRole or client-cert flows are misconfigured. Store it in a secure vault (not in
  GitHub secrets). Document the recovery process and limit who can use it.

NGINX configuration notes
------------------------
- `ssl_verify_client on;` forces clients to present certificates. Combine with `ssl_client_certificate`
  pointing at the correct CA bundle.
- Use `proxy_set_header X-SSL-CLIENT-S-DN $ssl_client_s_dn;` to forward the client identity to the upstream
  service for logging or authorization decisions.

Security considerations
-----------------------
- Only accept certificates issued by your authorized PKI.
- Monitor issuance events in Vault and admin-service logs.
- Revoke compromised certificates via Vault's revoke capabilities using stored accessors.
