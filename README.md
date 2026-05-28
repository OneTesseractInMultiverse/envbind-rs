# Envbind-rs

Typed environment binding primitives for Rust services.

Envbind is an open source Rust library for typed configuration. It reads
environment-like sources at startup. It parses values into settings structs and
keeps raw reads at the application edge.

## Why This Crate Exists

Rust services often repeat the same startup work. They read environment
variables, parse text, validate values, then build settings structs.

Envbind keeps that work clear and testable:

- service code defines ordinary settings structs
- `Environment` adapters fetch raw values
- tests use `MapEnvironment` instead of process state
- each variable spec owns one parsing job
- `Binder` coordinates specs against a source

## Design Principles

Each part has one job:

- `Environment` adapters fetch raw values.
- `StringVar`, `IntVar`, `FloatVar`, `BoolVar`, `ListVar`, `JsonVar`,
  `EnumVar`, `B64DecodedStringVar`, and `U16Var` parse typed values.
- `Binder` coordinates variable specs against an environment.
- `ParameterSource` builds settings structs from typed values.

Functions either coordinate work or compute values. They do not do both.

The crate forbids unsafe code. It denies missing public docs. Local checks run
Clippy with warnings denied.

## Safe Defaults

Envbind treats values as sensitive by default.

- General raw values stop at 1 MiB.
- `JsonVar` stops at 64 KiB.
- `B64DecodedStringVar` stops at 1 MiB of decoded text.
- `ListVar` stops at 1024 items.
- Validator details stay hidden until `.sensitive(false)` is set.
- Adapter read messages use generic display text.

Set `.max_bytes(...)`, `.max_decoded_bytes(...)`, or `.max_items(...)` for
larger values.

## Installation

Add this crate to your dependencies, then import the Rust crate name:

```rust
use envbind::{Binder, Environment, ParameterSource, StringVar};
```

## Quick Start

```rust
use envbind::{
    B64DecodedStringVar, Binder, BindingExt, BoolVar, Environment, IntVar, ListVar, MapEnvironment,
    ParameterSource, StringVar, validators,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Settings {
    host: String,
    port: i64,
    service_name: String,
    tracing_enabled: bool,
    hosts: Vec<String>,
    certificate: String,
}

impl ParameterSource for Settings {
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, envbind::BindError> {
        Ok(Settings {
            host: binder.bind(&StringVar::new("SERVICE_HOST").default("127.0.0.1"))?,
            port: binder.bind(
                &IntVar::new("SERVICE_PORT")
                    .default(8080)
                    .sensitive(false)
                    .validate(validators::in_range(1, 65_535)),
            )?,
            service_name: binder.bind(&StringVar::new("SERVICE_NAME").default("api"))?,
            tracing_enabled: binder.bind(&BoolVar::new("SERVICE_TRACING").default(false))?,
            hosts: binder.bind(&ListVar::strings("SERVICE_HOSTS").default(vec![
                "localhost".to_owned(),
            ]))?,
            certificate: binder.bind(&B64DecodedStringVar::new("SERVICE_CERT").optional())?
                .unwrap_or_default(),
        })
    }
}

fn main() -> Result<(), envbind::BindError> {
    let environment = MapEnvironment::from_pairs([
        ("SERVICE_HOST", "localhost"),
        ("SERVICE_PORT", "9090"),
        ("SERVICE_HOSTS", "api,worker"),
        ("SERVICE_CERT", "Y2VydGlmaWNhdGU="),
    ]);

    let settings = Settings::from_environment(environment)?;
    assert_eq!(settings.port, 9090);
    Ok(())
}
```

Load the process environment at the boundary:

```rust
let settings = Settings::from_process_environment()?;
```

## Public API

- `Environment`, `ProcessEnvironment`, and `MapEnvironment`
- `Binder`, `Binding`, `BindingExt`, and `OptionalVar`
- `ParameterSource`
- `StringVar`, `OptionalStringVar`, `BoolVar`, `IntVar`, `FloatVar`,
  `ListVar`, `JsonVar`, `EnumVar`, `B64DecodedStringVar`, and `U16Var`
- `BindError`, `EnvironmentError`, and `ValidationError`
- validator helpers under `validators`

## Field Behavior

### `StringVar`

`StringVar` binds a required string. Missing values fail without a default.
Empty strings act as missing by default. Whitespace-only strings remain input.
Defaults return before validation.

```rust
# use envbind::{Binder, MapEnvironment, StringVar};
let host = Binder::new(MapEnvironment::from_pairs([("HOST", "localhost")]))
    .bind(&StringVar::new("HOST"))?;
assert_eq!(host, "localhost");
# Ok::<(), envbind::BindError>(())
```

### `OptionalStringVar`

`OptionalStringVar` binds `Option<String>`. Missing values return `None`.
Empty strings return `None` by default.

```rust
# use envbind::{Binder, MapEnvironment, OptionalStringVar};
let token = Binder::new(MapEnvironment::new()).bind(&OptionalStringVar::new("TOKEN"))?;
assert_eq!(token, None);
# Ok::<(), envbind::BindError>(())
```

### `BoolVar`

`BoolVar` accepts these true values: `1`, `true`, `yes`, `on`, `y`, and `t`.
It accepts these false values: `0`, `false`, `no`, `off`, `n`, and `f`.
Matching ignores ASCII case after trimming.

```rust
# use envbind::{Binder, BoolVar, MapEnvironment};
let enabled = Binder::new(MapEnvironment::from_pairs([("TRACE", "yes")]))
    .bind(&BoolVar::new("TRACE"))?;
assert!(enabled);
# Ok::<(), envbind::BindError>(())
```

### `U16Var`

`U16Var` parses a `u16` and runs typed validators.

```rust
# use envbind::{Binder, MapEnvironment, U16Var, validators};
let port = Binder::new(MapEnvironment::from_pairs([("PORT", "8080")]))
    .bind(&U16Var::new("PORT").validate(validators::u16_in_range(1, 65_535)))?;
assert_eq!(port, 8080);
# Ok::<(), envbind::BindError>(())
```

### Python EnvBind Parity

Rust uses typed binding specs instead of Python descriptors. The core field set
matches the Python package:

- `StringEnv` maps to `StringVar`
- `IntEnv` maps to `IntVar`
- `FloatEnv` maps to `FloatVar`
- `BooleanEnv` maps to `BoolVar`
- `ListEnv` maps to `ListVar`
- `JSONEnv` maps to `JsonVar`
- `EnumEnv` maps to `EnumVar`
- `B64DecodedStringEnv` maps to `B64DecodedStringVar`

Use `.default(...)` for fallback values. Use `.allow_empty()` to parse empty
text. Set `.sensitive(false)` only for display-safe validation details.
Use `.optional()` from `BindingExt` for missing values that return `None`.

Set `.max_bytes(...)`, `.max_decoded_bytes(...)`, or `.max_items(...)` for
larger values. The enum helpers accept both enum names and string values.

## Error Handling

Binding failures return `BindError`. Each variant maps to a stable
`error_code()` string. Use it for logging, metrics, and tests.

Display text does not include raw environment values. Callers often bind
credentials or private deployment settings.

```rust
# use envbind::{Binder, MapEnvironment, U16Var};
let error = Binder::new(MapEnvironment::from_pairs([("PORT", "abc")]))
    .bind(&U16Var::new("PORT"))
    .expect_err("invalid port fails");

assert_eq!(error.error_code(), "parse_variable");
```

Stable codes:

- `missing_variable`
- `empty_variable`
- `environment_error`
- `invalid_boolean`
- `parse_variable`
- `validation_failed`
- `value_too_large`

## Repository Documentation

- [Architecture](docs/architecture.md)
- [API Guide](docs/api-guide.md)
- [Testing Guide](docs/testing-guide.md)
- [Publishing](docs/publishing.md)
- [Release Checklist](docs/release-checklist.md)
- [Open Source Practices](docs/open-source.md)
- [Contributing](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Changelog](CHANGELOG.md)

## Local Commands

```sh
make test
make test-doc
make build
make check
make lint
make fmt
make doc
make verify
make package
```

Unit tests use in-memory sources. They do not need external files, shell
settings, services, or process environment variables.

## Publishing Readiness

Run these commands before a release:

```sh
make verify
make package
```

The package includes source, tests, examples, docs, and the MIT license. It
excludes build output and machine-specific files.

## License

Licensed under the MIT License. See [LICENSE-MIT](LICENSE-MIT).
