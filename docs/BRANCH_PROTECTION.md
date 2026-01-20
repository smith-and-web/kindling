# Branch Protection Policy

This document outlines the recommended branch protection rules for the Kindling repository. These settings should be configured in GitHub under **Settings > Branches > Branch protection rules**.

## Protected Branch: `main`

The `main` branch is the primary production branch and should have the following protections enabled.

### Required Status Checks

Enable **"Require status checks to pass before merging"** with the following checks:

| Check | Job Name | Required |
|-------|----------|----------|
| Commit Messages | `Commit Messages` | Yes |
| Frontend | `Frontend` | Yes |
| Rust | `Rust` | Yes |

> **Note**: Multi-platform builds run only on pushes to main and weekly scheduled runs, not on PRs. This speeds up PR feedback while still verifying builds before release.

Additionally enable:
- **"Require branches to be up to date before merging"** - Ensures PRs are tested against the latest main

### Required Reviews

Enable **"Require a pull request before merging"** with:

| Setting | Value |
|---------|-------|
| Required approving reviews | 1 |
| Dismiss stale pull request approvals when new commits are pushed | Yes |
| Require review from Code Owners | No (optional, enable if CODEOWNERS file exists) |
| Restrict who can dismiss pull request reviews | No |
| Allow specified actors to bypass required pull requests | No |

### Additional Protections

| Setting | Recommended | Notes |
|---------|-------------|-------|
| Require signed commits | Optional | Recommended for enhanced security |
| Require linear history | No | Allow merge commits for clearer history |
| Include administrators | Yes | Admins should follow the same rules |
| Allow force pushes | No | Never allow force pushes to main |
| Allow deletions | No | Prevent accidental branch deletion |

### Conversation Resolution

- **Require conversation resolution before merging**: Yes
  - Ensures all review comments are addressed

## Branch Naming Conventions

While not enforced by GitHub, contributors should follow these branch naming conventions:

| Pattern | Purpose | Example |
|---------|---------|---------|
| `feature/*` | New features | `feature/obsidian-export` |
| `fix/*` | Bug fixes | `fix/cursor-position` |
| `docs/*` | Documentation | `docs/api-reference` |
| `refactor/*` | Code refactoring | `refactor/sidebar-components` |
| `test/*` | Test additions/changes | `test/e2e-import-flow` |
| `chore/*` | Maintenance tasks | `chore/update-dependencies` |
| `release/*` | Release preparation | `release/v1.0.0` |

## Setting Up Branch Protection

### Via GitHub UI

1. Navigate to repository **Settings**
2. Click **Branches** in the sidebar
3. Under "Branch protection rules", click **Add rule**
4. Enter `main` as the branch name pattern
5. Configure settings as described above
6. Click **Create** or **Save changes**

### Via GitHub CLI

```bash
# Note: This requires the gh CLI and appropriate permissions

gh api repos/{owner}/{repo}/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["Commit Messages","Frontend","Rust"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true}' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false
```

## Workflow Dependencies

The branch protection rules depend on these CI workflows:

- **`.github/workflows/ci.yml`** - Main CI pipeline
  - `commitlint` job - Validates commit messages
  - `frontend` job - Linting, formatting, type checking, tests
  - `rust` job - Formatting, Clippy, security audit, tests
  - `build` job - Multi-platform build verification

- **`.github/workflows/security.yml`** - Security scanning (not required for merge)

## Exceptions

### Emergency Fixes

In rare emergency situations (e.g., critical security patches), repository administrators may:

1. Temporarily disable branch protection
2. Push the fix directly to main
3. Immediately re-enable branch protection
4. Document the exception in the commit message

This should be extremely rare and always documented.

### Release Branches

Release branches (`release/*`) may have relaxed rules to allow version bumping:

- Required status checks: Same as main
- Required reviews: 1 (can be from release manager)
- Allow administrators to bypass: Yes (for version tagging)

## Monitoring

Regularly audit branch protection settings:

- Review after any CI workflow changes
- Verify all required checks are still valid
- Check that no unauthorized bypasses have occurred

GitHub provides an audit log (Enterprise/Team plans) to track changes to branch protection rules.

## Related Documentation

- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contributor guidelines
- [SECURITY.md](../SECURITY.md) - Security policy
- [CI Workflow](../.github/workflows/ci.yml) - CI configuration
