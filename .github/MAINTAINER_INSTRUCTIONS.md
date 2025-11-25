Maintainer instructions: Enforcing protected CI and limiting forked CI runs

1) Add the `MASTER_KEY` repository secret
   - Go to `Settings -> Secrets and variables -> Actions -> New repository secret`
   - Name: `MASTER_KEY`
   - Value: the generated passcode/key you will issue to authorized parties

2) Configure branch protection for `main`
   - Settings -> Branches -> Add rule (branch name: `main`)
   - Require status checks to pass before merging
   - Select the CI check `build-test` (the workflow in `.github/workflows/ci.yml`)
   - Optional: Require branches to be up to date before merging

3) Disable forking at org level (optional, requires organization admin)
   - Organization settings -> Policies -> Repository Forking
   - Set policy to prevent forking for selected repositories or disable forking globally

4) Handle PRs from forks
   - Because secrets are not provided to workflows from forks, the `build-test` job will fail
     for forked PRs. Maintain a workflow for maintainers to run CI manually or merge after
     verifying code locally.

5) Optional automated key issuance
   - If you want a programmatic process to issue `MASTER_KEY` values, set up a small
     internal service that: validates user requests, generates short-lived keys, and
     communicates them securely to the requester. Do NOT store long-lived keys in source.
