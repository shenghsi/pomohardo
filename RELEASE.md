# Release Guide

This guide covers the complete process for releasing a new version of Pomohardo.

## Prerequisites

- All changes committed to git
- Working directory is clean (`git status` shows no uncommitted changes)
- You have push access to the repository

## Release Steps

### 1. Update Version

**Option A: Using npm version (Recommended)**

This automatically updates `package.json` and creates a git commit:

```bash
# Patch version (0.1.0 -> 0.1.1)
npm version patch

# Minor version (0.1.0 -> 0.2.0)
npm version minor

# Major version (0.1.0 -> 1.0.0)
npm version major
```

**Option B: Manual Update**

1. Edit `package.json` and update the `version` field:
   ```json
   {
     "version": "0.2.0"
   }
   ```

2. Sync version to other files:
   ```bash
   npm run sync-version
   ```

### 2. Sync Version (if not using npm version)

If you manually updated `package.json`, sync the version to:
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

```bash
npm run sync-version
```

**Note:** `package-lock.json` will be automatically updated when you run `npm install` or during the build process.

### 3. Build Locally (Optional but Recommended)

Test the build locally before releasing:

```bash
# Build for your current platform
npm run build
```

The built artifacts will be in `src-tauri/target/release/bundle/`:
- **Linux**: `deb/`, `rpm/`, `appimage/` directories
- **Windows**: `msi/`, `nsis/` directories  
- **macOS**: `app/`, `dmg/` directories

**Note:** Building for all platforms requires running on each platform or using CI/CD.

### 4. Commit Version Changes

If you used manual version update (Option B), commit the changes:

```bash
# Review what changed
git status

# Add version files
git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json

# Commit
git commit -m "Bump version to X.Y.Z"
```

If you used `npm version`, the commit was already created automatically.

### 5. Push Changes to GitHub

```bash
# Push commits to main branch
git push origin main
```

### 6. Create and Push Git Tag

The GitHub Actions workflow triggers on tags starting with `v`:

```bash
# Get the current version from package.json
VERSION=$(node -p "require('./package.json').version")

# Create and push the tag
git tag v$VERSION
git push origin v$VERSION
```

**Or if the tag already exists and you need to update it:**

```bash
# Delete local tag
git tag -d v$VERSION

# Delete remote tag
git push origin :refs/tags/v$VERSION

# Create new tag
git tag v$VERSION

# Push new tag
git push origin v$VERSION
```

### 7. Monitor GitHub Actions

1. Go to your repository on GitHub
2. Click the **Actions** tab
3. Find the workflow run for your tag (e.g., `v0.2.0`)
4. Monitor the build progress:
   - `build-tauri (ubuntu-20.04)` - Linux DEB build
   - `build-tauri (windows-latest)` - Windows MSI/NSIS builds
   - `build-tauri (macos-12)` - macOS builds (Intel and Apple Silicon)

### 8. Review and Publish Release

Once all builds complete:

1. Go to **Releases** in your GitHub repository
2. Find the draft release for your version
3. Review the uploaded artifacts:
   - `Pomohardo_X.Y.Z_amd64.deb` (Linux)
   - `Pomohardo_X.Y.Z_x64_en-US.msi` (Windows MSI)
   - `Pomohardo_X.Y.Z_x64-setup.exe` (Windows NSIS)
   - `Pomohardo_X.Y.Z_x64.dmg` (macOS Intel)
   - `Pomohardo_X.Y.Z_aarch64.dmg` (macOS Apple Silicon)
4. Add release notes describing changes
5. Click **Publish release**

## Quick Release Checklist

- [ ] Update version in `package.json`
- [ ] Run `npm run sync-version` (if manual update)
- [ ] Test local build (optional)
- [ ] Commit version changes
- [ ] Push commits to `main`
- [ ] Create and push git tag `vX.Y.Z`
- [ ] Monitor GitHub Actions builds
- [ ] Review draft release
- [ ] Publish release with notes

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backward compatible
- **PATCH** (0.0.1): Bug fixes, backward compatible

## Troubleshooting

### Build fails in CI/CD
- Check the Actions tab for error logs
- Verify all dependencies are correctly specified
- Ensure version numbers are synced correctly

### Tag already exists
- Delete the existing tag (see step 6)
- Create a new tag pointing to the current commit

### Version not syncing
- Run `npm run sync-version` manually
- Verify `package.json` has the correct version
- Check that `scripts/sync-version.js` exists and is executable

## Automated Workflow

The GitHub Actions workflow (`.github/workflows/release.yml`) automatically:
1. Creates a draft release when a tag is pushed
2. Builds for all platforms (Linux, Windows, macOS)
3. Uploads all artifacts to the release
4. Runs in parallel for faster builds

You only need to:
1. Update version
2. Tag and push
3. Review and publish

