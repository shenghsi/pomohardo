# GitHub Actions Windows Upload Fix

## Problem
Windows binaries (`.exe` and `.msi` files) were being created and uploaded to the GitHub release, but were **not** being copied to the `pomohardo-releases` repository. This meant:
- ✅ Files appeared in GitHub Releases (e.g., `Pomohardo_0.10.1_x64-setup.exe`, `Pomohardo_0.10.1_x64_en-US.msi`)
- ❌ Files were missing from the `pomohardo-releases` repo (needed for auto-updates and direct downloads)

## Root Cause
The workflow used `find` commands with `-exec` to copy files:
```bash
find src-tauri/target/.../bundle/msi -name "*.msi.zip*" -exec cp {} releases-repo/ \; 2>/dev/null || true
```

**Issues:**
1. The `find` command on Windows (even in Git Bash) doesn't always work reliably with `-exec`
2. The `|| true` at the end **silently ignored all errors**, so the workflow appeared to succeed even when no files were copied
3. No debugging output to see what was happening
4. The `git commit ... || true` meant even if nothing was copied, the workflow continued without error

## Solution
Replaced `find` commands with simpler, more reliable bash globbing and added comprehensive debugging:

### Key Changes:
1. **Use glob patterns instead of find:**
   ```bash
   cp src-tauri/target/.../bundle/msi/*.msi.zip releases-repo/ 2>/dev/null && echo "Copied" || echo "Not found"
   ```

2. **Add debugging output:**
   - List bundle directories before copying
   - Echo success/failure for each copy operation
   - Show final contents of releases-repo

3. **Better error handling:**
   - Check if files were actually staged before committing
   - Only push if there are changes
   - Clear success messages

4. **Applied to all platforms:**
   - Windows (main fix)
   - Linux (consistency)
   - macOS (consistency)

## Testing the Fix
When you create the next release, you should see in the GitHub Actions logs:
```
=== Listing bundle directories ===
=== MSI directory contents ===
=== NSIS directory contents ===
=== Copying MSI updater files ===
Copied MSI zip files
=== Copying NSIS installer ===
Copied EXE installer
=== Files in releases-repo ===
Successfully pushed Windows release files
```

## Next Steps
1. Commit and push this workflow change
2. Create a new release tag (or re-run the v0.10.1 workflow if possible)
3. Check the Actions logs for the debugging output
4. Verify files appear in `pomohardo-releases` repo

## Files Modified
- `.github/workflows/release.yml` - All three upload steps (Linux, Windows, macOS)
