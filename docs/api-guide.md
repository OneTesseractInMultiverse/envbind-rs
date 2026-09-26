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
                    .validate_default()
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

Custom adapter diagnostics are redacted by both `Display` and `Debug`, including
alternate debug formatting, nested `BindError` output, and `Error::source()`
formatting. This also protects error output when `main` returns a binding error.
The `message` field of `EnvironmentError::Read` retains the original diagnostic
for explicit structured handling. Treat direct access to that field as
potentially sensitive.

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

### Process Environment Names

`ProcessEnvironment` requires a nonempty name containing neither `=` nor NUL
(`\0`). It checks the name before reading the process environment. This applies
on every platform, including names such as Windows' reserved `=C:` entries.
`allow_empty()` permits empty **values** and does not relax name validation.

Invalid names return `EnvironmentError::InvalidName`. Binding wraps this in
`BindError::Environment`, with the existing stable code `environment_error`.
The source displays `invalid environment variable name`; both debug formats
display `InvalidName`. The source stores no name, diagnostic payload, or raw
value. The enclosing binding error still exposes the caller-supplied name as
normal error context, so names must not contain secrets.

```rust
# use envbind::{Binder, BindingExt, ProcessEnvironment, StringVar};
let result = Binder::new(ProcessEnvironment)
    .bind(&StringVar::new("INVALID=NAME").default("fallback").optional())
    .map_err(|error| error.error_code());

assert_eq!(result, Err("environment_error"));
```

Defaults, `OptionalStringVar`, and `BindingExt::optional()` propagate this
failure. A valid name that is absent still follows the existing missing,
default, and optional behavior. All other names pass unchanged to the standard
library: Unicode, spaces, punctuation, and names starting with a digit are not
restricted to shell identifier syntax. Names are not trimmed or case-folded;
case matching remains platform-specific (case-sensitive on Unix and
case-insensitive on Windows).

This is a behavior change for malformed process keys. Previously, the
[standard library's absent-or-invalid result](https://doc.rust-lang.org/std/env/fn.var.html)
could select a fallback or become `None`. Correct malformed field names instead
of relying on that fallback. `MapEnvironment` continues to accept arbitrary
string keys, including empty strings, `=`, and NUL; custom `Environment`
implementations define their own namespace. The new variant extends the
already non-exhaustive adapter error enum without changing existing binding
error codes.

## Field Types and Options

Every field spec follows the same shape. It has a variable name, a default,
empty-string handling, sensitivity control, and validation. The shared methods
are `.default(...)`, `.validate_default()`, `.allow_empty()`,
`.sensitive(false)`, and `.validate(...)`.

By default, missing values fail without a default. Empty strings act as missing.
Values are sensitive, so custom validation details are hidden. Defaults skip
validators unless `.validate_default()` is enabled.

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
parser input.

Use `allow_empty()` for empty text that must parse as a real value. Use
`.sensitive(false)` for values that are safe to mention in validation details.

`BindingExt::optional()` wraps any binding spec and returns `None` for missing
or empty errors. The wrapped spec resolves defaults first. Successful defaults
return `Some`, and parsing or validation failures remain errors. This is useful
for optional ports, optional JSON values, and optional decoded strings.

## Validating Defaults

Defaults retain the original behavior: `.default(...)` alone skips validators.
Add `.validate_default()` when a rule must hold regardless of whether the value
comes from the environment or a fallback. This option is available on every
field type, including `OptionalStringVar`.

```rust
use envbind::{Binder, MapEnvironment, U16Var, validators};

let port = U16Var::new("PORT")
    .default(0)
    .validate_default()
    .validate(validators::u16_in_range(1, 65_535));
let results = [MapEnvironment::new(), MapEnvironment::from_pairs([("PORT", "0")])]
    .map(|environment| Binder::new(environment).bind(&port).map_err(|e| e.error_code()));
assert_eq!(results, [Err("validation_failed"), Err("validation_failed")]);
```

Validation happens during each binding, not while building the spec. Each
attached validator runs once in registration order, stopping at the first
failure. Calling `.validate_default()` before or after `.default(...)` or
`.validate(...)` has the same effect; repeated calls do not duplicate validation.
Errors use `validation_failed` and obey `.sensitive(...)` exactly as they do for
environment values. Custom details remain redacted unless sensitivity is disabled.

| Input | Behavior with `.validate_default()` |
| --- | --- |
| Missing, with a default | Validate the typed fallback. |
| Explicitly empty, with a default | Validate the typed fallback unless `.allow_empty()` is set. |
| Explicitly empty with `.allow_empty()` | Parse and validate the empty input; do not select the fallback. |
| Present but malformed or too large | Return the parsing or limit error; do not select the fallback. |
| Missing, without a default | Keep the existing missing error or optional `None`; no validator runs. |
| Explicitly empty, without a default or `.allow_empty()` | Keep the existing empty error or optional `None`; no validator runs. |

`allow_empty()` controls how input is resolved. It does not change which typed
defaults are valid. An empty string fallback still reaches attached validators;
use a length validator to reject it. Without validators, it remains a valid
fallback. Missing input still uses the default even with `allow_empty()`.

`OptionalStringVar` validates a fallback's string value before returning
`Some(value)`, including empty strings, and never invokes validators for `None`.
Apply `.validate_default()` to a field before wrapping it with
`BindingExt::optional()`. That wrapper preserves validation failures, including
failures for an empty fallback.

Typed defaults are never serialized, reparsed, decoded, split, or looked up by
enum label. JSON validators receive the provided `serde_json::Value` (including
`Null`), list validators receive the typed slice without invoking item parsers,
and enum validators receive the supplied target even if it has no input label.
Base64 defaults are already decoded text. Use attached validators for invariants
that must also hold on these typed values.

## Size Limits

Size limits protect startup from accidental large values. `StringVar` and
`OptionalStringVar` stop at 1 MiB of raw text. `BoolVar`, `IntVar`, `FloatVar`,
`U16Var`, and `EnumVar` use the same 1 MiB raw-text limit without a per-field
override.

`JsonVar` stops at 64 KiB of raw JSON, and `.max_bytes(...)` changes that
limit. `B64DecodedStringVar` stops at 1 MiB of decoded text, and
`.max_decoded_bytes(...)` changes that limit. `ListVar` stops at 1 MiB of raw
text and 1024 parsed items. `.max_items(...)` changes the item limit.

These built-in limits protect parsing environment input. None of them apply to
typed defaults, even with `.validate_default()`: this includes raw byte limits,
base64 decoded byte limits, and list item limits. To constrain fallback values,
attach a typed validator for the relevant byte length, character length, or
item count and enable `.validate_default()`.

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

### Finite Floating-Point Settings

`FloatVar` and `ListVar::floats` use Rust's `f64` parser. They accept `NaN`,
`inf`, `-inf`, and values that overflow to infinity, such as `1e999` and
`-1e999`. This permissive behavior remains the default for compatibility.
Finite-only validation is an explicit application policy; there is no planned
default change in this release. Changing that default would require a
documented breaking release and migration guidance for callers using non-finite
values.

Attach `validators::is_finite()` to a scalar or `validators::all_finite()` to a
float list. Both reject NaN and either infinity with `validation_failed`.
Diagnostics follow the normal sensitivity setting; even with
`.sensitive(false)`, these helpers use fixed messages without input values or
item positions. Optional wrappers preserve these failures.

Both signs of zero, finite negative numbers, subnormal values, and ordinary
exponent notation remain valid. Underflow can produce signed zero and remains
valid. An empty typed list is valid for `all_finite()`; enforce length separately
when needed. Empty environment input still follows the normal missing/empty
policy rather than becoming an empty list.

Compose finite validation with a lower bound for nonnegative rates. A lower
bound by itself admits positive infinity. Use finite bounds appropriate to the
application for timeouts and delays: even a finite `f64::MAX` cannot fit in a
`Duration`. Prefer `Duration::try_from_secs_f64` for fallible conversion.

```rust
use std::time::Duration;
use envbind::{Binder, FloatVar, ListVar, MapEnvironment, validators};

let binder = Binder::new(MapEnvironment::from_pairs([
    ("TIMEOUT_SECONDS", "2.5e1"),
    ("REQUESTS_PER_SECOND", "100"),
    ("RETRY_DELAYS", "0.1,0.5,1"),
]));
let seconds = binder.bind(
    &FloatVar::new("TIMEOUT_SECONDS")
        .default(30.0)
        .validate_default()
        .validate(validators::is_finite())
        .validate(validators::in_range(0.0, 300.0)),
)?;
let timeout = Duration::try_from_secs_f64(seconds)?;
let rate = binder.bind(
    &FloatVar::new("REQUESTS_PER_SECOND")
        .default(100.0)
        .validate_default()
        .validate(validators::is_finite())
        .validate(validators::min_value(0.0)),
)?;
let delays = binder.bind(
    &ListVar::floats("RETRY_DELAYS")
        .default(vec![0.1, 0.5, 1.0])
        .validate_default()
        .validate(validators::all_finite())
        .validate(|values| values.iter().copied().try_for_each(validators::in_range(0.0, 300.0))),
)?;
assert_eq!((timeout, rate, delays), (Duration::from_secs(25), 100.0, vec![0.1, 0.5, 1.0]));
# Ok::<(), Box<dyn std::error::Error>>(())
```

`.default(...)` continues to bypass validation unless `.validate_default()` is
enabled. For example, a NaN fallback remains accepted without that option even
when `is_finite()` is attached. Float-list defaults bypass item parsing, so
attach `all_finite()` as a whole-list validator and enable `.validate_default()`
to enforce the same invariant on every fallback element. These rules apply
equally to missing input and empty input that selects a fallback. Malformed
present input never selects a fallback.

The executable [finite settings example](../examples/finite_settings.rs) applies
these policies when loading process configuration at startup.

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

Helpers include `is_finite`, `all_finite`, `in_range`, `min_value`, `max_value`,
`one_of`, `one_of_values`, `min_length`, `max_length`, `matches_pattern`, `is_url`,
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

### URL Validation

`validators::is_url()` uses the `url` crate's WHATWG URL parser and accepts
HTTP/HTTPS URLs with a nonempty host. Ports, when supplied, must parse as a
`u16`; both 0 and 65,535 are valid syntax. Localhost, IPv4, bracketed IPv6,
internationalized domain names, and credentials in the authority are supported.
Validation does not perform DNS lookups or check whether a service is reachable.

Use `is_url_with_options(require_scheme, allowed_schemes)` for other schemes.
Explicit schemes are always checked against the allowlist, ignoring ASCII
case. This includes forms without `://`: `https:example.com` has the scheme
`https`, while `http:example.com` fails an HTTPS-only allowlist even when
`require_scheme` is false. Configured custom schemes such as
`postgres://db.example.com:5432/service` must also have a host. Hostless URLs
such as `mailto:user@example.com` are rejected even if their scheme is allowed.

When `require_scheme` is false, the following forms are supported:

| Form | Example |
| --- | --- |
| Bare hostname or IPv4, with optional path, query, and fragment | `example.com/path?key=value#part` |
| Authority with a port, prefixed by `//` | `//localhost:8080/path` |
| Bracketed IPv6, prefixed by `//` | `//[::1]:8443/path` |
| Credentials in an authority, prefixed by `//` | `//user:password@example.com:443/path` |
| Absolute URL with an allowed scheme | `https://example.com/path` |

Scheme-less authorities with ports, credentials, or IPv6 require `//`.
A syntactically valid scheme prefix is always treated as an explicit scheme,
so use `//example.com:443` instead of `example.com:443`. Relative paths such as
`/path`, `./path`, and `../path`, and query-only or fragment-only references,
are rejected.

Scheme-less input uses an internal HTTPS prefix only to validate the host and
port. The prefix is not checked against the allowlist, so an empty allowlist
still permits scheme-less input when the scheme is optional. Validation never
adds a scheme to the bound string or replaces it with the parser's normalized
output.

Raw whitespace, control characters, and backslashes are rejected in all modes.
Use percent encoding where appropriate, such as `https://example.com/a%20b`.
URL validation failures use fixed messages that never include the URL or its
credentials, including when the field uses `.sensitive(false)`.

These rules tighten the previous string-splitting validator: missing hosts,
malformed IPv6, invalid ports, disallowed explicit schemes without `://`, and
ambiguous scheme-less authorities now fail. Migrate scheme-less ports and
credentials to the explicit `//` form, or supply an allowed absolute URL.
