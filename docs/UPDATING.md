# Updating Sprite

This guide covers how to keep Sprite up-to-date with the latest features and bug fixes.

## Quick Update

```bash
# Check for updates
sprite update --check

# Update to latest version
sprite update

# Update without confirmation
sprite update --yes
```

## What Gets Updated?

The `sprite update` command will:

1. ✅ Check GitHub Releases for the latest version
2. ✅ Show you what's new in the release notes
3. ✅ Create a backup of your current installation
4. ✅ Update using the same method you used to install:
   - **Cargo installation**: Runs `cargo install --force --git ...`
   - **Binary installation**: Downloads and replaces the binary

## Latest Release: v0.1.0

### 🎉 Major Features

- **Auto-Update Mechanism** - `sprite update` command
- **One-Command Init** - No manual git setup needed
- **Fixed `sprite start`** - Profile scripts now execute correctly
- **Worktree Edge Cases** - Handles all git worktree scenarios

### 🐛 Critical Fixes

1. **Profile Script Execution** (#19)
   - Fixed: `sprite start` was completely broken
   - Now: Profile scripts execute as bash scripts correctly
   - Impact: `sprite start` actually works!

2. **Worktree Edge Cases** (#17)
   - Fixed: "missing but registered worktree" errors
   - Now: Auto-prunes stale entries, handles deleted directories
   - Impact: `sprite init` works in all scenarios

3. **One-Command Init** (#15)
   - Before: 10+ manual git steps required
   - Now: `sprite init` handles everything
   - Impact: 90% faster setup

### ✨ New Commands

```bash
# Auto-update commands
sprite update --check    # Check for updates
sprite update            # Update to latest
sprite update --yes      # Skip confirmation
```

## Manual Update (Without Command)

If you prefer to update manually:

### From Cargo

```bash
cargo install --force --git https://github.com/hotaq/Sprit-mutil
```

### From Binary

1. Download the latest release from:
   https://github.com/hotaq/Sprit-mutil/releases/latest

2. Replace your current binary:
   ```bash
   # macOS/Linux
   sudo mv sprite /usr/local/bin/sprite
   sudo chmod +x /usr/local/bin/sprite
   
   # Or to ~/.local/bin
   mv sprite ~/.local/bin/sprite
   chmod +x ~/.local/bin/sprite
   ```

3. Verify the update:
   ```bash
   sprite --version
   ```

## Update Best Practices

### Before Updating

1. **Check what's new**:
   ```bash
   sprite update --check
   ```
   Read the release notes to understand what changed

2. **Backup your configuration** (optional):
   ```bash
   cp -r agents agents.backup
   ```

3. **Stop running sessions**:
   ```bash
   sprite kill
   ```

### After Updating

1. **Verify the update**:
   ```bash
   sprite --version
   ```
   Should show: `sprite 0.1.0` or later

2. **Test basic commands**:
   ```bash
   sprite status
   sprite start --help
   ```

3. **Report issues**:
   If you encounter any problems, please create an issue at:
   https://github.com/hotaq/Sprit-mutil/issues

## Rollback (If Needed)

If you need to rollback to a previous version:

### Cargo Installation

```bash
# Install specific version from git tag
cargo install --force --git https://github.com/hotaq/Sprit-mutil --tag v0.0.9
```

### Binary Installation

```bash
# Restore from backup (created automatically)
mv ~/.local/bin/sprite.backup ~/.local/bin/sprite
```

## Version History

### v0.1.0 (2025-10-21) - Initial Release

**Features:**
- Auto-update mechanism
- One-command init with auto git setup
- Fixed profile script execution
- Worktree edge case handling

**Breaking Changes:**
- None (initial release)

**Migration:**
- No migration needed
- Existing projects continue to work

## Troubleshooting

### "Command not found: sprite update"

**Cause**: You're running an old version without update support.

**Solution**: Manual update required:
```bash
cargo install --force --git https://github.com/hotaq/Sprit-mutil
```

### "No releases found yet"

**Cause**: GitHub API rate limit or network issue.

**Solution**: Try again later or update manually.

### Update fails with permission error

**Cause**: Binary is in a protected directory.

**Solution**: Run with sudo or install to ~/.local/bin:
```bash
# Option 1: Use sudo
sudo sprite update

# Option 2: Reinstall to user directory
cargo install --force --git https://github.com/hotaq/Sprit-mutil
```

## Stay Updated

- 📢 Watch the repository for release notifications
- ⭐ Star the project to stay informed
- 📝 Check the [changelog](../CHANGELOG.md) regularly
- 🐛 Report issues to help improve Sprite

## Automatic Updates (Coming Soon)

Future versions may include:
- Automatic update checks on startup
- Configurable update frequency
- Update notifications
- Background updates

---

**Always stay up-to-date for the best experience!** 🚀
