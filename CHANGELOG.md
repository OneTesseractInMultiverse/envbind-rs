# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

## Unreleased

### Fixed

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
