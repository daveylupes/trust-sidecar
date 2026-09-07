# Security Policy

## Supported Versions

We currently support the following versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please **do not** open a public issue. Instead, please report it via one of the following methods:

1. **X (Twitter)**: [@daveylupes](https://x.com/daveylupes) - Send a DM
2. **Private Security Advisory**: [GitHub Security Advisories](https://github.com/daveylupes/trust-sidecar/security/advisories)

Please include the following information:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond to security reports within 48 hours and work with you to address the issue before making it public.

## Security Best Practices

### For Users

1. **Private Keys**: Never share your private keys or DIDs. They are stored in your OS keychain and should remain private.

2. **Network Security**: The default configuration binds to `127.0.0.1` (localhost only). Only expose the API to trusted networks.

3. **Credential Storage**: Credentials are stored in-memory by default. For production use, ensure proper persistence and encryption.

4. **Dependencies**: Keep dependencies up to date. Run `cargo update` regularly.

### For Developers

1. **Key Management**: Always use the OS keychain for storing secrets. Never hardcode keys in source code.

2. **Input Validation**: Validate all user inputs, especially in credential verification endpoints.

3. **Error Messages**: Avoid exposing sensitive information in error messages.

4. **Dependencies**: Review and audit dependencies regularly using `cargo audit`.

## Known Security Considerations

1. **Key Recovery**: Currently, there is no key recovery mechanism. Losing your private key means losing access to your identity.

2. **In-Memory Storage**: Some credentials are stored in-memory. This is acceptable for development but should be persisted securely in production.

3. **SD-JWT Implementation**: The current SD-JWT implementation is basic. Full SD-JWT support is planned for future releases.

4. **No authentication on the API**: none of the HTTP endpoints require authentication. This is fine for local (`127.0.0.1`) use, but `serve --host` will happily bind to `0.0.0.0` with no warning and no auth layer — don't do that on an untrusted network.

5. **Agent spend-authorization credentials have no revocation**: a `SpendAuthorization` credential (see [API.md](API.md#issue-spend-authorization)) is valid until its expiry regardless of what happens after issuance — there's no way to invalidate one early if an agent is compromised. It also enforces only a per-transaction cap, not a cumulative spending limit across transactions. Both are documented, deliberate scope limits for the current phase, not oversights — but treat any authorization you issue as valid for its full stated window.

## Security Updates

Security updates will be released as patch versions (e.g., 0.1.1, 0.1.2) and will be clearly marked in release notes.

## Security Review

This codebase has gone through an internal code review — not a third-party or professional security audit. Issues identified at critical and high severity have been fixed; a few lower-severity items (some dead code from unfinished phases, a couple of browser-extension hardening nits, general lint cleanliness) remain open.

If you're evaluating this project for a security-sensitive use case, we encourage independent review rather than relying on this statement alone.

