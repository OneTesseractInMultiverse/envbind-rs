# Testing Guide

Tests prove public behavior. They do not exist to raise a coverage number.
Each test states one fact about the crate, and the test data lives in the test
itself.

Envbind tests favor `MapEnvironment`. This keeps tests deterministic and avoids
process state. `ProcessEnvironment` appears only in tests about process
environment behavior.

## Test Rules

Use one assertion macro per test. This rule keeps failure output direct. It
forces the test to name the single behavior under review.

Build error values directly in tests. Assert `BindError::error_code()` for
error branches, since those codes form part of the public contract. Avoid
`expect_err` followed by a second assertion. Map the result into the exact value
the test needs, then assert once.

Add tests first, then change parser behavior. A parser change affects startup
failure modes, so the intended branch must be visible in the test suite.

Examples must build through normal Cargo doc tests. Unit tests must not require
external files, shell settings, network access, local services, or host-specific
environment variables.

## Commands

| Command | Purpose |
| --- | --- |
| `make fmt` | Format Rust code. |
| `make lint` | Run Clippy with warnings denied. |
| `make test` | Run target tests. |
| `make test-doc` | Run rustdoc examples. |
| `make doc` | Build docs.rs-style docs. |
| `make verify` | Run the local quality gate. |

`make verify` is the main local gate. It runs formatting checks, Cargo check,
Clippy, tests, rustdoc examples, and docs.rs-style docs.

## Release Workflow Tests

The separate `Release workflow tests` CI job checks the scripts and security
controls in `.github/workflows/publish.yml`. Run it locally with Python 3.11
or newer, Git, and an installed stable Rust toolchain:

```sh
python3 -m venv /tmp/envbind-workflow-tests
/tmp/envbind-workflow-tests/bin/python3 -m pip install -r .github/tests/requirements.txt
/tmp/envbind-workflow-tests/bin/python3 -B -m unittest discover -s .github/tests -v
```

These integration tests create temporary local repositories and execute the
scripts extracted from the workflow. They cover tag syntax, shell-injection
payloads, manifest version matching, annotated and lightweight tags, missing
tags, checkout mismatches, and the validation-only publishing gate. Release
handoff fixtures move a tag after validation, verify the pinned commit, retain
the lockfile, and rebuild dependency-free crates using real Cargo in offline
mode. They also reject modified metadata, locks, audits, and archives, and
verify that the final integrity check blocks publication on drift. The upload
command is replaced with a local recorder for the final invocation tests.
Tests never dispatch workflows, contact a registry, or publish a package. Git
identity/configuration and Cargo caches are isolated from developer settings.

PyYAML 6.0.3 is a pinned, MIT-licensed test dependency used to read the actual
workflow YAML. It has no runtime dependencies and is excluded from the Rust
crate package. Maintain its pin with the other CI tooling dependencies.

## Coverage Expectations

A field type needs tests for present values, missing values, explicit empty
strings, defaults, optional behavior, parser failures, adapter failures,
validator failures, stable error codes, and composition through
`ParameterSource`.

New behavior needs a self-contained test. The test data belongs in the test
body. Shared helpers stay small and local to the behavior under test.

## Example Pattern

This pattern keeps one assertion and checks the error branch:

```rust
# use envbind::{Binder, MapEnvironment, U16Var};
let value = Binder::new(MapEnvironment::from_pairs([("PORT", "bad")]))
    .bind(&U16Var::new("PORT"))
    .map_err(|error| error.error_code());

assert_eq!(value, Err("parse_variable"));
```
