Admin Service (PoC)
====================

This small Flask service provides endpoints to:
- issue AppRole `secret_id` for a Vault role (`/issue-secret-id`)
- revoke old secret_id accessors and keep only the latest (`/revoke-accessors`)
- rotate the `MASTER_KEY` in Vault and revoke old accessors (`/rotate-master-key`)

Security
--------
- The service is protected by a shared `ADMIN_API_KEY` environment variable for PoC.
- In production replace this with mTLS, OAuth, or another strong auth method.

Running locally
---------------
1. Start Vault and this service using docker compose (root of repo):

```bash
VAULT_TOKEN=root-token VAULT_ADDR=http://127.0.0.1:8200 docker compose -f vault/docker-compose.yml up --build
```

2. Authenticate to the admin service via mTLS (recommended)

The admin service enforces mTLS through the TLS terminator (nginx in the PoC). For local testing, generate
short-lived client certificates (see `vault/issue_client_cert.sh`) and use them when calling the admin API. The
admin service no longer accepts API-key authentication; you must present a client certificate trusted by the CA
configured in `vault/nginx/mutual_tls.conf`.

Example request (wrapped secret issuance):

```bash
curl --cert vault/certs/client.crt --key vault/certs/client.key -k -X POST -H "Content-Type: application/json" \
  -d '{"role":"master-key-approle","wrap_ttl":"60s"}' https://localhost:8443/issue-wrapped-secret-id
```

Rotate master key (use client cert):

```bash
curl --cert vault/certs/client.crt --key vault/certs/client.key -k -X POST https://localhost:8443/rotate-master-key
```
