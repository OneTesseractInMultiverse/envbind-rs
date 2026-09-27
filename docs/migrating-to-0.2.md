# Migrating to 0.2

Version 0.2.0 is the next, currently unpublished release. The latest registry
version is 0.1.0; the repository's intermediate 0.1.1 metadata correction was
not published and is included here. Examples in the current checkout describe
0.2.0. Do not assume they are supported by the old registry package.

## Versioning Decision

The review tightened URL syntax and process-variable name handling. Although
these changes fix validation defects, configurations accepted by 0.1 can now
fail at startup. They therefore ship together in a new pre-1.0 minor line,
0.2.0, rather than a 0.1.x patch. Cargo treats a change to the leftmost nonzero
version component as incompatible; see the
[Cargo compatibility guide](https://doc.rust-lang.org/cargo/reference/semver.html).
The minimum Rust version remains 1.85, with the same documented
[platform baseline](../SUPPORT.md#supported-platforms).

## Changes to Review Before Upgrading

| Area | Before | Required review for 0.2 |
| --- | --- | --- |
| URL validation | String splitting could accept malformed URLs or overlook a disallowed explicit scheme when a scheme was optional. | Recheck startup configuration against the parser and scheme allowlist. Include a host and a valid port. Raw whitespace, controls, backslashes, and relative paths are rejected. |
| Scheme-less authorities | Some ports, credentials, and IPv6 forms were interpreted ambiguously. | With optional schemes, write `//localhost:8080`, `//user@example.test`, or `//[::1]:8080`. Bare hostnames still work. With required schemes, use an allowed explicit scheme such as `https://`. |
| Process-variable names | Empty names and names containing `=` or NUL could be treated as absent. | Correct the field names. These now return `EnvironmentError::InvalidName` even for defaulted or optional fields. Custom and map adapters retain their own namespace policy. |
| Adapter diagnostics | Derived `Debug` could reveal an adapter's stored message. | Logs now redact that message, including nested errors and alternate formatting. Do not parse debug text. Explicit field access still exposes the original message. |
| Bound settings | A shipped example derived `Debug` and printed the whole settings object. | Remove whole-settings dumps and review application `Debug`, logging, and serialization separately from binding diagnostics. |

For example, migrate a scheme-less host and port to an explicit authority:

```rust
use envbind::{Binder, MapEnvironment, StringVar, validators};

let environment = MapEnvironment::from_pairs([("SERVICE_URL", "//localhost:8080")]);
let spec = StringVar::new("SERVICE_URL")
    .validate(validators::is_url_with_options(false, ["http", "https"]));

assert_eq!(Binder::new(environment).bind(&spec)?, "//localhost:8080");
# Ok::<(), envbind::BindError>(())
```

Validation preserves the original URL string; it does not normalize the bound
value or resolve a hostname. See the complete
[URL policy](api-guide.md#url-validation) before choosing custom schemes.

`EnvironmentError` and `BindError` were already non-exhaustive. Keep the
fallback arm in downstream matches and explicitly handle `InvalidName` if the
application needs to distinguish malformed configuration keys. Binding still
reports adapter failures through the existing `environment_error` code.
No existing binding error-code string was renamed. Prefer these stable codes
and structured variants over matching error prose or debug formatting.

## Policies That Stay Opt-In

Defaults still bypass typed validators unless `.validate_default()` is enabled.
This is retained for compatibility. Enable it when an application requires its
fallback to satisfy the same typed policy as an environment value. Defaults
are already typed: they are not reparsed and do not use raw-input size limits.

Floating-point parsing still accepts NaN and positive/negative infinity,
including overflow that parses to infinity. Use `validators::is_finite()` for
a scalar or `validators::all_finite()` for a list. Validate defaults too when a
finite fallback is required:

```rust
use envbind::{Binder, FloatVar, MapEnvironment, validators};

let environment = MapEnvironment::new();
let spec = FloatVar::new("REQUEST_TIMEOUT")
    .default(5.0)
    .validate_default()
    .validate(validators::is_finite())
    .validate(validators::min_value(0.0));

assert_eq!(Binder::new(environment).bind(&spec)?, 5.0);
# Ok::<(), envbind::BindError>(())
```

See [validating defaults](api-guide.md#validating-defaults) and
[finite settings](api-guide.md#finite-floating-point-settings) for the full
composition and error behavior. Generic optional binding still converts only
missing/empty errors into `None`; parse, validation, size, and adapter failures
remain errors. The field-specific empty behavior is unchanged.

## Diagnostics and Operational Review

Binding diagnostics remain sensitive by default. `.sensitive(false)` exposes
validator details, so custom messages must not include credentials. Variable
names are visible error context. Successfully returned settings are ordinary
values; sensitivity does not redact logging, serialization, assertion output,
or direct field access, and there is no automatic zeroization. Use the
[reviewed logging example](api-guide.md#sensitivity-and-application-logging).

Input-size and item limits are unchanged. Review application-specific limits
and callback costs rather than raising limits indiscriminately. An arbitrary
custom parser or validator can still panic; the library does not make caller
code safe by catching its failures. Consult the
[binding-contract matrix](binding-contract-matrix.md) for exact boundaries.

Before deploying an upgrade, run the application's configuration tests with
representative synthetic inputs, include absent/defaulted/invalid cases, and
audit the application's own lockfile. The library's fresh dependency audits do
not cover every downstream dependency selection. The
[release-readiness record](release-readiness.md) describes the tested scope.
