# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

## 0.1.0 - Unreleased

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
