# pip-rs Architecture

## Overview

pip-rs is a high-performance Rust implementation of Python's pip package manager. It maintains architectural parity with pip while leveraging Rust's safety, concurrency, and performance characteristics.

## Core Architecture

### Module Organization

```
src/
├── main.rs                    # CLI entry point and command routing
├── lib.rs                     # Library root, module exports
├── cli/                       # Command-line interface
│   ├── mod.rs                 # CLI module root
│   └── parser.rs              # Argument parsing with clap
├── commands/                  # Command implementations
│   ├── install.rs             # Package installation
│   ├── uninstall.rs           # Package removal
│   ├── list.rs                # Package listing & outdated detection
│   ├── show.rs                # Package information
│   ├── search.rs              # Package search
│   ├── check.rs               # Dependency checking
│   ├── freeze.rs              # Requirements generation
│   ├── download.rs            # Offline package download
│   ├── lock.rs                # Lock file generation
│   ├── debug.rs               # System diagnostics
│   ├── completion.rs          # Shell completion
│   └── upgrade/               # Update functionality (modularized)
│       ├── mod.rs             # Upgrade command root
│       ├── detector.rs        # Outdated package detection
│       ├── installer.rs       # Upgrade installation
│       ├── handler.rs         # Upgrade orchestration
│       ├── progress.rs        # Progress reporting
│       ├── conflict.rs        # Conflict detection
│       ├── traits.rs          # Trait definitions
│       └── default_impl.rs    # Default implementations
├── network/                   # PyPI communication
│   ├── mod.rs                 # Network module root
│   ├── client.rs              # HTTP client with retry logic
│   ├── pypi.rs                # PyPI API interactions
│   └── index.rs               # Package index management
├── resolver/                  # Dependency resolution
│   ├── mod.rs                 # Resolver module root
│   ├── resolver.rs            # Main resolution algorithm
│   ├── lockfile.rs            # Lock file handling
│   ├── extras.rs              # Extras resolution
│   └── version_spec.rs        # Version constraint parsing
├── installer/                 # Package installation
│   ├── mod.rs                 # Installer module root
│   ├── installer.rs           # Main installation logic
│   ├── wheel.rs               # Wheel file handling
│   ├── editable.rs            # Editable installs (.pth)
│   ├── entry_point.rs         # Entry point generation
│   └── site_packages.rs       # Site-packages management
├── cache/                     # Caching mechanisms
│   ├── mod.rs                 # Cache module root
│   ├── package_cache.rs       # Package metadata cache
│   └── disk_cache.rs          # Disk-based caching
├── config/                    # Configuration handling
│   ├── mod.rs                 # Config module root
│   ├── config.rs              # pip.conf/pip.ini parsing
│   └── pyproject.rs           # pyproject.toml parsing
├── models/                    # Data structures
│   ├── mod.rs                 # Models module root
│   ├── package.rs             # Package metadata
│   ├── requirement.rs         # Requirement specifications
│   ├── metadata.rs            # Installation metadata
│   └── marker.rs              # Environment markers
├── errors.rs                  # Error types and handling
└── utils/                     # Utility functions
    ├── mod.rs                 # Utils module root
    ├── validation.rs          # Input validation
    └── security.rs            # Security checks
```

## Key Design Patterns

### 1. Command Pattern
Each command is a separate module implementing a consistent interface:
```rust
pub async fn handle_<command>(args) -> Result<i32, PipError>
```

### 2. Trait-Based Design
Traits enable testability and flexibility:
- `ConflictDetector` - Detect version conflicts
- `ProgressReporter` - Report upgrade progress
- `PackageResolver` - Resolve dependencies

### 3. Async/Await
All I/O operations use Tokio for concurrency:
- Parallel PyPI requests (10 concurrent with semaphore)
- Non-blocking package fetching
- Real-time streaming results

### 4. Error Handling
Custom error types with context:
```rust
pub enum PipError {
    NetworkError { message, retries, last_error },
    PackageNotFound { name, version },
    DependencyConflict { package, required, installed },
    // ... more variants
}
```

## Data Flow

### Installation Flow
```
User Input
    ↓
Parse Requirements (PEP 508)
    ↓
Resolve Dependencies
    ├─ Fetch package metadata from PyPI
    ├─ Parse version constraints
    └─ Build dependency graph
    ↓
Download Wheels
    ├─ Find best wheel (platform-specific)
    └─ Download with retry logic
    ↓
Extract & Install
    ├─ Extract wheel contents
    ├─ Install files to site-packages
    ├─ Generate entry points
    └─ Write metadata
    ↓
Success/Error Report
```

### Outdated Detection Flow
```
Discover Installed Packages
    ├─ Scan ~/Library/Python/*/lib/python/site-packages
    ├─ Scan /Library/Frameworks/Python.framework/Versions/*/lib/python*/site-packages
    └─ Parse .dist-info directories
    ↓
Fetch Latest Versions (Parallel)
    ├─ 10 concurrent requests to PyPI
    ├─ Timeout: 180s (handles large packages)
    └─ Retry with exponential backoff
    ↓
Compare Versions (PEP 440)
    ├─ Parse versions with pep440 crate
    ├─ Handle pre-releases, post-releases, dev versions
    └─ Normalize package names
    ↓
Display Results
    └─ Show outdated packages with versions
```

## Network Layer

### HTTP Client
- **Connection Pooling**: Reuses TCP connections (10 per host)
- **Timeout**: 180s request, 30s connect
- **Retry Logic**: Exponential backoff (2 retries)
- **User-Agent**: Identifies as pip-rs

### PyPI Integration
- **Base URL**: `https://pypi.org/pypi`
- **API Format**: JSON endpoint (`/package/json`)
- **Package Name Normalization**: Handles hyphens/underscores
- **Caching**: In-memory cache for metadata

## Performance Optimizations

### 1. Parallel Requests
```rust
let semaphore = Arc::new(Semaphore::new(10));
// Limits concurrent requests to 10
```

### 2. Connection Pooling
```rust
.pool_max_idle_per_host(10)
// Reuses HTTP connections
```

### 3. Real-Time Streaming
Results displayed immediately as fetched, not buffered.

### 4. Caching
- In-memory package metadata cache
- Disk cache infrastructure (ready for integration)

## Site-Packages Discovery

Searches multiple locations in order:
1. `~/Library/Python/3.12/lib/python/site-packages` (macOS user)
2. `~/Library/Python/3.11/lib/python/site-packages`
3. `~/Library/Python/3.10/lib/python/site-packages`
4. `/Library/Frameworks/Python.framework/Versions/3.12/lib/python3.12/site-packages` (macOS framework)
5. `/usr/local/lib/python3.12/site-packages` (Linux)
6. `~/.local/lib/python3.12/site-packages` (Linux user)

Deduplicates packages across locations (prefers earlier paths).

## Version Comparison

Uses PEP 440 parsing for accurate version comparison:
- Handles pre-releases: `1.0.0a1`, `1.0.0b2`, `1.0.0rc1`
- Handles post-releases: `1.0.0.post1`
- Handles dev releases: `1.0.0.dev1`
- Fallback to numeric comparison for unparseable versions

## Testing Strategy

### Unit Tests
- Module-level tests in `src/`
- Test individual functions and logic

### Integration Tests
- `tests/integration_tests.rs` - Full command workflows
- `tests/e2e_tests.rs` - End-to-end scenarios
- `tests/version_comparison_test.rs` - Version comparison accuracy

### Test Coverage
- 25+ unit tests (100% pass rate)
- Integration tests for core commands
- Version comparison tests for PEP 440 compliance

## Security Considerations

1. **Input Validation**: All user inputs validated
2. **URL Safety**: Package names sanitized before URL construction
3. **File Path Safety**: Validated before file operations
4. **SSL/TLS**: Uses HTTPS for PyPI communication
5. **Error Messages**: No sensitive information leaked

## Future Improvements

1. **Disk Caching**: Integrate disk cache for 10-20x faster repeated runs
2. **Lock Files**: Full lock file support for reproducible installs
3. **Multiple Indexes**: Support for private PyPI mirrors
4. **Advanced Conflict Resolution**: Better handling of complex dependency graphs
5. **Performance**: Target 50-100x faster than sequential operations

## Dependencies

Key external crates:
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `serde_json` - JSON parsing
- `clap` - CLI argument parsing
- `pep440` - PEP 440 version parsing
- `shellexpand` - Path expansion
- `anyhow` - Error handling
- `lazy_static` - Global state management
