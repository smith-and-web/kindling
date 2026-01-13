# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |

As Kindling is in early development, we support the latest version only.

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

1. **Do NOT open a public issue** for security vulnerabilities
2. Email us at **smithandweb+security@gmail.com**
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes (optional)

### What to Expect

- **Acknowledgment**: Within 48 hours of your report
- **Initial Assessment**: Within 1 week
- **Resolution Timeline**: Depends on severity and complexity
- **Credit**: We'll credit you in the release notes (unless you prefer anonymity)

### Scope

Security issues we're interested in:

- Local file access beyond intended scope
- Arbitrary code execution
- Data corruption or loss
- Privacy leaks (unintended data exposure)

### Out of Scope

- Issues requiring physical access to the user's machine
- Social engineering attacks
- Issues in dependencies (report these upstream, but let us know)

## Security Best Practices

Kindling is a desktop application that:

- Stores data locally in SQLite databases
- Reads project files from user-specified locations
- Does not transmit data over the network (in current version)

### For Users

- Download Kindling only from official sources (GitHub Releases)
- Verify checksums when available
- Keep your operating system updated
- Be cautious when importing files from untrusted sources

## Acknowledgments

We appreciate the security research community. Contributors who responsibly disclose vulnerabilities will be acknowledged here.

---

Thank you for helping keep Kindling secure!
