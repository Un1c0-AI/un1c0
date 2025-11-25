Private repository migration checklist

This document lists recommended steps and a small helper script to make the repository private
and guide issue/PR migration. Many steps require admin privileges and cannot be fully automated
without additional infrastructure.

High-level checklist

1) Review repository content
   - Remove any credentials, keys, or PII that should not be committed.
   - Ensure all contributors agree to migration if required by policy.

2) Create a plan for open issues/PRs
   - Identify which open issues/PRs should be migrated or closed.
   - For issues to migrate, consider exporting them (use `gh issue list` + `gh issue view` to collect data).

3) Make the repository private (one of these options):
   - Use the GitHub UI: `Settings -> General -> Change repository visibility -> Make private`.
   - Or, use the GitHub CLI (requires admin permissions):
     ```bash
     gh repo edit OWNER/REPO --visibility private
     ```

4) Configure access and branch protection
   - Add teams or users who should have access.
   - Reconfigure branch protection rules to allow required checks and reviewers.

5) Handle forks and mirrors
   - If your org has forks, coordinate with fork owners.
   - Consider removing public mirrors and notifying stakeholders.

6) Communicate the change
   - Announce the migration window to contributors and consumers.

Helper script (local)
----------------------
The included script `scripts/make_repo_private.sh` is a convenience wrapper that uses the
`gh` CLI to flip visibility. Run it locally as a repo admin.
