# TODO - pip-rs Development

## Vision: The Fastest pip-Compatible Tool

**Goal**: Match uv speed while keeping 100% pip CLI compatibility.

## Current Status: v1.0 Ready, v1.1 Performance Focus

### v1.0 Complete ✅
- 21 features, 260+ tests, production ready
- 5-20x faster than pip (baseline)

### v1.1 Focus: Performance & UX

#### Performance (Target: Match uv)
- [x] **Lazy initialization** - ~5ms startup (was ~50ms)
- [x] **Smart cache** - Hashed keys, subdirectories, get_or_fetch
- [x] **Prefetch cache** - Background metadata refresh
- [x] **Streaming JSON** - Handle large packages without timeout
- [x] **Request batching** - Batch PyPI API calls

#### User Experience
- [x] **Progress bars** - Visual feedback with indicatif
- [x] **Quiet mode** (-q) - Minimal output for scripts
- [x] **Smart defaults** - Auto-detect venv, cache aggressively
- [x] **Better errors** - Actionable suggestions

#### Compatibility
- [x] **pip config** - Full pip.conf support
- [x] **Constraints files** - `-c constraints.txt`
- [x] **Trusted hosts** - `--trusted-host`

## Performance Targets

| Metric | pip | uv | pip-rs v1.0 | pip-rs v1.1 |
|--------|:---:|:--:|:-----------:|:-----------:|
| Startup | 200ms | 10ms | 50ms | **5ms** |
| Install (cached) | 5s | 0.2s | 0.5s | **0.2s** |
| List outdated | 30s | 2s | 3s | **2s** |
| Memory | 50MB | 10MB | 15MB | **10MB** |

## Implementation Priority

### Phase 11: Performance Sprint ✅
1. [x] Lazy client initialization (high impact, low effort)
2. [x] Progress bars with indicatif (high UX impact)
3. [x] Quiet mode for CI/CD (quick win)
4. [x] Prefetch cache warming (background task)

### Phase 12: Compatibility ✅
1. [x] Full pip.conf parsing
2. [x] Constraints file support
3. [x] Trusted host support
4. [x] Index URL from environment

### Phase 13: Polish ✅
1. [x] Streaming JSON for large packages
2. [x] Request batching
3. [x] Memory optimization - Version parsing cache, concurrent resolution
4. [x] Binary size reduction - Release profile optimizations, dependency feature flags

### Phase 14: Code Quality & Modularization ✅
1. [x] Modularize into workspace (`pip-rs` CLI and `pip-rs-core`)
2. [x] Fix variable shadowing and compilation errors
3. [x] Remove dead code (unused functions, traits, modules)
4. [x] Fix compilation warnings (imports, unused variables)
5. [x] Implement proper package details extraction in `pip-rs-core`

## Known Issues

1. ~~**Large Package Timeouts** - grpcio, clickhouse-connect~~ ✅ Fixed
   - ~~Fix: Streaming JSON parsing (Phase 13)~~ ✅ Implemented

2. ~~**Startup Overhead** - ~50ms currently~~ ✅ Fixed
   - ~~Fix: Lazy initialization (Phase 11)~~ ✅ Implemented

3. ~~**Update not detecting outdated packages**~~ ✅ Fixed (Dec 2024)
   - Root cause: reqwest had `default-features = false` which disabled TLS
   - Fix: Enabled `rustls-tls` feature for HTTPS support
   - Also fixed: Package name normalization for upgrade_packages command

4. ~~**Update command hanging during download**~~ ✅ Fixed (Dec 2024)
   - Root cause: Native pip-rs install had wheel filename parsing issues
   - Fix: Use pip subprocess for batch upgrades (more reliable, faster)

## Test Coverage

- **260+ tests** (100% pass)
- **Clean build** (minor warnings)

## Release Plan

- [x] v1.0.0-rc.1 - Feature complete
- [ ] v1.0.0 - Stable release
- [ ] v1.1.0 - Performance release (Fast Update Focus)

## Fast Update Optimizations ✅

- [x] **Batch installation** - Single command for all packages (10-50x faster)
- [x] **Increased concurrency** - 20 concurrent requests (2x faster scanning)
- [x] **Native pip-rs installation** - No subprocess overhead
- [x] **Removed slow operations** - Skip version verification
- [x] **Optimized detection** - Native site-packages with venv auto-detection
- [x] **Extended cache** - 24-hour TTL for faster repeated runs

**Result**: 6-9x faster package updates overall

### Phase 14: Modernization & Modularization (In Progress)
1. [ ] **Modularization** - Split into `pip-rs-core` and `pip-rs-cli` crates
2. [ ] **Dependency Updates** - Update reqwest, clap, pulldown-cmark
3. [ ] **Snapshot Testing** - Adopt `insta` for snapshot testing
4. [ ] **Documentation Style** - Apply Carbon Design System to `docs/index.html`

