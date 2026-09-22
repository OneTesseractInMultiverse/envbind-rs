# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

## Unreleased

### Changed

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

Added `.validate_default()` to every field spec to run typed validators on
fallback values during binding. Existing defaults still skip validation unless
this option is enabled. Fallback failures use the normal validation error and
sensitivity policy. Parsing and size limits remain specific to environment
input; typed defaults are never reparsed. Fixes
[#12](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/12).

### Fixed

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
