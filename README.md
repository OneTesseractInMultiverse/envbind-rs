# Envbind-rs

Typed environment binding primitives for Rust services.

Envbind is an open source Rust library for typed configuration. It reads environment-like sources at startup, parses
values into settings structs, and keeps raw reads at the application edge. The crate fits services that separate domain
code from infrastructure code.

The library is small by design. It gives each part one job, and it keeps those jobs visible in tests. An application
defines a settings struct, then uses field specs to bind each value. The field spec handles parsing, validation, and
safe error text.

## Design Model

Configuration loading has three steps. First, an `Environment` adapter reads raw text. Next, a field spec such as
`StringVar` or `U16Var` parses one value. Then `ParameterSource` builds a settings struct from typed values.

`Binder` sits between the source and the spec. It coordinates the read and the parse, but it does not contain service
rules. This split keeps startup code clear. Domain code receives typed settings and never needs `std::env`.

The crate forbids unsafe code and denies missing public docs. Local checks run formatting, Cargo check, Clippy with
warnings denied, tests, doc tests, and docs.rs-style docs.

## Safe Defaults

Envbind treats values as sensitive by default. Validation details stay hidden until a field is marked with
`.sensitive(false)`. Parse errors name the variable and the expected type. They do not print the raw value.

The crate limits input size before costly parsing. General raw values stop at 1 MiB. `JsonVar` stops at 64 KiB.
`B64DecodedStringVar` stops at 1 MiB of decoded text. `ListVar` stops at 1024 items. Larger values require an explicit
limit setter.

| Type                  | Default limit                 | Override                  |
|-----------------------|-------------------------------|---------------------------|
| `StringVar`           | 1 MiB raw text                | `.max_bytes(...)`         |
| `OptionalStringVar`   | 1 MiB raw text                | `.max_bytes(...)`         |
| `BoolVar`             | 1 MiB raw text                | No byte override          |
| `IntVar`              | 1 MiB raw text                | No byte override          |
| `FloatVar`            | 1 MiB raw text                | No byte override          |
| `U16Var`              | 1 MiB raw text                | No byte override          |
| `EnumVar`             | 1 MiB raw text                | No byte override          |
| `JsonVar`             | 64 KiB raw JSON               | `.max_bytes(...)`         |
| `B64DecodedStringVar` | 1 MiB decoded text            | `.max_decoded_bytes(...)` |
| `ListVar`             | 1 MiB raw text and 1024 items | `.max_items(...)`         |

## Installation

The GitHub repository uses the `OneTesseractInMultiverse` namespace. The crates.io package uses the lowercase package
name
`onetesseractinmultiverse-envbind`. Rust code imports the crate as `envbind`.

```toml
[dependencies]
onetesseractinmultiverse-envbind = "0.1.0"
```

Then import the Rust crate name:

```rust
use envbind::{Binder, Environment, ParameterSource, StringVar};
```

## Quick Start

This example binds six fields into a single settings struct. It uses a map source for a deterministic example.
Production code normally calls
`Settings::from_process_environment()` at startup.

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

Load the process environment at the application boundary:

```rust
let settings = Settings::from_process_environment() ?;
```

## Public API

The public API centers on a small set of traits and field specs.

| Item                 | Role                                               |
|----------------------|----------------------------------------------------|
| `Environment`        | Reads raw values by name.                          |
| `ProcessEnvironment` | Reads from the process environment.                |
| `MapEnvironment`     | Gives tests and examples in-memory values.         |
| `Binder`             | Coordinates one binding spec with one environment. |
| `ParameterSource`    | Builds an application settings struct.             |
| `BindError`          | Reports stable binding failures.                   |
| `ValidationError`    | Carries safe validation text.                      |

Field specs cover the common startup types: `StringVar`, `OptionalStringVar`,
`BoolVar`, `IntVar`, `FloatVar`, `ListVar`, `JsonVar`, `EnumVar`,
`B64DecodedStringVar`, and `U16Var`.

Every field spec supports `.default(...)`, `.allow_empty()`,
`.sensitive(false)`, and `.validate(...)`. `BindingExt::optional()` wraps any binding spec and returns `None` for
missing or empty input.

## Field Behavior

`StringVar` binds a required string. Missing values fail without a default. Empty strings act as missing by default.
Whitespace-only strings remain input. Defaults return before validation.

```rust
# use envbind::{Binder, MapEnvironment, StringVar};
let host = Binder::new(MapEnvironment::from_pairs([("HOST", "localhost")]))
.bind( & StringVar::new("HOST")) ?;
assert_eq!(host, "localhost");
# Ok::<(), envbind::BindError>(())
```

`OptionalStringVar` returns `Option<String>`. Missing values return `None`. Empty strings return `None` by default.

```rust
# use envbind::{Binder, MapEnvironment, OptionalStringVar};
let token = Binder::new(MapEnvironment::new()).bind( & OptionalStringVar::new("TOKEN")) ?;
assert_eq!(token, None);
# Ok::<(), envbind::BindError>(())
```

`BoolVar` accepts `1`, `true`, `yes`, `on`, `y`, and `t` for true. It accepts
`0`, `false`, `no`, `off`, `n`, and `f` for false. Matching ignores ASCII case after trimming.

```rust
# use envbind::{Binder, BoolVar, MapEnvironment};
let enabled = Binder::new(MapEnvironment::from_pairs([("TRACE", "yes")]))
.bind( & BoolVar::new("TRACE")) ?;
assert!(enabled);
# Ok::<(), envbind::BindError>(())
```

`U16Var` parses a `u16` and runs typed validators. Use it for ports and other bounded unsigned values.

```rust
# use envbind::{Binder, MapEnvironment, U16Var, validators};
let port = Binder::new(MapEnvironment::from_pairs([("PORT", "8080")]))
.bind( & U16Var::new("PORT").validate(validators::u16_in_range(1, 65_535))) ?;
assert_eq!(port, 8080);
# Ok::<(), envbind::BindError>(())
```

## Python EnvBind Parity

Rust uses typed binding specs instead of Python descriptors. The field set maps to the Python package in a direct way.

| Python field          | Rust field            |
|-----------------------|-----------------------|
| `StringEnv`           | `StringVar`           |
| `IntEnv`              | `IntVar`              |
| `FloatEnv`            | `FloatVar`            |
| `BooleanEnv`          | `BoolVar`             |
| `ListEnv`             | `ListVar`             |
| `JSONEnv`             | `JsonVar`             |
| `EnumEnv`             | `EnumVar`             |
| `B64DecodedStringEnv` | `B64DecodedStringVar` |

Use `.default(...)` for fallback values. Use `.allow_empty()` to parse empty text. Set `.sensitive(false)` only for
display-safe validation details. Use
`.optional()` from `BindingExt` for missing values that return `None`.

The enum helpers accept both enum names and string values. The list helpers cover string, integer, float, boolean,
`u16`, enum-like, and custom item parsers.

For exact enum labels, call `.case_sensitive()`. For extra labels, call
`.alias(...)`.

## Error Handling

Binding failures return `BindError`. Each variant maps to a stable
`error_code()` string. Use it for logging, metrics, and tests.

Display text does not include raw environment values. Callers often bind credentials or private deployment settings.

```rust
# use envbind::{Binder, MapEnvironment, U16Var};
let error_code = Binder::new(MapEnvironment::from_pairs([("PORT", "abc")]))
.bind( & U16Var::new("PORT"))
.map_err( | error| error.error_code());

assert_eq!(error_code, Err("parse_variable"));
```

| Code                | Meaning                                |
|---------------------|----------------------------------------|
| `missing_variable`  | A required variable was absent.        |
| `empty_variable`    | A required variable was empty.         |
| `environment_error` | The adapter failed to read a value.    |
| `invalid_boolean`   | A boolean value used an unknown token. |
| `parse_variable`    | A value did not match the target type. |
| `validation_failed` | A typed value failed validation.       |
| `value_too_large`   | A raw value exceeded its byte limit.   |

## Repository Documentation

The repository includes focused documents for design, use, tests, release work, and project policy. Start
with [API Guide](docs/api-guide.md) for usage. Read [Architecture](docs/architecture.md) for design rules.

Reference files:

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
make package-list
make package
make publish-dry-run
```

`make verify` is the main local gate. It checks formatting, type checks the crate, runs Clippy, runs tests, runs doc
tests, and builds docs.rs-style docs.

## Publishing Readiness

Run these commands before a release:

```sh
make verify
make package-list
make package
make publish-dry-run
```

The package includes source, tests, examples, docs, and the MIT license. It excludes build output and machine-specific
files.

## License

Licensed under the MIT License. See [LICENSE-MIT](LICENSE-MIT).
