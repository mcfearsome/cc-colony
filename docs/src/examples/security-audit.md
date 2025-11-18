# Security Auditing

Automated security scanning and vulnerability detection.

## Setup

### Install Template

```bash
colony template install security-auditor
```

### Configure Colony

```yaml
name: security-audit

agents:
  - id: scanner
    role: Security Auditor
    focus: OWASP Top 10, vulnerability detection
    template: security-auditor
    model: claude-opus-4-20250514
    worktree: agent/security
```

## Usage

### 1. Full Codebase Scan

```bash
colony start
```

In the scanner pane:
```
> Perform a comprehensive security audit of the entire codebase.
> Focus on OWASP Top 10 vulnerabilities.
```

### 2. Specific Module Scan

```
> Audit the authentication module (src/auth/) for security issues
```

### 3. Review New PR

```
> Review PR #45 for security vulnerabilities
> git diff main...feature/new-api
```

## What Gets Checked

The security auditor looks for:

### Injection Flaws
- SQL injection
- Command injection
- NoSQL injection
- LDAP injection

### Authentication Issues
- Weak password policies
- Insecure session management
- Missing MFA
- Credential exposure

### XSS Vulnerabilities
- Reflected XSS
- Stored XSS
- DOM-based XSS

### Access Control
- Missing authorization checks
- Insecure direct object references
- Path traversal

### Security Misconfiguration
- Default credentials
- Unnecessary features enabled
- Missing security headers
- Verbose error messages

### Cryptographic Failures
- Weak algorithms
- Hardcoded secrets
- Insecure random number generation
- Missing encryption

### Dependency Vulnerabilities
- Outdated packages
- Known CVEs
- Vulnerable dependencies

## Reports

### Generate Report

```bash
colony state memory add note \
  --content "Security audit report for v1.2.0"

# View audit findings
colony logs scanner --level warn
```

### Export Findings

```bash
colony logs scanner --json > security-report.json
```

## Integration with CI/CD

### Pre-Commit Hook

```bash
# .git/hooks/pre-commit
#!/bin/bash
colony start --no-attach
colony state task add "Security scan staged files"
# Wait for scan completion
colony logs scanner --level error
```

### GitHub Actions

```yaml
name: Security Audit
on: [pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run Security Audit
        run: |
          colony init --template security-auditor
          colony start --no-attach
          colony logs scanner --json > audit.json
      - name: Upload Report
        uses: actions/upload-artifact@v2
        with:
          name: security-report
          path: audit.json
```

## See Also

- [Templates](../concepts/templates.md) - Security auditor template details
- [Code Review](./code-review.md) - Add security review to workflow
