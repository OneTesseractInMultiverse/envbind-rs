# Security Policy

## Reporting a Vulnerability

Please report suspected vulnerabilities privately to
security@subvertic.com.

Include:

- affected version
- impact
- reproduction steps
- suggested fix, if known

Do not open a public issue for a vulnerability until a fix or disclosure plan
is ready.

## Security Expectations

Envbind reads environment variables that can contain credentials or private
deployment settings. Changes must not disclose raw environment values in logs,
docs, panics, or error messages.

The library treats values as sensitive by default. It redacts custom validation
details until a field is marked non-sensitive. It uses size limits for raw,
JSON, base64, and list inputs.
