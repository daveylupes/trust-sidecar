# Contributing to Trust Sidecar

Thank you for your interest in contributing to Trust Sidecar! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing Guidelines](#testing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Areas for Contribution](#areas-for-contribution)
- [Getting Help](#getting-help)

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors. We are committed to providing a welcoming and harassment-free experience for everyone.

## How to Contribute

### Reporting Issues

If you find a bug or have a feature request:

1. **Check existing issues**: Search the [Issues](https://github.com/daveylupes/trust-sidecar/issues) to see if the issue already exists
2. **Create a new issue** with:
   - **Clear title**: Brief description of the issue
   - **Description**: Detailed explanation
   - **Steps to reproduce**: For bugs, include exact steps
   - **Expected vs actual behavior**: What should happen vs what actually happens
   - **Environment**: OS, Rust version, browser (if applicable)
   - **Screenshots/logs**: If relevant

**Issue Templates:**
- Use "Bug Report" template for bugs
- Use "Feature Request" template for new features
- Use "Documentation" template for docs improvements

### Security Issues

**WARNING: Do NOT open a public issue for security vulnerabilities.**

Instead:
- Contact [@daveylupes](https://x.com/daveylupes) on X (Twitter) via DM
- Or use [GitHub Security Advisories](https://github.com/daveylupes/trust-sidecar/security/advisories)

We take security seriously and will respond promptly.

### Contributing Code

#### Step 1: Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/trust-sidecar.git
cd trust-sidecar

# Add upstream remote
git remote add upstream https://github.com/daveylupes/trust-sidecar.git
```

#### Step 2: Create a Branch

```bash
# Create a feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/your-bug-fix

# Or for documentation
git checkout -b docs/your-doc-update
```

**Branch Naming:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions/updates

#### Step 3: Make Your Changes

- Follow Rust style guidelines (see [Code Style Guidelines](#code-style-guidelines))
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass: `cargo test`
- Run linters: `cargo clippy` and `cargo fmt`

#### Step 4: Commit Your Changes

```bash
git add .
git commit -m "Description of your changes"
```

**Commit Message Guidelines:**
- Use clear, descriptive messages
- Start with a verb (Add, Fix, Update, Remove, etc.)
- Reference issue numbers if applicable: `Fix #123`
- Keep first line under 72 characters
- Add detailed description if needed

**Examples:**
```
Add SD-JWT proof generation support

Implements selective disclosure proof generation using bh-sd-jwt crate.
Fixes #45
```

```
Fix proof verification timestamp check

The timestamp validation was incorrectly rejecting valid proofs.
Now correctly checks for 24-hour window.
```

#### Step 5: Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then:
1. Go to GitHub and create a Pull Request
2. Fill out the PR template:
   - Description of changes
   - Related issues (use "Closes #123" to auto-close)
   - Testing performed
   - Screenshots (if UI changes)
3. Request review from maintainers
4. Address any feedback

## Development Setup

### Prerequisites

- **Rust 1.90+**: Check with `rustc --version`
- **Cargo**: Comes with Rust installation
- **Git**: For version control

### Initial Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/trust-sidecar.git
cd trust-sidecar

# Build the project
cargo build

# Run tests
cargo test

# Run the server
cargo run
```

### Development Workflow

```bash
# 1. Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# 2. Create feature branch
git checkout -b feature/my-feature

# 3. Make changes and test
cargo test
cargo clippy
cargo fmt

# 4. Commit and push
git add .
git commit -m "Add my feature"
git push origin feature/my-feature

# 5. Create PR on GitHub
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test
```

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run specific command with logging
RUST_LOG=debug cargo run -- generate-did --service-id test
```

## Code Style Guidelines

### Rust Style

1. **Format code**: Always run `cargo fmt` before committing
   ```bash
   cargo fmt
   ```

2. **Check for issues**: Run `cargo clippy` to catch common problems
   ```bash
   cargo clippy -- -D warnings
   ```

3. **Follow Rust conventions**:
   - Use `snake_case` for functions and variables
   - Use `PascalCase` for types
   - Use `SCREAMING_SNAKE_CASE` for constants
   - Prefer `&str` over `String` for function parameters when possible

4. **Documentation**:
   - Add doc comments for all public APIs
   - Use `///` for module-level and item docs
   - Include examples in doc comments when helpful
   - Run `cargo doc` to generate documentation

**Example:**
```rust
/// Generate a proof of view for Ad Tech.
///
/// Creates a cryptographic proof that a specific content was viewed by a DID.
///
/// # Arguments
/// * `viewer_did` - The DID of the viewer
/// * `content_id` - URL or identifier of the viewed content
///
/// # Returns
/// A `ProofOfView` structure with cryptographic proof
///
/// # Example
/// ```
/// let proof = protocol_manager.generate_proof_of_view(
///     "did:key:z6Mk...",
///     "https://example.com/article",
///     None
/// ).await?;
/// ```
pub async fn generate_proof_of_view(
    &self,
    viewer_did: &str,
    content_id: &str,
    credential_proof: Option<&str>,
) -> Result<ProofOfView, Box<dyn Error>> {
    // Implementation
}
```

### JavaScript Style (Browser Extension)

- Use modern ES6+ syntax
- Use `const` and `let` (avoid `var`)
- Use async/await for promises
- Add JSDoc comments for functions
- Follow existing code style

## Testing Guidelines

### Unit Tests

- Write tests for all new functionality
- Test edge cases and error conditions
- Use descriptive test names
- Group related tests in modules

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_proof_of_view() {
        let manager = ProtocolManager::new();
        let proof = manager.generate_proof_of_view(
            "did:key:test",
            "test-content",
            None
        ).await.unwrap();
        
        assert_eq!(proof.viewer_did, "did:key:test");
        assert_eq!(proof.content_id, "test-content");
    }
}
```

### Integration Tests

- Test API endpoints end-to-end
- Test browser extension integration
- Test error handling
- Test with real data when possible

### Manual Testing

- Test with provided test pages
- Test with mock ad network server
- Test CLI commands
- Test browser extension on different browsers

## Pull Request Process

### Before Submitting

1. Code follows style guidelines (`cargo fmt`, `cargo clippy`)
2. All tests pass (`cargo test`)
3. Documentation updated (if needed)
4. No new warnings or errors introduced by your change
5. Commit messages are clear

**Note:** `cargo clippy --all-targets` currently reports a handful of pre-existing warnings elsewhere in the codebase — you don't need to fix those in an unrelated PR, just don't add to them. Run clippy before and after your change and diff the warning count if you're unsure.

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] Tests added/updated for new functionality
- [ ] Documentation updated
- [ ] All tests pass
- [ ] No *new* clippy warnings (see note above re: pre-existing ones)
- [ ] Commit messages are descriptive
- [ ] PR description is clear
- [ ] Related issues referenced

### Review Process

1. **Automated checks**: CI will run tests and linters
2. **Code review**: Maintainers will review your code
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, maintainers will merge

### After Merge

- Your contribution will be credited in the project
- Thank you for contributing!

## Areas for Contribution

We welcome contributions in these areas:

### High Priority

- **Full SD-JWT Implementation**: Using `bh-sd-jwt` crate for zero-knowledge proofs, including holder-binding for `SpendAuthorizationClaims`
- **Complete DIDComm Messaging**: Full message sending/receiving implementation
- **Key Recovery Mechanisms**: Social recovery, key sharding
- **DID-to-Public-Key Resolution**: verification currently requires the caller to already have the issuer's public key out-of-band
- **Spend Authorization Revocation & Cumulative Limits**: currently only a per-transaction cap with no way to invalidate a credential early
- **Agent-Commerce Protocol Adapters**: bridging `SpendAuthorizationClaims`/`ProofOfAction` to external protocols like Google AP2 or Visa's Trusted Agent Protocol — see [ARCHITECTURE.md](ARCHITECTURE.md#agent-commerce-landscape)
- **Performance Optimizations**: Improve latency, reduce memory usage

### Medium Priority

- **Additional DID Methods**: Support for other DID methods beyond `did:key`
- **Browser Extension Enhancements**: New features, better UX
- **Test Coverage**: More comprehensive unit and integration tests
- **Documentation**: Improve docs, add examples, tutorials

### Nice to Have

- **WASM Compilation**: Compile core to WebAssembly for browser use
- **SDK Wrappers**: Python/JavaScript SDKs
- **Example Applications**: Demo apps showing usage
- **Tutorials and Guides**: Step-by-step tutorials

### Good First Issues

Look for issues labeled `good-first-issue` for beginner-friendly contributions:
- Documentation improvements
- Test additions
- Small bug fixes
- Code cleanup

## Project Structure

```
trust-sidecar/
├── src/
│   ├── main.rs           # Main entry point & API server
│   ├── cli.rs            # CLI interface
│   ├── security.rs       # Input validation, rate-limit config, error sanitization
│   ├── identity/         # DID generation & key management
│   │   └── mod.rs
│   ├── messaging/        # DIDComm encrypted messaging
│   │   └── mod.rs
│   └── protocols/        # Credentials, proofs, and agent authorization
│       ├── mod.rs             # ProtocolManager, ProofOfView, JWT issue/verify
│       ├── credentials.rs     # Typed credential wrapper (TypedCredential<T>)
│       ├── agent_auth.rs      # SpendAuthorization credentials & transaction checks
│       └── proof_of_action.rs # ProofOfAction (generalized proof-of-view)
├── browser-extension/    # Browser extension
│   ├── manifest.json
│   ├── background.js
│   ├── content.js
│   └── ...
├── ad-network-server/   # Mock ad network for testing
├── Cargo.toml
└── README.md
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for what each module actually does.

## Getting Help

### Questions?

- **GitHub Discussions**: [Ask questions](https://github.com/daveylupes/trust-sidecar/discussions)
- **Issues**: [Report bugs or request features](https://github.com/daveylupes/trust-sidecar/issues)
- **X (Twitter)**: [@daveylupes](https://x.com/daveylupes)

### Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Axum Documentation](https://docs.rs/axum/)
- [Affinidi TDK Documentation](https://github.com/affinidi/affinidi-tdk-rs)

## Recognition

Contributors will be:
- Listed in the project README (if desired)
- Credited in release notes
- Thanked in the project

Thank you for contributing to Trust Sidecar! Your contributions help make this project better for everyone.

---

**Ready to contribute?** Start by forking the repository and checking out the [Getting Started Guide](GETTING_STARTED.md)!
