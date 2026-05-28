# API Guide

This guide describes common Envbind usage in Rust services. It covers settings
structs, environment sources, field specs, validation, safe defaults, and error
codes.

## Define a Settings Object

Settings objects are plain Rust structs with typed fields. Keep them near
application startup or adapter wiring code. This keeps environment reads out of
handlers, use cases, and domain types.

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

`ParameterSource` keeps construction explicit. Each line names the environment
variable, the target type, the default, and the validation rule.

## Environment Sources

Use process loading only at the application boundary. This call reads the
actual process environment:

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

Use `MapEnvironment` for tests and deterministic examples. It avoids process
state and keeps each test self-contained.

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

## Field Types and Options

Every field spec follows the same shape. It has a variable name, a default,
empty-string handling, sensitivity control, and validation. The shared methods
are `.default(...)`, `.allow_empty()`, `.sensitive(false)`, and
`.validate(...)`.

By default, missing values fail without a default. Empty strings act as missing.
Values are sensitive, so custom validation details are hidden. Defaults return
before validators run.

| Field | Target type | Main options |
| --- | --- | --- |
| `StringVar` | `String` | `.max_bytes(...)` |
| `OptionalStringVar` | `Option<String>` | `.max_bytes(...)` |
| `IntVar` | `i64` | Shared options only |
| `FloatVar` | `f64` | Shared options only |
| `U16Var` | `u16` | Shared options only |
| `BoolVar` | `bool` | Shared options only |
| `ListVar<T>` | `Vec<T>` | `.delimiter(...)`, `.keep_whitespace()`, `.max_items(...)` |
| `JsonVar` | `serde_json::Value` | `.max_bytes(...)` |
| `EnumVar<T>` | `T` | `.alias(...)`, `.case_sensitive()` |
| `B64DecodedStringVar` | `String` | `.max_decoded_bytes(...)` |

Explicit empty strings act as missing by default. Whitespace-only text remains
parser input. Defaults return before validators run.

Use `allow_empty()` for empty text that must parse as a real value. Use
`.sensitive(false)` for values that are safe to mention in validation details.

`BindingExt::optional()` wraps any binding spec and returns `None` for missing
or empty input. This is useful for optional ports, optional JSON values, and
optional decoded strings.

## Size Limits

Size limits protect startup from accidental large values. `StringVar` and
`OptionalStringVar` stop at 1 MiB of raw text. `BoolVar`, `IntVar`, `FloatVar`,
`U16Var`, and `EnumVar` use the same 1 MiB raw-text limit without a per-field
override.

`JsonVar` stops at 64 KiB of raw JSON, and `.max_bytes(...)` changes that
limit. `B64DecodedStringVar` stops at 1 MiB of decoded text, and
`.max_decoded_bytes(...)` changes that limit. `ListVar` stops at 1 MiB of raw
text and 1024 parsed items. `.max_items(...)` changes the item limit.

## String and Boolean Examples

String values preserve whitespace. Empty strings act as missing by default.

```rust
# use envbind::{Binder, MapEnvironment, StringVar};
let name = Binder::new(MapEnvironment::from_pairs([("NAME", "api")]))
    .bind(&StringVar::new("NAME").default("worker"))?;
assert_eq!(name, "api");
# Ok::<(), envbind::BindError>(())
```

Boolean parsing accepts common service tokens:

```rust
# use envbind::{Binder, BoolVar, MapEnvironment};
let tracing = Binder::new(MapEnvironment::from_pairs([("TRACE", "on")]))
    .bind(&BoolVar::new("TRACE").default(false))?;
assert!(tracing);
# Ok::<(), envbind::BindError>(())
```

## Number and List Examples

Numeric fields parse first, then run validators. This example accepts ports
from 1 through 65,535.

```rust
# use envbind::{Binder, MapEnvironment, U16Var, validators};
let port = Binder::new(MapEnvironment::from_pairs([("PORT", "8080")]))
    .bind(&U16Var::new("PORT").validate(validators::u16_in_range(1, 65_535)))?;
assert_eq!(port, 8080);
# Ok::<(), envbind::BindError>(())
```

Lists split on commas by default. Items are trimmed by default. Use
`.delimiter(...)` for another separator and `.keep_whitespace()` for exact
items.

```rust
# use envbind::{Binder, ListVar, MapEnvironment};
let hosts = Binder::new(MapEnvironment::from_pairs([("HOSTS", "api, worker, db")]))
    .bind(&ListVar::strings("HOSTS"))?;
assert_eq!(hosts, vec!["api", "worker", "db"]);
# Ok::<(), envbind::BindError>(())
```

## JSON, Enum, and Base64 Examples

`JsonVar` parses a value into `serde_json::Value`. The raw JSON payload uses a
64 KiB default limit.

```rust
# use envbind::{Binder, JsonVar, MapEnvironment};
# use serde_json::json;
let value = Binder::new(MapEnvironment::from_pairs([("PROFILE", r#"{"debug":true}"#)]))
    .bind(&JsonVar::new("PROFILE"))?;
assert_eq!(value, json!({"debug": true}));
# Ok::<(), envbind::BindError>(())
```

`EnumVar` maps explicit labels to caller-owned values. Matching ignores ASCII
case by default. Use `.case_sensitive()` for exact labels.

```rust
# use envbind::{Binder, EnumVar, MapEnvironment};
#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    Blue,
    Green,
}

let mode = Binder::new(MapEnvironment::from_pairs([("MODE", "green")]))
    .bind(&EnumVar::new("MODE", [("BLUE", Mode::Blue), ("GREEN", Mode::Green)]))?;
assert_eq!(mode, Mode::Green);
# Ok::<(), envbind::BindError>(())
```

For enum names and external string values, use `EnumVar::from_names_and_values`.
For one extra label, use `.alias(...)`.

`B64DecodedStringVar` decodes base64 and then checks UTF-8.

```rust
# use envbind::{B64DecodedStringVar, Binder, MapEnvironment};
let text = Binder::new(MapEnvironment::from_pairs([("CERT", "Y2VydGlmaWNhdGU=")]))
    .bind(&B64DecodedStringVar::new("CERT"))?;
assert_eq!(text, "certificate");
# Ok::<(), envbind::BindError>(())
```

## Validation

Use validators for startup checks. Keep domain rules in domain code.
Configuration validation protects the boundary between raw text and typed
settings.

Helpers include `in_range`, `min_value`, `max_value`, `one_of`,
`one_of_values`, `min_length`, `max_length`, `matches_pattern`, `is_url`,
`is_url_with_options`, `is_email`, `all_of`, and `all_of_str`.

```rust
# use envbind::{Binder, MapEnvironment, StringVar, validators};
let environment = MapEnvironment::from_pairs([("ENVIRONMENT", "prod")]);
let value = Binder::new(environment).bind(
    &StringVar::new("ENVIRONMENT")
        .sensitive(false)
        .validate(validators::one_of(["dev", "prod"])),
)?;
assert_eq!(value, "prod");
# Ok::<(), envbind::BindError>(())
```

Validation messages must be safe to display. Values are sensitive by default,
and `.sensitive(false)` shows custom validation details.
