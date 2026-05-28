# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

## 0.1.0 - Unreleased

### Added

- Initial Envbind crate.
- `Environment`, `ProcessEnvironment`, and `MapEnvironment`.
- `Binder` and `Binding`.
- `ParameterSource`.
- `StringVar`, `OptionalStringVar`, `BoolVar`, `IntVar`, `FloatVar`, `ListVar`, `JsonVar`, `EnumVar`, `B64DecodedStringVar`, and `U16Var`.
- Generic optional binding through `BindingExt::optional`.
- Python EnvBind-compatible empty, default, and sensitivity behavior.
- Sensitivity-aware boolean validation.
- Enum name and value aliases.
- `BindError` and `ValidationError`.
- Reusable validators for ranges, lengths, membership, regex, configurable URLs, email addresses, and composed validation.
- Docs, examples, tests, and release checklist.
