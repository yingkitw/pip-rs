# Upgrade Command - Batch Package Updates

## Status: ✅ IMPLEMENTED

Implemented `pip upgrade` command to check and display all outdated packages with upgrade instructions.

## Features

### 1. Check All Packages ✅
- Scans site-packages for installed packages
- Checks each package against PyPI for updates
- Real-time streaming of results
- Shows current and latest versions

### 2. Real-Time Display ✅
- Displays packages as they're checked
- Shows status: "OUTDATED" or "up-to-date"
- Progress updates every 100 packages
- Concurrent fetching (5 concurrent requests)

### 3. Upgrade Instructions ✅
- Summary of outdated packages
- Command to upgrade all at once
- Individual upgrade commands
- Easy copy-paste for users

## Usage

### Check all packages for updates
```bash
pip upgrade
# or
pip update
```

### Output Example
```
Checking for updates for all packages...

Package                                            Current              Latest               Status
----------------------------------------------------------------------------------------------------
Checking 1025 packages for updates (real-time streaming)...
absl_py                                            2.1.0                2.3.1                OUTDATED
accelerate                                         1.3.0                1.11.0               OUTDATED
acp_sdk                                            0.13.0               1.0.3                OUTDATED
acres                                              0.2.0                0.5.0                OUTDATED
...
Progress: 100/1025 packages checked
...
Progress: 1025/1025 packages checked

====================================================================================================
Summary: 250+ outdated packages found

To upgrade all outdated packages, run:
  pip install --upgrade absl_py accelerate acp_sdk acres ...

Or upgrade individually:
  pip install --upgrade absl_py==2.3.1
  pip install --upgrade accelerate==1.11.0
  pip install --upgrade acp_sdk==1.0.3
  ...
```

## Implementation Details

### Architecture

```rust
pub async fn handle_upgrade_all() -> Result<i32> {
    // 1. Get installed packages from site-packages
    let packages = get_installed_packages()?;
    
    // 2. Create channel for real-time streaming
    let (tx, mut rx) = mpsc::channel(100);
    
    // 3. Spawn fetching task
    tokio::spawn(async move {
        // Spawn all tasks with 5 concurrent limit
        for pkg in packages {
            let handle = tokio::spawn(async move {
                // Fetch from PyPI
                // Send result to channel
            });
        }
    });
    
    // 4. Display results in real-time
    while let Some((name, version, latest, is_outdated)) = rx.recv().await {
        if is_outdated {
            println!("{} {} -> {}", name, version, latest);
        }
    }
    
    // 5. Show summary and upgrade instructions
    println!("To upgrade all: pip install --upgrade {}", names.join(" "));
}
```

### Performance

| Metric | Value |
|--------|-------|
| Concurrent requests | 5 |
| Channel buffer | 100 |
| Time for 1000 packages | ~100-150 seconds |
| First result | ~2-3 seconds |
| Progress updates | Every 100 packages |

## Files Modified

1. **src/commands/upgrade.rs**
   - Implemented `handle_upgrade_all()` function
   - Real-time streaming with channels
   - Concurrent PyPI lookups
   - Upgrade instructions generation

2. **src/main.rs**
   - Added `Upgrade` command variant
   - Added `update` alias
   - Wired up command handler

3. **Cargo.toml**
   - Added `glob = "0.3"` dependency

## Command Variants

### Upgrade all outdated packages
```bash
pip upgrade
```

### Upgrade specific packages (future)
```bash
pip upgrade package1 package2 package3
```

### Alias support
```bash
pip update  # Same as pip upgrade
```

## Comparison with Official pip

### Official pip
```bash
pip list --outdated          # Shows outdated packages
pip install --upgrade pip    # Upgrade specific package
pip install -U package1 package2  # Upgrade multiple
```

### pip-rs
```bash
pip list --outdated          # Shows outdated packages (real-time)
pip upgrade                  # Shows outdated + upgrade instructions
pip upgrade package1         # Future: upgrade specific (not yet)
```

## Future Enhancements

### 1. Upgrade Specific Packages
```bash
pip upgrade package1 package2
```

### 2. Dry-Run Mode
```bash
pip upgrade --dry-run
```

### 3. Interactive Mode
```bash
pip upgrade --interactive
# Select which packages to upgrade
```

### 4. Auto-Upgrade
```bash
pip upgrade --yes
# Automatically upgrade all outdated packages
```

### 5. Exclude Packages
```bash
pip upgrade --exclude package1,package2
```

### 6. Include Only
```bash
pip upgrade --include package1,package2
```

## Test Status

✅ All 25 unit tests passing
✅ Build successful
✅ Real-time streaming working
✅ Upgrade instructions generated
✅ No breaking changes

## Example Workflow

### 1. Check for outdated packages
```bash
$ pip upgrade
Checking for updates for all packages...

Package                                            Current              Latest               Status
----------------------------------------------------------------------------------------------------
absl_py                                            2.1.0                2.3.1                OUTDATED
accelerate                                         1.3.0                1.11.0               OUTDATED
...
Progress: 100/1025 packages checked
...

Summary: 250+ outdated packages found

To upgrade all outdated packages, run:
  pip install --upgrade absl_py accelerate acp_sdk ...
```

### 2. Copy and run upgrade command
```bash
$ pip install --upgrade absl_py accelerate acp_sdk acres ...
Successfully installed absl_py-2.3.1 accelerate-1.11.0 ...
```

### 3. Verify upgrades
```bash
$ pip list --outdated
# Should show fewer outdated packages
```

## Recommendations

1. **Implement Specific Package Upgrade**
   - Allow `pip upgrade package1 package2`
   - Fetch and display versions for specific packages

2. **Add Dry-Run Mode**
   - Show what would be upgraded
   - Don't actually upgrade

3. **Add Interactive Mode**
   - Ask user which packages to upgrade
   - Show size and dependencies

4. **Add Auto-Upgrade**
   - `pip upgrade --yes` to auto-upgrade all
   - Useful for automation

5. **Add Filtering**
   - `pip upgrade --exclude package1,package2`
   - `pip upgrade --include package1,package2`

## Conclusion

The `pip upgrade` command provides a convenient way to check all packages for updates and get instructions on how to upgrade them. It combines the functionality of `pip list --outdated` with helpful upgrade commands.

**Status**: Implemented and working ✅
**Performance**: Real-time streaming with 5 concurrent requests
**User Experience**: Clear output with actionable upgrade instructions
**Next**: Implement specific package upgrade and dry-run mode
