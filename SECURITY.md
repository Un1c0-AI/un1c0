## Security and access to repository build/run

This repository uses a repository-level secret called `MASTER_KEY` which is required
to run CI and build/test flows. The goal is to restrict automated CI/build runs
from unauthorized forks and to maintain control over who can execute the repository's
protected workflows.

Important notes:
- GitHub does not allow a repository to prevent a public fork from being created.
  A fork can be created by anyone with GitHub access when the repository is public.
- What we can do: require the presence of a repository secret (`MASTER_KEY`) for CI
  and require passing status checks before merging. Secrets are not provided to
  workflows triggered from forks, so CI will fail on forked PRs unless the maintainer
  explicitly provides the key.

How to request a `MASTER_KEY`:
1. Open a private issue in this repository titled `MASTER_KEY request: <your-name>`.
2. Explain the reason for your request and provide contact information.
3. A repository maintainer will review and, if approved, provide a short-lived key
   and instruct you how to use it for authorized CI runs.

Maintainers: store the approved `MASTER_KEY` as a repository secret at
`Settings -> Secrets -> Actions -> New repository secret` with name `MASTER_KEY`.

Enforcement and merging
- Configure branch protection rules to require the CI workflow (`build-test`) as a
  required status check. Because secrets are not available to workflows from forks,
  this ensures PRs from forks cannot satisfy required status checks until a maintainer
  approves and performs the build/merge as appropriate.

Dynamic issuance (recommended)
------------------------------
We recommend using an admin issuance service or Vault AppRole dynamic secret_id issuance instead
of storing `VAULT_SECRET_ID` as a repository secret. This repo includes a PoC admin service and
Vault AppRole workflows that allow issuing a one-time `secret_id` per workflow run and revoking
old secret_ids during key rotation. See `vault/` for the PoC artifacts and `vault/APPROLE_README.md`
for details.

If you need a different access model (e.g., programmatic issuance of keys), contact a maintainer
and we can discuss building a webhook or small admin service to automate key issuance.
