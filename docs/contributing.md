# Contributing Guide

Welcome! We're excited that you're interested in contributing to FabreXLens. This guide will help you get started with development, testing, and contributing to the project.

## 🚀 Quick Start for Contributors

### Prerequisites

- **Rust**: 1.75 or later
- **Cargo**: Latest stable version
- **Git**: For version control
- **Optional**: Docker for containerized testing

### First Time Setup

```bash
# Clone the repository
git clone https://github.com/Digital-Data-Co/FabreXLens.git
cd FabreXLens

# Build the project
cargo build

# Run tests
cargo test

# Launch the application
cargo run
```

## 🏗️ Development Workflow

### Branching Strategy

We use a simple branching model:

- **`main`**: Stable production code
- **`feature/*`**: New features and enhancements
- **`bugfix/*`**: Bug fixes
- **`docs/*`**: Documentation updates

### Development Process

1. **Choose or create an issue** on GitHub
2. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   # OR
   git checkout -b bugfix/issue-number-description
   ```

3. **Make your changes** following our coding standards
4. **Test thoroughly** (see Testing section below)
5. **Commit with clear messages:**
   ```bash
   git commit -m "feat: add new dashboard widget

   - Implements real-time metrics display
   - Adds configuration options for refresh interval
   - Includes unit tests for new components

   Closes #123"
   ```

6. **Push and create a pull request**
7. **Address review feedback**
8. **Merge when approved**

### Commit Message Format

We follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Testing
- `chore`: Maintenance

Examples:
```
feat(ui): add dark mode toggle
fix(api): handle timeout errors gracefully
docs: update installation guide
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with verbose output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration

# Run benchmarks
cargo bench
```

### Test Coverage

We aim for high test coverage. Before submitting a PR:

```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View report in browser
open tarpaulin-report.html
```

### Writing Tests

#### Unit Tests

Add unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Arrange
        let input = "test";
        let expected = "result";

        // Act
        let result = my_function(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error_conditions() {
        // Test error cases
        let result = my_function("");
        assert!(result.is_err());
    }
}
```

#### Integration Tests

Add integration tests in `tests/` directory:

```rust
// tests/api_integration.rs
use fabrexlens::services::api::fabrex::FabrexClient;

#[tokio::test]
async fn test_fabrex_client_real_api() {
    // Use httpmock or real test endpoints
    let client = FabrexClient::new("https://test-api.example.com").unwrap();

    // Test real API interactions
    let fabrics = client.list_fabrics().await.unwrap();
    assert!(!fabrics.is_empty());
}
```

#### Mock Testing

We use `httpmock` for API mocking:

```rust
use httpmock::prelude::*;

#[tokio::test]
async fn test_with_mock() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/fabrics");
        then.status(200)
            .json_body(serde_json::json!({"fabrics": []}));
    });

    let client = FabrexClient::new(&server.base_url()).unwrap();
    let result = client.list_fabrics().await.unwrap();

    mock.assert();
    assert!(result.is_empty());
}
```

## 🔧 Code Quality

### Linting and Formatting

```bash
# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Fix auto-fixable issues
cargo clippy --fix
```

### Code Standards

- **Follow Rust idioms** and conventions
- **Use meaningful variable names**
- **Add documentation comments** for public APIs
- **Handle errors appropriately** (use `thiserror` and `anyhow`)
- **Write comprehensive tests**
- **Keep functions small and focused**

### Documentation

- **Document public APIs** with `///` comments
- **Include code examples** where helpful
- **Update README** for significant changes
- **Add changelog entries** for user-facing changes

## 🏭 Building and Releasing

### Local Builds

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Cross-compilation for different platforms
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release
```

### Release Process

1. **Update version** in `Cargo.toml`
2. **Update CHANGELOG.md** with new features/fixes
3. **Create a git tag:**
   ```bash
   git tag -a v1.2.3 -m "Release version 1.2.3"
   git push origin v1.2.3
   ```

4. **GitHub Actions** will automatically:
   - Build binaries for all platforms
   - Create a GitHub release
   - Upload artifacts with checksums

## 🐛 Debugging

### Debug Builds

```bash
# Build with debug symbols
cargo build

# Run with debug logging
RUST_LOG=debug cargo run

# Enable backtraces
RUST_BACKTRACE=1 cargo run
```

### Profiling

```bash
# Install profiling tools
cargo install flamegraph

# Generate flame graph
cargo flamegraph --bin fabrexlens -- test_function
```

### Common Debugging Techniques

1. **Enable logging:**
   ```bash
   export RUST_LOG=fabrexlens=debug
   export FABREXLENS__LOGGING__LEVEL=debug
   ```

2. **Use debugger:**
   ```bash
   # Install debugger
   rustup component add llvm-tools-preview

   # Debug with lldb
   rust-lldb target/debug/fabrexlens
   ```

3. **Memory debugging:**
   ```bash
   # Check for memory leaks
   valgrind target/debug/fabrexlens --headless
   ```

## 🏛️ Architecture Guidelines

### Code Organization

```
src/
├── main.rs              # Application entry point
├── app.rs               # egui application shell
├── cli.rs               # Command-line interface
├── config.rs            # Configuration management
├── services/            # Business logic
│   ├── auth.rs         # Authentication & credentials
│   └── api/            # External API clients
├── ui/                  # User interface components
└── utils/               # Shared utilities
```

### Design Principles

- **Separation of Concerns**: UI, business logic, and data access are separate
- **Dependency Injection**: Services are injected for testability
- **Error Handling**: Comprehensive error handling with context
- **Async/Await**: Use async for I/O operations
- **Type Safety**: Leverage Rust's type system

### API Design

- **Strongly Typed**: Use structs/enums over strings for API data
- **Builder Pattern**: For complex object construction
- **Result Types**: Use `Result<T, E>` for fallible operations
- **Documentation**: Comprehensive API documentation

## 🔒 Security Considerations

### Credential Handling

- **Never log credentials** or sensitive data
- **Use secure keychain storage** for credentials
- **Validate input** to prevent injection attacks
- **Implement timeouts** for network operations

### Code Security

- **Regular dependency updates**: `cargo audit`
- **Security linting**: `cargo clippy -- -W clippy::pedantic`
- **Fuzz testing** for critical components
- **Input validation** and sanitization

## 📊 Performance Guidelines

### Optimization Tips

- **Profile before optimizing**: Use `cargo flamegraph`
- **Minimize allocations**: Reuse buffers where possible
- **Async operations**: Don't block the UI thread
- **Memory efficiency**: Use appropriate data structures

### Benchmarking

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| my_function(black_box(input)))
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

## 🤝 Pull Request Process

### Before Submitting

1. **Update tests** for any changed functionality
2. **Run full test suite**: `cargo test`
3. **Check code quality**: `cargo clippy && cargo fmt --check`
4. **Update documentation** if needed
5. **Test on multiple platforms** if possible

### PR Template

Please fill out the PR template with:

- **Description**: What does this change?
- **Type of change**: Bug fix, feature, documentation, etc.
- **Testing**: How was this tested?
- **Breaking changes**: Does this break existing functionality?
- **Checklist**: All items checked off

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Testing** in staging environment if applicable
4. **Approval** and merge

## 📞 Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and community support
- **Pull Request Comments**: Code review discussions

### Asking for Help

When asking for help:

1. **Be specific**: Include error messages, code snippets, reproduction steps
2. **Provide context**: OS, Rust version, what you're trying to accomplish
3. **Share logs**: Debug logs with sensitive info redacted
4. **Show your work**: What have you tried already?

### Office Hours

We hold virtual office hours for contributors:

- **Schedule**: Every Tuesday 2-3 PM UTC
- **Location**: GitHub Discussions → Office Hours
- **Topics**: Code reviews, architecture discussions, mentoring

## 🎉 Recognition

Contributors are recognized through:

- **GitHub Contributors list**
- **Changelog mentions**
- **Release notes credits**
- **Community recognition** in discussions

Thank you for contributing to FabreXLens! Your efforts help make infrastructure management better for everyone.

---

## 📋 Checklist for Contributors

- [ ] Code follows Rust idioms and project conventions
- [ ] Tests added for new functionality
- [ ] Documentation updated if needed
- [ ] Code passes `cargo clippy` and `cargo fmt`
- [ ] Commit messages follow conventional format
- [ ] PR description is clear and comprehensive
- [ ] Changes work on supported platforms
