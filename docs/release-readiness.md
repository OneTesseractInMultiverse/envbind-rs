# Release Readiness: 0.2.0

This record covers the consolidated changes tracked in
[#23](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/23).
Version 0.2.0 is prepared but **not published**. The current registry package is
0.1.0; neither its source nor its dependencies inherit the results of checks on
the new candidate. A production deployment must select the reviewed version
and follow the [migration guide](migrating-to-0.2.md).

## Release Boundary

The supported compiler baseline is Rust 1.85 on Linux plus current stable Rust
on native Linux GNU, macOS Apple silicon, and Windows MSVC. See
[Support](../SUPPORT.md#supported-platforms) for exact targets and the boundary
around older operating systems, other architectures, and other C libraries.

The documented API covers startup binding from map, process, and custom
environment adapters. Bindings use the field-specific empty, optional,
default, validation, and size-limit policies in the
[contract matrix](binding-contract-matrix.md). Stable error codes and sensitivity
scope are documented in the [API guide](api-guide.md). Returned values,
application logs, custom callback behavior, and memory handling remain the
application's responsibility. Defaults and finite-float validation remain
explicit choices; 0.2.0 does not silently change those policies.

The root lockfile is not committed. Validation must retain the exact audited
resolution; consumers must audit their own locks. The yoke-derive 0.8.2 MSRV
constraint remains intentional. Current resolution/tooling limits and review
dates are recorded in [dependency maintenance](dependencies.md).

## Work Included in the Candidate

| Review area | Implementation and evidence |
| --- | --- |
| Adapter-message redaction and safe examples | [#8](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/8) and [#21](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/21); display, debug, alternate, nested, source-chain, and process-exit regression cases; compiled redacted-settings example. |
| URL syntax and schemes | [#9](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/9); proper parsing, allowlist enforcement, malformed authorities/ports, and strict raw-input rejection. Migration is required for previously accepted ambiguous forms. |
| Release input and artifact integrity | [#10](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/10) and [#11](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/11); hostile tag tests, validated commit/lock retention, checksum verification, and reproducible package comparison before authentication. |
| Release approval and registry identity | [#14](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/14); GitHub reviewer/ref/tag controls and a verified environment-bound crates.io Trusted Publisher. Recheck external settings before each release. |
| Runtime adapters and contracts | [#15](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/15), [#16](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/16), and [#17](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/17); invalid process names, native subprocess environments, all ten field types, optional propagation, and defensive-limit boundaries. |
| Default and floating-point policy | [#12](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/12) and [#19](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/19); compatible opt-in validation with deterministic regressions and migration guidance. |
| Robustness and documentation | [#18](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/18) and [#20](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/20); deterministic properties, five bounded sanitizer fuzz targets, and compiled README/guide examples. |
| Dependency maintenance and detection | [#13](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/13) and [#22](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/22); reviewed dependency/tool versions, both fresh root graphs and committed fuzz graph, enabled alerts/security updates, daily audits, and maintainer triage. |

The checklist and commit-specific validation links live in
[#23](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/23). Evidence must
identify a full source commit, toolchain, dependency lock, and run; a green
check for an earlier patch is not final validation of the combined candidate.
Retain package and audit hashes with the release bundle. Changing the source,
lockfile, or toolchain requires rerunning the affected gates before release.

## Final Candidate Gates

Use the [locked local gate sequence](release-checklist.md#before-tagging) from
a clean, committed checkout of the selected candidate. Generate and audit the
release resolution first, then retain that lockfile and report with the
candidate evidence. Use the reviewed stable compiler and cargo-audit version
from [dependency maintenance](dependencies.md). Also run:

- The full CI Rust matrix, including Rust 1.85, all supported native platforms,
  the required aggregate check, and every documentation example.
- The Python workflow suite and workflow lint checks. Release tests reject
  malformed or substituted tags, source/lock/package drift, missing protections,
  and unwanted publication from validation-only runs.
- Current advisory audits of fresh `fallback` and `allow` resolutions and the
  committed fuzz lock. A warning or infrastructure failure is a failing audit.
- All deterministic property tests plus a fresh bounded run of the five fuzz
  targets. Record budgets, seed, tools, source, lock, execution counts, and
  outcome. The earlier runs in [fuzzing](fuzzing.md#recorded-bounded-run) remain
  historical evidence; a bounded run does not establish exhaustive coverage.
- Package contents and package verification from the selected source. Confirm
  that the packaged Markdown snippets also compile and that generated output,
  credentials, and local metadata are excluded.

The current GitHub approval/ref checks are recorded in the
[release-control runbook](release-controls.md#github-verification-record).
Rerun the read-only preflight and inspect the exact registry publisher row for
each release. GitHub preflight success alone is insufficient. A token exchange
and crate publication are not required to test or claim these configuration
checks; do not publish a test version for validation.

## Remaining Release Actions

The Trusted Publisher required by
[#14](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/14) was saved
and verified on 2026-09-27. It names `OneTesseractInMultiverse/envbind-rs`,
`publish.yml`, and the `crates-io` environment. The
[release-control runbook](release-controls.md#registry-verification-record)
records the inspection. Repeat the GitHub preflight and inspect every registry
entry before approving a release; unexpected or unrestricted entries require
investigation.

After the readiness changes are reviewed and merged, choose the exact release
commit and follow the [release checklist](release-checklist.md). Update the
release date and published-version wording as part of the release preparation,
run the final locked gates on that source, and only then create its matching
version tag. Review the retained validation bundle before the protected
publication approval. This record does not create a release, move a tag, or
publish a crate, and it does not promise that undiscovered defects are absent.
