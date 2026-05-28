# Testing Guide

Tests prove behavior, not line counts. Every public behavior needs a focused
test. Tests use no external services or process-specific settings.

## Test Rules

- Use `MapEnvironment` for binding tests.
- Use `ProcessEnvironment` tests only for process environment behavior.
- Do not require files, shell settings, network access, or local services.
- Prove one branch or one behavior per test.
- Keep each `#[test]` self-contained.
- Use exactly one assertion macro per test.
- Assert error results directly.
- Avoid `expect_err` followed by a second assertion.
- Assert stable `BindError::error_code()` values for error branches.
- Add tests first, then change parser behavior.
- Keep examples buildable through Cargo doc tests.

## Commands

```sh
make fmt
make lint
make test
make test-doc
make doc
make verify
```

`make verify` is the local quality gate. It runs formatting checks, Cargo
check, Clippy with warnings denied, tests, rustdoc examples, and docs.rs-style
docs.

## Coverage Expectations

The repository does not chase coverage as a number. It expects tests for:

- present values
- missing values
- explicit empty strings
- default values
- optional values
- parser failures
- environment adapter failures
- validator failures
- stable error codes
- settings composition through `ParameterSource`

New field types need tests for each relevant branch.
