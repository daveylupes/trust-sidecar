# Contributing to Trust Sidecar

Thank you for your interest in contributing to Trust Sidecar! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## How to Contribute

### Reporting Issues

If you find a bug or have a feature request:

1. Check if the issue already exists in the [Issues](https://github.com/daveylupes/trust-sidecar/issues) section
2. If not, create a new issue with:
   - Clear title and description
   - Steps to reproduce (for bugs)
   - Expected vs actual behavior
   - Environment details (OS, Rust version, etc.)

### Security Issues

**Do not** open a public issue for security vulnerabilities. Instead:
- Contact [@daveylupes](https://x.com/daveylupes) on X (Twitter) via DM
- Or use [GitHub Security Advisories](https://github.com/daveylupes/trust-sidecar/security/advisories)

### Contributing Code

1. **Fork the repository**
   ```bash
   git clone https://github.com/daveylupes/trust-sidecar.git
   cd trust-sidecar
   ```

2. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

3. **Make your changes**
   - Follow Rust style guidelines (run `cargo fmt`)
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass: `cargo test`

4. **Commit your changes**
   ```bash
   git add .
   git commit -m "Description of your changes"
   ```
   - Write clear, descriptive commit messages
   - Reference issue numbers if applicable (e.g., "Fix #123")

5. **Push and create a Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```
   - Go to GitHub and create a Pull Request
   - Fill out the PR template with:
     - Description of changes
     - Related issues
     - Testing performed

## Development Setup

### Prerequisites

- Rust 1.90+ (check with `rustc --version`)
- Cargo (comes with Rust)

### Building

```bash
# Clone the repository
git clone https://github.com/daveylupes/trust-sidecar.git
cd trust-sidecar

# Build the project
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` to check for common issues
- Follow Rust naming conventions
- Add documentation comments for public APIs

### Testing

- Write unit tests for new functionality
- Ensure all existing tests pass
- Test edge cases and error conditions
- Update integration tests if adding new endpoints

## Project Structure

```
trust-sidecar/
├── src/
│   ├── main.rs           # Main entry point & API server
│   ├── cli.rs            # CLI interface
│   ├── identity/         # DID generation & key management
│   ├── messaging/        # DIDComm encrypted messaging
│   └── protocols/        # SD-JWT & credential verification
├── Cargo.toml
└── README.md
```

## Areas for Contribution

We welcome contributions in these areas:

### High Priority
- Full SD-JWT implementation using `bh-sd-jwt` crate
- Complete DIDComm message sending/receiving
- Key recovery mechanisms
- Performance optimizations

### Medium Priority
- Additional DID methods support
- Browser extension integration
- More comprehensive test coverage
- Documentation improvements

### Nice to Have
- WASM compilation support
- Python/JavaScript SDK wrappers
- Example applications
- Tutorials and guides

## Pull Request Process

1. Ensure your code follows the project's style guidelines
2. Update documentation for any API changes
3. Add tests for new functionality
4. Ensure all tests pass and there are no warnings
5. Request review from maintainers
6. Address any feedback
7. Once approved, maintainers will merge your PR

## Questions?

- Open a [Discussion](https://github.com/daveylupes/trust-sidecar/discussions) for questions
- Contact [@daveylupes](https://x.com/daveylupes) on X (Twitter)

Thank you for contributing to Trust Sidecar!

