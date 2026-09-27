# Security Policy

## Supported Versions

The latest published minor line receives vulnerability fixes.

| Version | Status |
| --- | --- |
| `0.2.x` | Upcoming release line; not yet published |
| `0.1.x` | Latest published line; reviewed fixes are prepared in 0.2.0 |

The current published version is 0.1.0. The repository's 0.2.0 validation and
fixes do not retroactively apply to that package. On publication, 0.2.x becomes
the supported minor line; consumers should follow the
[migration guide](docs/migrating-to-0.2.md).

## Reporting a Vulnerability

Please report suspected vulnerabilities privately to
security@subvertic.com.

Include the affected version, impact, reproduction steps, and suggested fix if
known. Do not open a public issue for a vulnerability until a fix or disclosure
plan is ready.

## Dependency Monitoring and Response

Pedro Guzmán (`@OneTesseractInMultiverse`) owns dependency advisory triage and
security fixes. Dependabot alerts and automatic security-update pull requests
are enabled for supported dependencies. Version-update checks continue weekly;
dependency changes require review and the normal CI checks before merging.

The `Security audit` job runs on pull requests, pushes to `main`, and daily at
07:23 UTC against the default branch. A manual audit-only mode runs the same
job. It fetches the RustSec advisory database and checks fresh MSRV-compatible
and stable dependency resolutions plus the committed fuzz lockfile. Both CI and
release validation reject vulnerabilities and all cargo-audit warnings,
including unmaintained, unsound, notice, and yanked-package findings. Database
fetch and dependency-resolution failures also fail the check.

There are currently no advisory exceptions. Any proposed exception needs a
reviewed pull request identifying the advisory, affected versions and graphs,
why the application is unaffected or how risk is mitigated, an accountable
owner, an upstream fix or replacement plan, and an expiry of at most 30 days.
Review exceptions at least weekly and before a release. An exception must
include an automated expiry check that fails CI when overdue; do not add a
bare ignore flag or silently suppress warnings.

The maintainer's existing GitHub notification settings were verified on
2026-09-27: failed workflow runs notify by email, and new Dependabot alerts
notify through GitHub, email, and the CLI. Scheduled-workflow notifications
follow the schedule's author or the user who last changed its cron expression
or re-enabled it. Reverify ownership and notification settings when either
changes. This verifies the configured route; it does not establish email
delivery or an operational response-time guarantee.

On a failure, the maintainer should:

1. Inspect the failed run and its retained lockfiles/reports. Distinguish an
   advisory from an unavailable registry/database or other execution failure;
   rerun infrastructure failures and keep the check failing until resolved.
2. Assess all affected graphs and supported releases, including the exact
   retained release lockfile. Prioritize exploitable vulnerabilities over
   maintenance warnings, but resolve or formally review every finding.
3. Prepare a dependency update, replacement, or mitigation; run the MSRV,
   stable, advisory, and release-validation checks. Coordinate any undisclosed
   vulnerability through security@subvertic.com and publish a fix/disclosure
   when ready.
4. Confirm that the updated graph audits cleanly and the corresponding alert
   is resolved. Document any dismissal with evidence and an owner; a dismissal
   is not a substitute for the exception process above.

Review scheduled-run history at least weekly. If no daily audit has completed
in 48 hours, investigate and run the manual audit. Hosted schedules may be
delayed, and GitHub disables scheduled workflows in inactive public
repositories after 60 days. Re-enable a disabled schedule and verify its owner.

See the [audit runbook](docs/dependencies.md#ongoing-advisory-checks) for commands,
release-lockfile scope, evidence retention, and repository-control verification.
Secret scanning and push protection remain enabled; the private disclosure
address above remains the reporting channel.

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
