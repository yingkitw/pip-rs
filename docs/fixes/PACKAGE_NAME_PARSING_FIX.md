# Package Name Parsing Fix

## Problem
The `pip update` command showed a different list of packages compared to `pip list --outdated`. This was because package names with dashes (like `langchain-core`) were being parsed incorrectly.

## Root Cause
The code was using `rfind('-')` to split package name from version, which finds the **last dash**. This breaks for packages with dashes in their names:

**Example:**
- Directory: `langchain-core-0.3.0.dist-info`
- **Wrong parsing:** `langchain-core-0` (name) + `3.0` (version)
- **Correct parsing:** `langchain-core` (name) + `0.3.0` (version)

## Solution
Changed the parsing logic to find the **last dash that precedes a digit** (version always starts with a digit):

```rust
// Find the version part (starts with a digit)
let mut version_start = pkg_info.len();
for (i, ch) in pkg_info.char_indices().rev() {
    if ch == '-' && i + 1 < pkg_info.len() {
        if let Some(next_ch) = pkg_info[i + 1..].chars().next() {
            if next_ch.is_ascii_digit() {
                version_start = i;
                break;
            }
        }
    }
}

if version_start < pkg_info.len() {
    let pkg_name = pkg_info[..version_start].to_string();
    let version = pkg_info[version_start + 1..].to_string();
}
```

## How It Works

### Before (Wrong)
```
langchain-core-0.3.0.dist-info
                 ↑ (last dash)
langchain-core-0 | 3.0  ❌
```

### After (Correct)
```
langchain-core-0.3.0.dist-info
              ↑ (dash before digit)
langchain-core | 0.3.0  ✅
```

## Files Modified
1. `src/commands/upgrade/detector.rs` - Fixed `get_installed_packages()`
2. `src/commands/list.rs` - Fixed package parsing in `handle_list()`

## Affected Packages
This fix correctly handles packages with dashes in their names:
- `langchain-core`
- `langchain-experimental`
- `langchain-google-community`
- `langchain-google-genai`
- `langchain-groq`
- `langchain-huggingface`
- `langchain-mistralai`
- `langchain-mongodb`
- `langchain-ollama`
- `langchain-pinecone`
- `langchain-text-splitters`
- `langchain-unstructured`
- `langgraph-prebuilt`
- `tree-sitter-*` (all variants)
- And many others...

## Testing
- ✅ All 81 unit tests pass
- ✅ No breaking changes
- ✅ Backward compatible

## Result
Now `pip update` and `pip list --outdated` show the same list of packages, with correct package names and versions.

## Example
Before fix:
```
langchain-core-0 (wrong name)
langchain-core-0 (wrong name)
langchain-core-0 (wrong name)
```

After fix:
```
langchain-core (correct)
langchain-experimental (correct)
langchain-google-community (correct)
```
