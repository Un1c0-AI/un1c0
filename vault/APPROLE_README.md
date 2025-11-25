AppRole PoC
===========

This document explains how the AppRole PoC works.

1) `vault/init_vault.sh` now creates an AppRole named `master-key-approle` and prints
   the `ROLE_ID` and `SECRET_ID` values. For local PoC you can copy these values and store
   them as GitHub repository secrets `VAULT_ROLE_ID` and `VAULT_SECRET_ID` (NOT recommended
   for production; prefer AppRole secret_id generation per-request).

2) The workflow `.github/workflows/issue_master_key_vault_approle.yml` demonstrates how to
    authenticate to Vault using AppRole credentials and read or create `MASTER_KEY` in Vault
    and then write it into the repository secrets using a maintainer PAT (`KEY_ADMIN_TOKEN`).

3) Admin service integration (mTLS-only recommended)
    - Instead of storing a `VAULT_SECRET_ID` repository secret, prefer using the admin service
       to issue a new `secret_id` per run. For security, the admin service must be protected by mTLS
       and not by a static API key. Configure the admin service and set the secrets or expose it
       via a secure endpoint reachable by GitHub Actions.
    - The workflow will request a wrapped `secret_id` from the admin service at runtime and then
       unwrap it via Vault. The admin service records `secret_id` accessors and can revoke old
       accessors when rotating keys.

Security notes
--------------
- In production, do NOT store long-lived `SECRET_ID` values in repository secrets. Instead,
  generate secret_ids dynamically for each request and deliver them securely to the requester.
- Use AppRole with constrained policies and short TTL tokens. Consider AppRole wrapped tokens
  or an intermediate issuance service that provides ephemeral secret_ids.
