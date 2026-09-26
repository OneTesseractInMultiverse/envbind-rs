# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

## Unreleased

### Changed

Refreshed the fuzzing lockfile to cc 1.5.1, find-msvc-tools 0.1.14, and
smallvec 1.16.2. Rechecked all dependency and tooling versions on 2026-09-26;
documented the remaining MSRV and upstream compatibility constraints.

Updated base64 to 0.23.1 while retaining the existing scalar decoder and Rust
1.85 support. Updated cargo-audit to 0.22.2, checkout to v7.0.1, upload-artifact
to v7.0.1, and crates-io-auth-action to v1.0.5 with verified commit pins.
Stable CI now tests the newest permitted dependency graph, the MSRV job tests
the Rust 1.85-compatible graph, and both graphs receive advisory audits.
Dependabot also covers Python workflow test dependencies. The yoke-derive
0.8.2 compatibility pin remains necessary; see the
[dependency review](docs/dependencies.md) for the complete version inventory
and remaining constraints. Fixes
[#13](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/13).

### Added

Added `validators::is_finite()` and `validators::all_finite()` for scalar and
list floating-point settings. They reject NaN and infinities, including parsing
overflow, with safe validation errors. Existing permissive parsing and default
behavior remain unchanged; use `.validate_default()` to cover typed fallbacks.
Added timeout/rate examples, regression tests, and property/fuzz checks. Fixes
[#19](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/19).

Added deterministic property tests and five isolated fuzz targets for parser
safety, defensive limits, optional-error propagation, round trips, and redaction.
Added a reviewed synthetic corpus, bounded CI smoke runs, dependency auditing
for the fuzz workspace, and failure-reproduction guidance. Fixes
[#18](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/18).

Added a documented regression matrix for all ten field types, covering optional
error propagation, validation diagnostics, and settings composition. Added exact
byte and item boundaries, parser callback counters, numeric overflow cases,
list parsing policies, and JSON/base64 edge cases. Fixes
[#17](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/17).

Added subprocess integration coverage for production settings loading,
present/absent/empty and Unicode values, invalid-Unicode adapter failures, and
safe error output. Stable CI now runs on Linux, macOS, and Windows, with an
aggregate `Rust stable` gate preserving the existing required check name and
the Linux Rust 1.85 check. Documented the supported target baseline. Fixes
[#16](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/16).

Configured the `crates-io` environment with required maintainer approval,
selected deployment refs, and no administrator bypass. Protected version tags
against updates and deletion. Added a read-only release preflight, regression
tests, and a token-free approval-gate fixture with a recovery runbook. See
[#14](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/14).

Added `.validate_default()` to every field spec to run typed validators on
fallback values during binding. Existing defaults still skip validation unless
this option is enabled. Fallback failures use the normal validation error and
sensitivity policy. Parsing and size limits remain specific to environment
input; typed defaults are never reparsed. Fixes
[#12](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/12).

### Fixed

Reject empty process-environment names and names containing `=` or NUL before
lookup. These now return `EnvironmentError::InvalidName` through the existing
`environment_error` binding code, including defaulted and optional fields.
Callers that relied on malformed names selecting defaults must correct their
field names. Valid missing names, Unicode names, and map/custom adapter
namespaces retain their behavior. Fixes
[#15](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/15).

Bound publication to the full commit SHA and dependency resolution that passed
release validation. Release artifacts now retain the audited lockfile, audit
results, package, and provenance metadata. Publishing verifies their checksums,
uses the recorded Rust toolchain, and compares the rebuilt package before
authentication. Moving a tag between jobs cannot substitute source code.
Fixes [#11](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/11).

Prevented shell injection through release tag inputs. The release workflow
validates tags before checkout, passes inputs as environment data, and checks
fully qualified tag refs and manifest versions before publishing. Release tags
now require the documented `v` prefix; manual runs also accept `refs/tags/v...`.
Fixes [#10](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/10).

Redacted adapter diagnostic messages in `EnvironmentError` debug output,
including nested binding errors, error sources, and errors returned from `main`.
The original message remains available through the public field for structured
handling. Fixes [#8](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/8).

Replaced URL string splitting with the `url` parser. Explicit schemes now obey
the allowlist even when the scheme is optional, and missing hosts, malformed
IPv6, and invalid ports fail validation. Error messages never include URL
credentials. Fixes [#9](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/9).

URL validation now requires a `//` prefix for scheme-less ports, credentials,
and IPv6 authorities, such as `//localhost:8080`. Bare hostnames remain valid
when the scheme is optional. Raw whitespace, control characters, backslashes,
and relative path references are rejected. The original bound string is
preserved; see the [URL validation guide](docs/api-guide.md#url-validation).

## 0.1.1 - 2026-05-28

### Fixed

Corrected the crates.io repository metadata to point at
`OneTesseractInMultiverse/envbind-rs`.

## 0.1.0 - 2026-05-28

### Added

Initial Envbind crate with environment adapters, binding traits, typed field
specs, validation helpers, examples, tests, docs, and release files.

The initial field set includes `StringVar`, `OptionalStringVar`, `BoolVar`,
`IntVar`, `FloatVar`, `ListVar`, `JsonVar`, `EnumVar`,
`B64DecodedStringVar`, and `U16Var`.

The initial behavior includes generic optional binding through
`BindingExt::optional`, Python EnvBind-compatible empty and default handling,
sensitivity-aware validation, enum name aliases, enum value aliases, stable
`BindError` codes, and reusable validators for ranges, lengths, membership,
regex, configurable URLs, email addresses, and composed validation.
