# Contributing to Jobsuche

Thank you for your interest in contributing to the Jobsuche Rust client! This document provides guidelines and instructions for setting up your development environment and contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- (Optional) pre-commit tool

### Installing pre-commit

We use pre-commit hooks to ensure code quality. To set them up:

```bash
# Install pre-commit (if not already installed)
# macOS
brew install pre-commit

# Linux
pip install pre-commit

# Install the git hooks
pre-commit install
pre-commit install --hook-type pre-push
```

### Pre-commit Hooks

The following checks run automatically on commit:

- **cargo fmt**: Ensures code is properly formatted
- **cargo check**: Verifies code compiles
- **cargo clippy**: Lints for common mistakes
- **cargo test**: Runs all tests

On pre-push, these additional checks run:

- **cargo doc**: Ensures documentation builds
- **cargo audit**: Checks for security vulnerabilities

You can run all hooks manually:

```bash
pre-commit run --all-files
```

### Continuous Integration

Our CI pipeline (GitHub Actions) runs on every push and PR:

- **Format check**: `cargo fmt --check`
- **Compilation**: `cargo check --all-targets --all-features`
- **Linting**: `cargo clippy -- -D warnings`
- **Tests**: `cargo test --all-features`
- **Documentation**: `cargo doc`
- **Security audit**: `cargo audit`
- **Semver check**: Ensures API compatibility
- **Unused dependencies**: `cargo-udeps`
- **License compliance**: `cargo-deny`
- **Code coverage**: `cargo-tarpaulin` (uploaded to codecov)

## Code Quality Tools

### Formatting

We use `rustfmt` with custom settings in `rustfmt.toml`:

```bash
cargo fmt
```

### Linting

We enforce clippy warnings:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Testing

Run tests with:

```bash
cargo test --all-features
```

### Documentation

Build and view docs:

```bash
cargo doc --open --all-features
```

### Security Auditing

Check for security vulnerabilities:

```bash
# Install cargo-audit if not already installed
cargo install cargo-audit

# Run audit
cargo audit
```

### Dependency Checking

Check for unused dependencies:

```bash
# Install cargo-udeps if not already installed
cargo install cargo-udeps

# Run check
cargo +nightly udeps
```

### License Compliance

We use `cargo-deny` to enforce license compliance:

```bash
# Install cargo-deny if not already installed
cargo install cargo-deny

# Check licenses, advisories, and bans
cargo deny check
```

## Development Workflow

1. **Fork and clone** the repository
2. **Create a branch** for your feature: `git checkout -b feature/my-feature`
3. **Install pre-commit hooks**: `pre-commit install`
4. **Make your changes**
5. **Write tests** for new functionality
6. **Run checks locally**:
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo doc --no-deps --all-features
   ```
7. **Commit your changes** (pre-commit hooks will run automatically)
8. **Push to your fork** and **create a Pull Request**

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat: add new search filter`
- `fix: correct pagination logic`
- `docs: update README examples`
- `test: add tests for error handling`
- `refactor: simplify builder pattern`
- `perf: optimize search query`
- `chore: update dependencies`
- `ci: improve GitHub Actions workflow`

## Changelog Generation

We use [git-cliff](https://github.com/orhun/git-cliff) for automated changelog generation:

```bash
# Install git-cliff
cargo install git-cliff

# Generate changelog
git cliff -o CHANGELOG.md

# Generate unreleased changes
git cliff --unreleased
```

## Code Style Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use descriptive variable names
- Add documentation comments for all public APIs
- Include examples in documentation
- Keep functions focused and small

### Documentation

- All public items must have documentation
- Include at least one example per public function
- Explain panics, errors, and safety concerns
- Reference related functions and types

Example:

```rust
/// Searches for jobs with the given options
///
/// Returns a single page of job search results. Use pagination parameters
/// (page, size) in SearchOptions to retrieve different pages.
///
/// # Example
///
/// ```no_run
/// use jobsuche::{Jobsuche, Credentials, SearchOptions};
///
/// let client = Jobsuche::new(
///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
///     Credentials::default()
/// )?;
///
/// let results = client.search().list(SearchOptions::builder()
///     .was("Developer")
///     .wo("Berlin")
///     .build()
/// )?;
/// ```
pub fn list(&self, options: SearchOptions) -> Result<JobSearchResponse> {
    // Implementation
}
```

### Testing

- Write unit tests for all public functions
- Include doc tests in documentation
- Test error cases
- Test edge cases (empty results, pagination, etc.)

## Release Process

1. Update version in `Cargo.toml`
2. Generate changelog: `git cliff -o CHANGELOG.md`
3. Commit version bump: `git commit -am "chore(release): prepare for v0.x.x"`
4. Tag the release: `git tag -a v0.x.x -m "Release v0.x.x"`
5. Push with tags: `git push && git push --tags`
6. Publish to crates.io: `cargo publish`

## Getting Help

- Check existing [GitHub Issues](https://github.com/wunderfrucht/jobsuche/issues)
- Ask questions in [Discussions](https://github.com/wunderfrucht/jobsuche/discussions)
- Review the [API Documentation](https://docs.rs/jobsuche)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
