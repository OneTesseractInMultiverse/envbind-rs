# Open Source Practices

Envbind is easy to inspect, test, and change. The repository keeps policy files
near the code, so contributors can find project rules without searching
external systems.

`LICENSE-MIT` contains the license text. `CONTRIBUTING.md` explains the
contribution process. `SECURITY.md` gives the private report channel for
security issues. `CODE_OF_CONDUCT.md` sets conduct rules. `CHANGELOG.md`
records user-facing releases.

## Review Standards

Pull requests stay small enough for careful review. A good contribution states
the behavior change, the reason the change belongs in this crate, the tests
that prove the behavior, and any public API or error contract change.

Review focuses on correctness, clear boundaries, tests, and documentation. A
change that mixes environment reads with parsing needs revision. A change that
prints raw values needs revision.

## Dependency Policy

Runtime dependencies stay narrow. A dependency must support concrete parser or
validator behavior. Simple parsing and validation use standard library code.

A dependency needs clear maintenance value. Review the license, release
activity, transitive tree, and RustSec status before adoption. CI runs a
dependency advisory audit so known vulnerable releases fail the gate.

## Security Policy

Do not log raw environment values by default. Users can store credentials or
private deployment state in environment variables.

Error messages name the variable and failure class. They do not echo the raw
value. Validation details stay hidden for sensitive fields.
