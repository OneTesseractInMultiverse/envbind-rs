# Security Policy

## Supported Versions

The latest published minor line receives vulnerability fixes.

| Version | Status |
| --- | --- |
| `0.1.x` | Supported |

## Reporting a Vulnerability

Please report suspected vulnerabilities privately to
security@subvertic.com.

Include the affected version, impact, reproduction steps, and suggested fix if
known. Do not open a public issue for a vulnerability until a fix or disclosure
plan is ready.

## Security Expectations

Envbind reads environment variables that can contain credentials or private
deployment settings. Changes must not disclose raw environment values in logs,
docs, panics, or error messages.

The library treats binding diagnostics as sensitive by default. It replaces
validator-provided details with a generic validation message. Marking a field
with `.sensitive(false)` retains those details in the structured error and its
display/debug output; callbacks must not include private data in those messages.
It uses size limits for raw, JSON, base64, and list inputs.

Built-in parse errors name the variable and failure class without echoing input.
Variable names are visible error context and must not contain secrets.
Adapter read errors keep custom messages for structured handling, but display
text stays generic and debug output redacts them. This applies to alternate
debug formatting, nested binding errors, and formatting through `Error::source()`.
These adapter guarantees are independent of the field's sensitivity setting.
Explicit access to the stored adapter message can expose sensitive data.

Successfully bound values are ordinary Rust values. `.sensitive(...)` does not
redact application settings, `Debug` derives, serialization, logging, assertion
output, or direct access to those values. Applications must review their own
output and avoid whole-settings dumps. Credential-bearing settings should omit
derived `Debug` or implement explicitly redacted formatting. See the
[application logging example](docs/api-guide.md#sensitivity-and-application-logging).

Cloning, retention, and memory handling remain the application's responsibility.
Envbind does not provide automatic zeroization or secret-memory protection.
