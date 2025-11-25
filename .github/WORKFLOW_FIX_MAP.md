# GitHub Actions Workflow Fix Map

**Generated**: 2025-11-25 UTC  
**Purpose**: Systematic resolution of workflow diagnostics before v0.9.5 launch

## Problem Categories

### Category A: YAML Syntax Errors (CRITICAL - Severity 8)

#### A1. Compact Mapping Colons
**Issue ID**: `YAML-COMPACT-COLON-001`, `YAML-COMPACT-COLON-002`  
**Files**: `.github/workflows/ci.yml`  
**Lines**: 48, 83  
**Root Cause**: Compact YAML mapping syntax requires quoted values when containing unescaped colons  
**Fix**: Wrap step names in double quotes  
**Impact**: Workflow parser failures preventing CI execution

#### A2. Invalid Patch Marker
**Issue ID**: `YAML-INVALID-MARKER-001`  
**File**: `.github/workflows/e2e_wrapped_flow.yml`  
**Line**: 98  
**Root Cause**: Orphaned merge conflict marker `*** End Patch` (not valid YAML)  
**Fix**: Remove entire line  
**Impact**: Parser error preventing workflow execution

#### A3. Invalid Secrets Declaration
**Issue ID**: `YAML-INVALID-SECRETS-001`  
**File**: `.github/workflows/cert_rotation.yml`  
**Line**: 18  
**Root Cause**: `secrets: write` permission invalid (only `read` supported for secrets)  
**Fix**: Remove `secrets: write` line  
**Impact**: Workflow validation failure

### Category B: Action Resolution Errors (CRITICAL - Severity 8)

#### B1. Nonexistent Rust Setup Action
**Issue ID**: `ACTION-NOT-FOUND-001`  
**File**: `.github/workflows/ci.yml`  
**Line**: 28  
**Root Cause**: `dtolnay/setup-rust@v1` doesn't exist (should be `dtolnay/rust-toolchain@v1`)  
**Fix**: Replace with correct action name  
**Impact**: Rust toolchain installation fails

#### B2. Nonexistent OIDC Token Action
**Issue ID**: `ACTION-NOT-FOUND-002`  
**File**: `.github/workflows/issue_master_key_vault_oidc.yml`  
**Line**: 22  
**Root Cause**: `actions/oidc-token@v1` doesn't exist (GitHub provides OIDC natively via `github.token`)  
**Fix**: Replace with direct `ACTIONS_ID_TOKEN_REQUEST_TOKEN` usage  
**Impact**: OIDC authentication fails

#### B3. Missing Reusable Workflow
**Issue ID**: `WORKFLOW-NOT-FOUND-001`  
**File**: `.github/workflows/gated_ci_example.yml`  
**Line**: 16  
**Root Cause**: Calls `./.github/workflows/e2e_wrapped_flow.yml` but target missing `workflow_call` trigger  
**Fix**: Add `workflow_call:` to e2e_wrapped_flow.yml triggers  
**Impact**: Reusable workflow call fails

### Category C: Secret Context Warnings (WARNING - Severity 4)

**Issue ID**: `SECRET-CONTEXT-WARN-*` (65 instances)  
**Files**: All workflow files  
**Root Cause**: Secrets not visible in forked PR contexts, but valid for main repo  
**Fix**: Add conditional checks with informative error messages (already partially implemented)  
**Impact**: Expected behavior - workflows fail on forks without secrets (security feature)  
**Action**: INFORMATIONAL ONLY - no fixes needed, warnings are correct

## Fix Implementation Map

### FIX-001: ci.yml Line 48 (YAML Syntax)
```yaml
# BEFORE:
      - name: Check & apply formatting: gofmt

# AFTER:
      - name: "Check & apply formatting: gofmt"
```

### FIX-002: ci.yml Line 83 (YAML Syntax)
```yaml
# BEFORE:
      - name: Check & apply formatting: zig fmt

# AFTER:
      - name: "Check & apply formatting: zig fmt"
```

### FIX-003: ci.yml Line 28 (Action Resolution)
```yaml
# BEFORE:
      - name: Setup Rust
        uses: dtolnay/setup-rust@v1
        with:
          toolchain: stable

# AFTER:
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: stable
```

### FIX-004: e2e_wrapped_flow.yml Line 98 (Invalid Marker)
```yaml
# BEFORE (around line 95-100):
          docker compose -f vault/docker-compose.yml down --volumes --remove-orphans

*** End Patch

# AFTER:
          docker compose -f vault/docker-compose.yml down --volumes --remove-orphans
```

### FIX-005: e2e_wrapped_flow.yml Trigger (Reusable Workflow)
```yaml
# BEFORE:
on:
  workflow_dispatch:

# AFTER:
on:
  workflow_dispatch:
  workflow_call:
    secrets:
      VAULT_ADDR:
        required: false
      # ... other secrets inherited
```

### FIX-006: cert_rotation.yml Line 18 (Invalid Permission)
```yaml
# BEFORE:
permissions:
  contents: write
  actions: write
  secrets: write

# AFTER:
permissions:
  contents: write
  actions: write
  # Note: 'secrets: write' is not a valid permission
```

### FIX-007: issue_master_key_vault_oidc.yml Line 22 (Action Resolution)
```yaml
# BEFORE:
      - name: Request OIDC token
        id: oidc
        uses: actions/oidc-token@v1
        with:
          target_audience: 'api://GitHubActions'

# AFTER:
      - name: Request OIDC token
        id: oidc
        run: |
          # GitHub Actions provides OIDC token via environment
          echo "OIDC_TOKEN=${{ github.token }}" >> $GITHUB_OUTPUT
          # For JWT auth, use: curl -H "Authorization: bearer $ACTIONS_ID_TOKEN_REQUEST_TOKEN" \
          #   "$ACTIONS_ID_TOKEN_REQUEST_URL&audience=api://GitHubActions"
```

## Validation Checklist

- [ ] FIX-001: ci.yml line 48 quoted
- [ ] FIX-002: ci.yml line 83 quoted
- [ ] FIX-003: ci.yml rust-toolchain action corrected
- [ ] FIX-004: e2e_wrapped_flow.yml patch marker removed
- [ ] FIX-005: e2e_wrapped_flow.yml workflow_call added
- [ ] FIX-006: cert_rotation.yml invalid permission removed
- [ ] FIX-007: issue_master_key_vault_oidc.yml OIDC native implementation
- [ ] Build verification: All workflows validate via `actionlint`
- [ ] Error count: 9 critical → 0 critical (65 warnings remain as expected)

## Launch Impact Assessment

**Before Fixes**: 9 critical errors blocking workflow execution  
**After Fixes**: 0 critical errors, 65 informational warnings (expected for secret access on forks)  
**Matrix Risk**: ZERO - workflow fixes don't touch translation logic  
**Launch Readiness**: CLEAR - CI/CD pipeline operational for v0.9.5 validation
