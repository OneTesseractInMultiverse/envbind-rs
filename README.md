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

Envbind treats values as sensitive in binding diagnostics by default. Validation
details stay hidden until a field is marked with `.sensitive(false)`. Built-in
parse errors name the variable and the expected type without printing the input.

Successful bindings return ordinary Rust values. Sensitivity does not redact
their `Debug` output, serialization, or application logs, and it provides no
automatic zeroization or secret-memory protection. Avoid deriving `Debug` for
settings containing credentials; cloning and memory handling remain the caller's
responsibility. See the [sensitivity boundary](docs/api-guide.md#sensitivity-and-application-logging)
for an application-owned redacted `Debug` example.

The crate limits input size before costly parsing. General raw values stop at 1 MiB. `JsonVar` stops at 64 KiB.
`B64DecodedStringVar` stops at 1 MiB of decoded text. `ListVar` stops at 1024 items.
String and JSON byte limits, the base64 decoded-byte limit, and the list item
limit have explicit setters. Scalar and list raw-byte limits are fixed.

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

The crates.io package name is `envbind`. The GitHub repository is
`OneTesseractInMultiverse/envbind-rs`. Rust code imports the crate as
`envbind`.

This checkout prepares **0.2.0**, which has not been published yet. The latest
published version is 0.1.0; the intermediate 0.1.1 metadata change was not
published. This README describes the upcoming API and behavior. Review the
[migration guide](docs/migrating-to-0.2.md) before upgrading and the
[release-readiness record](docs/release-readiness.md) for validation and support
limits. After 0.2.0 is published, use:

```toml
[dependencies]
envbind = "0.2.0"
```

Until then, build this checkout directly or use a Cargo path dependency to a
reviewed local checkout; the registry requirement above is not yet installable.

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

// Credentials are ordinary strings; omit automatically derived debug output.
#[derive(Clone, PartialEq, Eq)]
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
                    .validate_default()
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

Load the process environment at the application boundary. This startup snippet
is compiled but not executed by documentation tests because it reads the
application's process environment:

```rust,no_run
# use envbind::{BindError, ParameterSource};
# fn load<Settings: ParameterSource>() -> Result<Settings, BindError> {
let settings = Settings::from_process_environment()?;
# Ok(settings)
# }
```

## Public API

The public API centers on a small set of traits and field specs.

| Item                 | Role                                               |
|----------------------|----------------------------------------------------|
| `Environment`        | Reads raw values by name.                          |
| `ProcessEnvironment` | Reads from the process environment.                |
| `MapEnvironment`     | Gives tests and examples in-memory values.         |
| `Binder`             | Coordinates one binding spec with one environment. |
| `Binding`            | Defines a typed binding, including custom specs.   |
| `BindingExt`, `OptionalVar` | Wrap a spec to handle missing or empty values. |
| `ParameterSource`    | Builds an application settings struct.             |
| `BindError`          | Reports stable binding failures.                   |
| `EnvironmentError`   | Reports adapter failures and invalid process names. |
| `VariableName`       | Holds the variable name attached to a binding error. |
| `ValidationError`    | Carries validator-provided text; binding sensitivity controls disclosure. |

Field specs cover the common startup types: `StringVar`, `OptionalStringVar`,
`BoolVar`, `IntVar`, `FloatVar`, `ListVar`, `JsonVar`, `EnumVar`,
`B64DecodedStringVar`, and `U16Var`.

`ProcessEnvironment` rejects empty variable names and names containing `=` or
NUL. These return `environment_error` with source `EnvironmentError::InvalidName`,
even for defaulted or optional fields. Valid Unicode names remain supported;
name matching follows the operating system. `MapEnvironment` and custom
adapters keep their own naming rules. See the
[process-name contract](docs/api-guide.md#process-environment-names).

Every field spec supports `.default(...)`, `.validate_default()`, `.allow_empty()`,
`.sensitive(false)`, and `.validate(...)`. `BindingExt::optional()` wraps any binding spec and converts
missing or empty errors to `None`. Successful defaults return `Some`; adapter,
size-limit, parsing, and validation errors remain errors.

## Field Behavior

`StringVar` binds a required string. Missing values fail without a default. Empty strings act as missing by default.
Whitespace-only strings remain input. Defaults skip validation unless
`.validate_default()` is enabled. This runs the same typed validators on a selected
fallback, with the usual error redaction. Parsing limits still apply only to
environment input. See [Validating Defaults](docs/api-guide.md#validating-defaults)
for optional fields, empty values, and container defaults.

```rust
# use envbind::{Binder, MapEnvironment, StringVar};
let host = Binder::new(MapEnvironment::from_pairs([("HOST", "localhost")]))
    .bind(&StringVar::new("HOST"))?;
assert_eq!(host, "localhost");
# Ok::<(), envbind::BindError>(())
```

`OptionalStringVar` returns `Option<String>`. Missing values return `None`. Empty strings return `None` by default.

```rust
# use envbind::{Binder, MapEnvironment, OptionalStringVar};
let token = Binder::new(MapEnvironment::new()).bind(&OptionalStringVar::new("TOKEN"))?;
assert_eq!(token, None);
# Ok::<(), envbind::BindError>(())
```

`BoolVar` accepts `1`, `true`, `yes`, `on`, `y`, and `t` for true. It accepts
`0`, `false`, `no`, `off`, `n`, and `f` for false. Matching ignores ASCII case after trimming.

```rust
# use envbind::{Binder, BoolVar, MapEnvironment};
let enabled = Binder::new(MapEnvironment::from_pairs([("TRACE", "yes")]))
    .bind(&BoolVar::new("TRACE"))?;
assert!(enabled);
# Ok::<(), envbind::BindError>(())
```

`U16Var` parses a `u16` and runs typed validators. Use it for ports and other bounded unsigned values.

```rust
# use envbind::{Binder, MapEnvironment, U16Var, validators};
let port = Binder::new(MapEnvironment::from_pairs([("PORT", "8080")]))
    .bind(&U16Var::new("PORT").validate(validators::u16_in_range(1, 65_535)))?;
assert_eq!(port, 8080);
# Ok::<(), envbind::BindError>(())
```

`FloatVar` and `ListVar::floats` accept NaN, infinities, and overflow to infinity
by default. Use `validators::is_finite()` for scalars and
`validators::all_finite()` for lists. Enable `.validate_default()` to enforce
the rule on fallbacks too. Finite values still need application-specific bounds:

```rust
use std::time::Duration;
use envbind::{Binder, FloatVar, MapEnvironment, validators};

let seconds = Binder::new(MapEnvironment::new()).bind(
    &FloatVar::new("TIMEOUT_SECONDS")
        .default(30.0)
        .validate_default()
        .validate(validators::is_finite())
        .validate(validators::in_range(0.0, 300.0)),
)?;
let timeout = Duration::try_from_secs_f64(seconds)?;
assert_eq!(timeout, Duration::from_secs(30));
# Ok::<(), Box<dyn std::error::Error>>(())
```

See the [floating-point policy](docs/api-guide.md#finite-floating-point-settings)
and [timeout, rate, and retry example](examples/finite_settings.rs).

`validators::is_url()` parses HTTP/HTTPS URLs and requires a host and a valid port when supplied.
`is_url_with_options` supports custom schemes and optional schemes. Every explicit scheme is checked against the
allowlist. Optional-scheme input accepts bare hostnames; prefix authorities containing ports, credentials, or IPv6
with `//`, as in `//localhost:8080` or `//[::1]:443`.
Raw whitespace, control characters, and backslashes are rejected. Validation preserves the original string.
See [URL Validation](docs/api-guide.md#url-validation) for the accepted forms and compatibility changes.

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

Built-in parsing errors omit raw input. Sensitive validation failures replace
validator-provided details with a generic message. With `.sensitive(false)`,
custom details are retained in the structured error and its display/debug output,
so validators must supply messages that are safe to disclose. Variable names
are error context and must not contain secrets.

Adapter read errors hide their diagnostic messages in both `Display` and `Debug`, including alternate debug
formatting and errors returned from `main`. The `message` field of `EnvironmentError::Read` retains the original
diagnostic for explicit structured handling; treat that field as potentially sensitive.

```rust
# use envbind::{Binder, MapEnvironment, U16Var};
let error_code = Binder::new(MapEnvironment::from_pairs([("PORT", "abc")]))
    .bind(&U16Var::new("PORT"))
    .map_err(|error| error.error_code());

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
- [Binding Contract Matrix](docs/binding-contract-matrix.md)
- [Property Tests and Bounded Fuzzing](docs/fuzzing.md)
- [Dependency Maintenance](docs/dependencies.md)
- [Migrating to 0.2](docs/migrating-to-0.2.md)
- [Release Readiness](docs/release-readiness.md)
- [Publishing](docs/publishing.md)
- [Release Checklist](docs/release-checklist.md)
- [Release Controls](docs/release-controls.md)
- [Open Source Practices](docs/open-source.md)
- [Support](SUPPORT.md)
- [Contributing](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Changelog](CHANGELOG.md)

## Local Commands

Rust 1.85 is the declared minimum. Stable Rust CI covers Linux, macOS, and
Windows, including real process-environment loading and package verification.
See the [supported target baseline](SUPPORT.md#supported-platforms).

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
tests, and builds docs.rs-style docs. The documentation tests include Rust
snippets directly from this README and every guide under `docs/`. See the
[documentation test gate](docs/testing-guide.md#documentation-tests) for snippet
setup and execution rules.

Packaging and publishing dry runs require a clean, committed checkout. Use the
[contribution checklist](CONTRIBUTING.md#pull-request-checklist) for changes and
the release checklist below for locked release validation.

## Publishing Readiness

Follow the [release checklist](docs/release-checklist.md) to audit one dependency
resolution and run the checks, package verification, and publishing dry run with
that lockfile. Record the exact commit and validation results. The
[0.2.0 readiness record](docs/release-readiness.md) tracks support limits and
remaining release actions, including the missing registry Trusted Publisher.

The package includes source, tests, examples, docs, and the MIT license. It excludes build output and machine-specific
files.

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
