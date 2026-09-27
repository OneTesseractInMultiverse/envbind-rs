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
| `make test-doc` | Test API, README, and guide examples. |
| `make doc` | Build docs.rs-style docs. |
| `make verify` | Run the local quality gate. |

`make verify` is the main local gate. It runs formatting checks, Cargo check,
Clippy, tests, rustdoc examples, and docs.rs-style docs.

## Documentation Tests

`make test-doc` runs `cargo test --doc`. Alongside API documentation, rustdoc
reads `README.md` and every Markdown guide under `docs/` through
[`src/documentation.rs`](../src/documentation.rs). Each file is attached to a
private module using `#[doc = include_str!(...)]`. The crate includes that
module only under `#[cfg(doctest)]`, so it adds no public API or rendered
documentation pages. Snippets stay in their published Markdown source; there
are no separate copies of their tested code.

The existing `make verify` gate and the CI `Test docs` step run these same
checks. CI executes them on Rust 1.85 and stable Linux, macOS, and Windows.
Use `make test-doc CARGO='cargo +1.85.0'` to select the minimum compiler locally.
To focus on the included Markdown or list discovered snippets:

```sh
cargo test --doc documentation::
cargo test --doc -- --list
```

The gate currently discovers 33 snippets: 5 API examples and 28 from
Markdown (9 README, 15 API guide, 1 architecture, 1 testing guide, and 2 migration
guide examples). Two process-loading examples compile without running; the
other 31 compile and execute. No Rust examples are ignored. Counts can grow with the documentation;
the other registered guides currently contain only non-Rust code blocks.

The credential-settings example checks normal and alternate `Debug` output
against a synthetic token. Its application-owned formatter deliberately omits
all values; the bound token itself remains an ordinary string.

Follow these rules when editing examples:

- Use `rust` fences for executable examples. Prefer `MapEnvironment` and
  synthetic values so assertions run without external configuration or services.
- Give each snippet its own imports and setup. Rustdoc compiles snippets
  independently; an earlier block's definitions are not in scope. Lines starting
  with `# ` supply hidden rustdoc setup. The process-loading snippets use a
  hidden generic function bounded by `ParameterSource`, so the method is checked
  without duplicating a settings implementation.
- Use `rust,no_run` only when execution needs an external boundary, and explain
  the reason in adjacent prose. The README and API guide process-loading calls
  are compile-only because they read arbitrary process configuration. The API
  guide's invalid-name example executes: it rejects the name before any lookup.
  Real process loading remains covered by the isolated subprocess tests below.
- Use a non-Rust language fence such as `text`, `sh`, or `toml` for pseudocode
  or other languages. Do not mark working Rust examples `ignore` to hide errors.
  Use `compile_fail` only to demonstrate an intentional compile-time rejection,
  with an explanation of the rule being tested.
- Register every new Markdown guide in `src/documentation.rs`, even if it has
  no Rust examples yet. The workflow test suite checks this inventory and that
  CI does not skip documentation tests on any Rust matrix entry.

An API typo in any checked Rust snippet must make `make test-doc` fail, including
within a `no_run` block. A failing assertion in an executable snippet must also
fail the gate. To verify the wiring after changing the harness, temporarily
misspell a method in a registered guide, run the command, confirm a nonzero
exit and the Markdown filename in the diagnostic, and restore the method before
committing. The Rustdoc Book documents
[doctest-only includes and snippet attributes](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html).

## Process Environment Tests

`tests/environment.rs` checks malformed process names through required,
defaulted, and optional bindings and verifies that map and custom adapters keep
their own key namespace. These invalid-name reads do not modify process state.

The `process_environment` test target launches six child-process scenarios on
the supported Unix and Windows targets. Each starts with `Command::env_clear`
and receives only its synthetic fixture variables through `Command::env`:

- Valid missing names, Unicode and non-shell names, whitespace preservation,
  and native case matching.
- Present, explicitly empty, and Unicode values, including default, optional,
  and `allow_empty()` behavior.
- Successful `ParameterSource::from_process_environment()` composition of
  string, integer, boolean, defaulted, and optional fields.
- A failing field during production settings loading; the actual error returned
  from `main` must identify the field without printing synthetic secret values.
- Invalid-Unicode adapter reads remaining errors through defaults,
  `OptionalStringVar`, and generic optional wrappers, including typed fields.
- A production settings read failure with invalid Unicode; termination output
  must contain the safe error variant and no raw value.

Unix fixtures construct a non-UTF-8 value with `OsStringExt::from_vec`. Windows
fixtures construct an unpaired UTF-16 surrogate with `OsStringExt::from_wide`.
Source `Display`, `Debug`, and alternate `Debug` formatting are checked too.
No child mutates its environment after launch, and the parent never changes
its own environment. The custom harness uses safe code and runs from
`cargo test --all-targets`; the child mode flags are internal implementation
details. Run the focused tests with:

```sh
cargo test --test environment --test process_environment
```

## Platform CI and Required Checks

CI runs the full Rust suite on current stable Rust for Linux, macOS, and Windows,
plus Rust 1.85 on Linux. See [Support](../SUPPORT.md#supported-platforms) for the
tested targets and the OS-version boundary. The job logs include the compiler's
host target; no cross-compilation is used for these tests.

The required `Rust 1.85.0` and `Security audit` check names are unchanged.
`Rust stable` is now an aggregate gate that depends on the entire Rust matrix,
including the MSRV. It uses `always()` and explicitly rejects every result
except `success`, so failed, cancelled, or skipped platform jobs cannot satisfy
the required check. Individual stable jobs are named `Rust stable (Linux)`,
`Rust stable (macOS)`, and `Rust stable (Windows)`. The existing main-branch
ruleset therefore gates every platform without a settings change.

The Python workflow suite also checks platform coverage, the MSRV check name,
and the aggregate gate's behavior for success and unsuccessful results. These
local checks execute the actual gate script; they do not dispatch workflows.

Scheduled and explicit manual audit-only runs skip the build matrix and its
aggregate gate. They execute the same `Security audit` job used by full CI.
Workflow regression tests cover trigger routing, concurrency isolation,
read-only permissions, time limits, fresh resolutions, warning policy, and
failure propagation through the report pipeline. Synthetic command failures
verify that each graph can fail the job and that resolution failures stop the
audit. See the [audit runbook](dependencies.md#ongoing-advisory-checks).

## Release Workflow Tests

The separate `Release workflow tests` CI job runs 77 regression tests covering
the publishing workflow, CI matrix, documentation inventory, security audits,
fuzz runner, and release-control preflight. Run it locally from the full Git
checkout with Python 3.11 or newer, Git, and an installed stable Rust toolchain:

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

The same test command checks the release-control preflight against local API
fixtures. It covers missing approval rules, administrator bypass, reviewer
changes, branch/tag restrictions, release-tag exceptions and bypass actors,
main protection drift, and API access failures. It never changes repository
settings. Live approval-gate verification is a separate maintainer procedure
using the token-free fixture in the [release control runbook](release-controls.md).

PyYAML 6.0.3 is a pinned, MIT-licensed test dependency used to read the actual
workflow YAML. It has no runtime dependencies and is excluded from the Rust
crate package. Maintain its pin with the other CI tooling dependencies.

Also run `actionlint` from the repository root when changing workflow YAML.
Install ShellCheck alongside it to include shell diagnostics. This is a
separate local lint check; `make verify` runs the Rust gates only. The published
crate excludes `.github/` and `fuzz/`, so workflow and fuzz commands require the
full repository checkout.

## Coverage Expectations

The normal gate also runs fixed-seed property tests. See
[property tests and bounded fuzzing](fuzzing.md) for the shared invariants,
input bounds, isolated nightly targets, seed corpus, CI budgets, and failure
reproduction/retention procedure.

A [binding contract matrix](binding-contract-matrix.md) maps every field type
to its common contracts, parser cases, defensive boundaries, and composition
tests. It also records intentionally unsupported combinations, the fallback
policy, and a reproducible coverage-inspection command.

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
