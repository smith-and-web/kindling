# Release Command

Create a new release for Kindling.

## Arguments

- `$ARGUMENTS` - The version to release (e.g., `0.3.0`, `0.3.0-alpha`, `0.3.0-beta.1`)

## Instructions

You are creating a release for the Kindling application. Follow these steps exactly:

### 1. Validate the version argument

If no version is provided in `$ARGUMENTS`, ask the user:
- What version should this release be? (e.g., 0.3.0, 0.3.0-alpha, 0.3.0-beta.1)

Validate the version format matches semver (X.Y.Z or X.Y.Z-prerelease).

### 2. Check for uncommitted changes

Run `git status` to ensure there are no uncommitted changes. If there are:
- List the uncommitted files
- Ask the user if they want to proceed anyway or commit first

### 3. Update version in all required files

Update the version string in these three files:
- `package.json` - the `"version"` field
- `src-tauri/tauri.conf.json` - the `"version"` field
- `src-tauri/Cargo.toml` - the `version` field under `[package]`

### 4. Commit the version bump

```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore: bump version to $VERSION"
git push
```

### 5. Create and push the tag

```bash
git tag -a v$VERSION -m "v$VERSION"
git push origin v$VERSION
```

### 6. Confirm success

Tell the user:
- The version has been bumped to `$VERSION`
- The tag `v$VERSION` has been pushed
- The release workflow has been triggered
- Provide the link: `https://github.com/smith-and-web/kindling/actions/workflows/release.yml`
- Remind them the release will be created as a draft and needs to be published manually

### Error Handling

- If any git command fails, stop and report the error
- If the tag already exists, ask the user if they want to delete it and recreate
- If pre-push hooks fail, report the specific error and stop
