# Architecture

Envbind is a boundary library for typed configuration loading. It is not a
service locator, settings store, secret manager, or domain rule engine. Its
purpose is narrower: move raw text from an environment source into typed
settings at application startup.

The crate fits hexagonal Rust services. Bootstrap code reads configuration,
builds a settings struct, and passes that struct into adapters or service
composition code. Domain code receives typed data and does not read the process
environment.

## Responsibility Model

The implementation separates coordination from computation. Coordination moves
work between parts. Computation parses, validates, or maps one value.

An `Environment` adapter owns raw value lookup. `ProcessEnvironment` reads from
the process environment, and `MapEnvironment` stores deterministic test values.
A `Binding<T>` implementation describes one typed variable spec. `Binder`
applies that spec to an environment source. `ParameterSource` composes several
typed values into one settings struct.

This split matters in service code. A handler or use case receives a typed
setting, such as `u16` or `bool`. It never needs to know how the value entered
the process. Startup code remains the only place that speaks to the outside
configuration source.

## Module Layout

| Module | Purpose |
| --- | --- |
| `binder.rs` | Defines `Binding`, `Binder`, `BindingExt`, and optional binding. |
| `environment.rs` | Defines the environment port and its process and map adapters. |
| `error.rs` | Defines stable error types and display text. |
| `fields/raw.rs` | Shares raw value lookup and byte limits across fields. |
| `fields/string.rs` | Defines `StringVar` and `OptionalStringVar`. |
| `fields/bool.rs` | Defines `BoolVar`. |
| `fields/int.rs` | Defines `IntVar`. |
| `fields/float.rs` | Defines `FloatVar`. |
| `fields/u16.rs` | Defines `U16Var`. |
| `fields/list.rs` | Defines `ListVar`. |
| `fields/json.rs` | Defines `JsonVar`. |
| `fields/enumeration.rs` | Defines `EnumVar`. |
| `fields/b64.rs` | Defines `B64DecodedStringVar`. |
| `source.rs` | Defines `ParameterSource`. |
| `validators.rs` | Defines reusable validation functions. |

## Field Type Rules

Add a field type only for a real startup configuration need. The field parses
one target type. It exposes options tied to that type. It returns `BindError`
with stable `error_code()` values. It accepts validators only for checks that
belong at the configuration boundary.

A new field needs tests for present values, missing values, explicit empty
strings, defaults, parse failures, and validation failures. These tests give
the field a clear contract. They protect users from silent changes in startup
behavior.

Explicit field types are favored over a broad generic field. This keeps each
type small. It gives each type one reason to change.

All current field specs expose defaults, empty-string handling, sensitivity
control, and validation. String, JSON, base64, and list specs add size or
shape controls tied to their target type.

## Error Contracts

`BindError::error_code()` is public behavior. Tests cover new variants and
branches that map to stable codes. Display text can change for clarity, but
error codes remain stable across compatible releases.

Error display text does not include raw environment values. Environment
variables often carry secrets. Parse failures name the variable and expected
shape, not the raw value.

Field specs apply safe defaults. General raw values have a byte limit. JSON and
base64 specs add type-specific limits. List specs limit item count before they
allocate a large vector. Callers raise limits through explicit setters.

Adapter read errors store adapter messages for structured use. Display text
uses a generic message. This prevents custom adapters from leaking raw values
through logs.

The error enums are non-exhaustive. Compatible releases can add failure modes
without breaking downstream matches.

## Boundary Example

An application can keep configuration in startup code:

```rust
use envbind::{Binder, Environment, ParameterSource, StringVar, U16Var};

struct Settings {
    host: String,
    port: u16,
}

impl ParameterSource for Settings {
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, envbind::BindError> {
        Ok(Self {
            host: binder.bind(&StringVar::new("HOST").default("127.0.0.1"))?,
            port: binder.bind(&U16Var::new("PORT").default(8080))?,
        })
    }
}
```

After startup, other layers receive `Settings`. They do not receive an
environment reader.
