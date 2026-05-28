# API Guide

This guide shows common Envbind usage in Rust services.

## Define a Settings Object

Settings objects are plain Rust structs with typed fields. Keep them near
application startup or adapter wiring code.

```rust
use envbind::{
    B64DecodedStringVar, Binder, BindingExt, BoolVar, Environment, IntVar, ListVar,
    ParameterSource, StringVar, validators,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceSettings {
    host: String,
    port: i64,
    service_name: String,
    tracing_enabled: bool,
    hosts: Vec<String>,
    certificate: Option<String>,
}

impl ParameterSource for ServiceSettings {
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, envbind::BindError> {
        Ok(Self {
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
            certificate: binder.bind(&B64DecodedStringVar::new("SERVICE_CERT").optional())?,
        })
    }
}
```

## Load the Process Environment

Use process loading only at the application boundary.

```rust
# use envbind::{BindError, Binder, Environment, ParameterSource, StringVar};
# #[derive(Debug, Clone, PartialEq, Eq)]
# struct Settings { host: String }
# impl ParameterSource for Settings {
#     fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, BindError> {
#         Ok(Self { host: binder.bind(&StringVar::new("HOST").default("localhost"))? })
#     }
# }
let settings = Settings::from_process_environment()?;
# let _ = settings;
# Ok::<(), BindError>(())
```

Adapter read failures return `BindError` with code `environment_error`. One
example is non-Unicode data in the process environment.

## Load an In-Memory Environment

Use `MapEnvironment` for tests and deterministic examples.

```rust
# use envbind::{BindError, Binder, Environment, MapEnvironment, ParameterSource, StringVar};
# #[derive(Debug, Clone, PartialEq, Eq)]
# struct Settings { host: String }
# impl ParameterSource for Settings {
#     fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, BindError> {
#         Ok(Self { host: binder.bind(&StringVar::new("HOST").default("localhost"))? })
#     }
# }
let environment = MapEnvironment::from_pairs([("HOST", "localhost")]);
let settings = Settings::from_environment(environment)?;
assert_eq!(settings.host, "localhost");
# Ok::<(), BindError>(())
```

## Field Types

- `StringVar` binds a required string and can have a default.
- `OptionalStringVar` binds `Option<String>`.
- `IntVar`, `FloatVar`, and `U16Var` bind numeric values.
- `BoolVar` accepts `1`, `true`, `yes`, `on`, `y`, and `t` as true.
- `BoolVar` accepts `0`, `false`, `no`, `off`, `n`, and `f` as false.
- `ListVar` binds delimiter-separated lists.
- `JsonVar` binds JSON as `serde_json::Value`.
- `EnumVar` binds enum-like values from explicit labels.
- `B64DecodedStringVar` binds base64-encoded UTF-8 text.

Explicit empty strings act as missing by default. Whitespace-only text remains
parser input. Defaults return before validators run.

Use `allow_empty()` for empty text that must parse as a real value.

Field specs have defensive size defaults. General raw values stop at 1 MiB.
`JsonVar` stops at 64 KiB. `B64DecodedStringVar` stops at 1 MiB of decoded text.
`ListVar` stops at 1024 items.

Set `.max_bytes(...)`, `.max_decoded_bytes(...)`, or `.max_items(...)` for
larger values.

## Validation

Use validators for startup checks.

```rust
# use envbind::{Binder, MapEnvironment, U16Var, validators};
let port = Binder::new(MapEnvironment::from_pairs([("PORT", "8080")]))
    .bind(&U16Var::new("PORT").validate(validators::u16_in_range(1, 65_535)))?;
assert_eq!(port, 8080);
# Ok::<(), envbind::BindError>(())
```

Helpers include `in_range`, `min_value`, `max_value`, `one_of`,
`one_of_values`, `min_length`, `max_length`, `matches_pattern`, `is_url`,
`is_url_with_options`, `is_email`, `all_of`, and `all_of_str`.

Keep domain rules in domain code. Configuration validation protects the
boundary between raw text and typed settings.

Validation messages must be safe to display. Do not include raw environment
values. Values are sensitive by default, and `.sensitive(false)` shows custom
validation details.
