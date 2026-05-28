# Architecture

Envbind is a small boundary library for typed configuration loading. It is not
a service locator, settings store, secret manager, or domain rule engine.

The crate fits hexagonal Rust services. Bootstrap code loads typed settings.
Domain code receives those settings and avoids process environment reads.

## Responsibility Split

The implementation keeps coordination separate from computation.

- `Environment` reads raw values by name.
- `ProcessEnvironment` reads the process environment.
- `MapEnvironment` gives tests in-memory values.
- `Binding<T>` describes one typed variable spec.
- `Binder` applies a spec to an environment source.
- `ParameterSource` builds settings structs.
- `validators` hold reusable validation functions.

No module reads an external source and parses domain settings. The environment
adapter fetches text or reports read failure. The variable spec parses and
validates one value. The binder coordinates those parts.

## Module Layout

```text
src/
  binder.rs       Binding trait and Binder coordinator
  environment.rs  Environment port plus process and map adapters
  error.rs        BindError and ValidationError
  fields/
    b64.rs        B64DecodedStringVar
    bool.rs       BoolVar
    enumeration.rs EnumVar
    float.rs      FloatVar
    int.rs        IntVar
    json.rs       JsonVar
    list.rs       ListVar
    raw.rs        Shared raw value lookup for field specs
    string.rs     StringVar and OptionalStringVar
    u16.rs        U16Var
  source.rs       ParameterSource trait
  validators.rs   Reusable validation helpers
```

## Adding Field Types

Add a field type for real service needs. Keep it narrow.

- It parses one target type.
- It exposes options tied to that type.
- It returns `BindError` with stable `error_code()` values.
- It supports validators for configuration boundary checks.
- It has tests for present, missing, empty, default, parse failure, and validation.

Use explicit field types instead of a broad generic field. Each type has one
reason to change.

## Error Contracts

`BindError::error_code()` is public behavior. Tests cover new variants and
branches that map to stable codes.

Display text can change for clarity. Error codes stay stable across compatible
releases.

Do not include raw environment values in error display text. Environment
variables often contain secrets. Parse failures name the variable and expected
shape, not the raw value.

Field specs apply safe defaults. General raw values have a byte limit. JSON and
base64 specs add type-specific limits. List specs limit item count before they
allocate a large vector.

Adapter read errors store adapter messages for structured use. Display text
uses a generic message, so custom adapters do not leak raw values through logs.

The error enums are non-exhaustive. Compatible releases can add failure modes
without breaking downstream matches.
