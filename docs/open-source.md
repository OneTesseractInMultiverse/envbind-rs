# Open Source Practices

Envbind is easy to inspect, test, and change.

## Repository Files

- `LICENSE-MIT` contains the MIT license text.
- `CONTRIBUTING.md` explains contribution rules.
- `SECURITY.md` explains private security reports.
- `CODE_OF_CONDUCT.md` sets conduct rules.
- `README.md` and `/docs` describe user-facing behavior.
- `CHANGELOG.md` tracks releases.

## Review Standards

Pull requests stay small enough for careful review. A good contribution
explains:

- what behavior changed
- why the change belongs in this crate
- which tests prove the behavior
- whether public API, docs, or error contracts changed

## Dependency Policy

Keep runtime dependencies narrow. Add a dependency only for concrete parser or
validator behavior.

Add a dependency only when it removes real maintenance work. Prefer standard
library code for simple parsing and validation.

## Security Policy

Do not log raw environment values by default. Users can store credentials or
private deployment state in environment variables.

Error messages name the variable and failure class. They do not echo the raw
value.
