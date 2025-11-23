# pip-rs Setup and Development Guide

## Project Setup

### Prerequisites
- Rust 1.70+ (edition 2021)
- Cargo
- Git

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/yourusername/pip-rs.git
cd pip-rs
```

2. **Build the project**
```bash
cargo build --release
```

3. **Run tests**
```bash
cargo test
```

4. **Run benchmarks**
```bash
cargo run --release --bin benchmarks
```

## Project Structure

```
pip-rs/
├── src/                    # Source code
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Library exports
│   ├── cli/               # Command-line interface
│   ├── commands/          # Command implementations
│   ├── models/            # Data models
│   ├── network/           # PyPI API client
│   ├── resolver/          # Dependency resolution
│   ├── installer/         # Package installation
│   ├── cache/             # Package caching
│   ├── venv/              # Virtual environments
│   ├── config/            # Configuration management
│   └── utils/             # Utility functions
├── tests/                 # Integration tests
├── benches/               # Performance benchmarks
├── Cargo.toml             # Project manifest
├── Cargo.lock             # Dependency lock file
├── .gitignore             # Git ignore rules
└── Documentation/         # Project documentation
```

## Development Workflow

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo run --release --bin benchmarks
```

### Code Quality
```bash
# Check code
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Fix warnings
cargo fix
```

### Documentation
```bash
# Build documentation
cargo doc --open

# Run doc tests
cargo test --doc
```

## Git Workflow

### Initial Setup
```bash
git init
git add .
git commit -m "Initial commit: pip-rs complete migration"
```

### Ignoring Files
The `.gitignore` file is configured to ignore:
- Build artifacts (`/target/`)
- IDE files (`.vscode/`, `.idea/`)
- Python artifacts (`__pycache__/`, `*.pyc`)
- Virtual environments (`venv/`, `.venv`)
- Temporary files (`*.tmp`, `*.bak`)
- OS files (`.DS_Store`, `Thumbs.db`)

### Committing Changes
```bash
# Check status
git status

# Stage changes
git add .

# Commit
git commit -m "Description of changes"

# Push
git push origin main
```

## Dependencies

### Core Dependencies
- **clap** 4.4 - CLI argument parsing
- **reqwest** 0.11 - HTTP client
- **tokio** 1 - Async runtime
- **serde** 1.0 - Serialization
- **anyhow** 1.0 - Error handling
- **zip** 0.6 - Wheel file handling
- **tempfile** 3 - Temporary files
- **walkdir** 2 - Directory traversal
- **pulldown-cmark** 0.9 - Markdown parsing
- **tracing** 0.1 - Logging

### Dev Dependencies
- **insta** 1.34 - Snapshot testing
- **tokio-test** 0.4 - Async testing

## Running the Application

### Basic Usage
```bash
# Show help
./target/release/pip --help

# Show package information
./target/release/pip show requests

# Search for packages
./target/release/pip search numpy

# List installed packages
./target/release/pip list

# Check for conflicts
./target/release/pip check
```

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test integration_tests
```

### All Tests
```bash
cargo test
```

### Test Coverage
- Unit tests: 23 tests
- Integration tests: 10 tests
- Doc tests: 7 tests
- Total: 40+ tests
- Pass rate: 100%

## Performance

### Benchmarks
```bash
cargo run --release --bin benchmarks
```

### Results
- Version parsing: 89ns per operation
- Requirement parsing: 203ns per operation
- Config creation: 140ns per operation
- Virtual environment operations: 517ns per operation

## Documentation

### Available Documentation
- **README.md** - Project overview
- **ARCHITECTURE.md** - Architecture guide
- **PROGRESS.md** - Progress tracking
- **STATUS.md** - Current status
- **TODO.md** - Task list
- **FINAL_SUMMARY.md** - Project summary
- **VERIFICATION.md** - Verification checklist
- **PHASE*.md** - Phase-specific reports

### Building Documentation
```bash
cargo doc --open
```

## Troubleshooting

### Build Issues
```bash
# Clean build
cargo clean
cargo build

# Check for issues
cargo check

# Run clippy
cargo clippy
```

### Test Failures
```bash
# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --nocapture
```

### Performance Issues
```bash
# Run benchmarks
cargo run --release --bin benchmarks

# Profile with perf (Linux)
perf record ./target/release/pip show requests
perf report
```

## Contributing

### Code Style
- Follow Rust conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting
- Add tests for new features

### Commit Messages
- Use clear, descriptive messages
- Reference issues when applicable
- Keep commits atomic

### Pull Requests
- Include tests for changes
- Update documentation
- Pass all tests before submitting

## Release Process

### Version Bumping
```bash
# Update version in Cargo.toml
# Update CHANGELOG.md
# Commit changes
git commit -m "Release v0.2.0"
git tag v0.2.0
git push origin main --tags
```

### Publishing to crates.io
```bash
# Login
cargo login

# Publish
cargo publish
```

## Resources

### Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### Python Packaging
- [PEP 508 - Dependency specification](https://www.python.org/dev/peps/pep-0508/)
- [PEP 427 - Wheel format](https://www.python.org/dev/peps/pep-0427/)
- [PEP 440 - Version scheme](https://www.python.org/dev/peps/pep-0440/)

### Tools
- [PyPI](https://pypi.org/)
- [Cargo](https://doc.rust-lang.org/cargo/)
- [Rustfmt](https://github.com/rust-lang/rustfmt)
- [Clippy](https://github.com/rust-lang/rust-clippy)

## Support

For issues or questions:
1. Check existing documentation
2. Search GitHub issues
3. Create a new issue with details
4. Include reproduction steps
5. Provide system information

## License

MIT License - See LICENSE file for details

## Acknowledgments

- Original pip project for inspiration
- Rust community for excellent tools
- Contributors and testers
