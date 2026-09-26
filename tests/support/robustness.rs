#![allow(missing_docs)]

use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use envbind::{
    B64DecodedStringVar, BindError, Binder, Binding, BindingExt, BoolVar, EnumVar, Environment,
    EnvironmentError, FloatVar, IntVar, JsonVar, ListVar, MapEnvironment, OptionalStringVar,
    StringVar, U16Var, ValidationError, validators,
};
use serde_json::Value;

pub const MAX_INPUT_BYTES: usize = 4096;
pub const MAX_ITEMS: usize = 64;
const MARKER: &str = "synthetic-fuzz-secret-7e1c";
pub type Check = Result<(), &'static str>;

fn require(condition: bool, message: &'static str) -> Check {
    if condition { Ok(()) } else { Err(message) }
}

fn error_contract(error: &BindError) -> Check {
    require(error.variable_name() == "VALUE", "wrong variable name")?;
    require(
        matches!(
            error.error_code(),
            "missing_variable"
                | "empty_variable"
                | "environment_error"
                | "invalid_boolean"
                | "parse_variable"
                | "validation_failed"
                | "value_too_large"
        ),
        "unknown error code",
    )?;
    let mut diagnostic = format!("{error} {error:?} {error:#?}");
    if let Some(source) = error.source() {
        diagnostic.push_str(&format!(" {source} {source:?} {source:#?}"));
    }
    require(
        !diagnostic.contains(MARKER),
        "sensitive marker in diagnostic",
    )
}

fn optional_contract<T: Debug, E: Environment, B: Binding<T>>(
    binder: &Binder<E>,
    spec: B,
) -> Check {
    let expected = match binder.bind(&spec) {
        Ok(value) => Ok(Some(format!("{value:?}"))),
        Err(error) => {
            error_contract(&error)?;
            match error {
                BindError::MissingVariable { .. } | BindError::EmptyVariable { .. } => Ok(None),
                _ => Err(error),
            }
        }
    };
    let actual = binder
        .bind(&spec.optional())
        .map(|value| value.map(|value| format!("{value:?}")));
    // Compare formatted successes so that NaN does not fail reflexivity.
    // Error values themselves must remain identical, including source data.
    require(
        actual == expected,
        "optional changed a value or swallowed an error",
    )
}

fn bounded(raw: &str, limit: usize) -> Check {
    require(
        raw.len() <= MAX_INPUT_BYTES && limit <= MAX_INPUT_BYTES,
        "unbounded harness input",
    )
}

pub fn check_json(raw: &str, limit: usize) -> Check {
    bounded(raw, limit)?;
    let binder = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]));
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let result = binder.bind(
        &JsonVar::new("VALUE")
            .allow_empty()
            .max_bytes(limit)
            .validate(move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
    );
    if raw.len() > limit {
        require(
            matches!(result, Err(BindError::ValueTooLarge { max_bytes, .. }) if max_bytes == limit),
            "oversized JSON reached parsing",
        )?;
        require(
            calls.load(Ordering::SeqCst) == 0,
            "oversized JSON reached validation",
        )?;
    }
    if let Err(error) = &result {
        error_contract(error)?;
    }
    optional_contract(
        &binder,
        JsonVar::new("VALUE").allow_empty().max_bytes(limit),
    )?;
    // Also reach the parser when the varied limit rejected the input above.
    optional_contract(&binder, JsonVar::new("VALUE").max_bytes(MAX_INPUT_BYTES))
}

pub fn check_json_roundtrip(value: &Value) -> Check {
    let raw = value.to_string();
    bounded(&raw, raw.len())?;
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw.as_str())]))
        .bind(&JsonVar::new("VALUE").max_bytes(raw.len()));
    require(
        result.as_ref() == Ok(value),
        "JSON round trip changed its value",
    )
}

pub fn check_base64(raw: &str, limit: usize) -> Check {
    bounded(raw, limit)?;
    let binder = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]));
    let result = binder.bind(
        &B64DecodedStringVar::new("VALUE")
            .allow_empty()
            .max_decoded_bytes(limit),
    );
    match &result {
        Ok(value) => require(value.len() <= limit, "decoded base64 exceeds limit")?,
        Err(error) => error_contract(error)?,
    }
    let encoded_limit = limit.div_ceil(3) * 4;
    if raw.len() > encoded_limit {
        require(
            matches!(result, Err(BindError::ValueTooLarge { max_bytes, .. }) if max_bytes == encoded_limit),
            "oversized base64 reached decoder",
        )?;
    }
    optional_contract(
        &binder,
        B64DecodedStringVar::new("VALUE")
            .allow_empty()
            .max_decoded_bytes(limit),
    )?;
    optional_contract(
        &binder,
        B64DecodedStringVar::new("VALUE").max_decoded_bytes(MAX_INPUT_BYTES),
    )
}

pub fn check_base64_roundtrip(raw: &str) -> Check {
    bounded(raw, raw.len())?;
    let encoded = STANDARD.encode(raw);
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", encoded)])).bind(
        &B64DecodedStringVar::new("VALUE")
            .allow_empty()
            .max_decoded_bytes(raw.len()),
    );
    require(
        result.as_deref() == Ok(raw),
        "base64 UTF-8 round trip changed text",
    )
}

pub fn check_list(raw: &str, delimiter: &str, limit: usize, keep_whitespace: bool) -> Check {
    bounded(raw, 0)?;
    require(
        delimiter.len() <= 8 && limit <= MAX_ITEMS,
        "unbounded list configuration",
    )?;
    let binder = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]));
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let mut spec = ListVar::new("VALUE", move |item| {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(item.to_owned())
    })
    .allow_empty()
    .delimiter(delimiter)
    .max_items(limit);
    if keep_whitespace {
        spec = spec.keep_whitespace();
    }
    let result = binder.bind(&spec);
    let count = calls.load(Ordering::SeqCst);
    require(count <= limit, "excess list item reached parser")?;
    if delimiter.is_empty() {
        require(
            count == 0 && result.is_err(),
            "empty delimiter reached parser",
        )?;
    }
    match result {
        Ok(items) => require(
            items.len() <= limit && items.len() == count,
            "list exceeds item limit",
        )?,
        Err(error) => error_contract(&error)?,
    }
    // Use fresh callbacks for the optional comparison.
    let mut spec = ListVar::strings("VALUE")
        .allow_empty()
        .delimiter(delimiter)
        .max_items(limit);
    if keep_whitespace {
        spec = spec.keep_whitespace();
    }
    optional_contract(&binder, spec)?;
    optional_contract(
        &binder,
        ListVar::integers("VALUE")
            .delimiter(delimiter)
            .max_items(limit),
    )?;
    optional_contract(
        &binder,
        ListVar::floats("VALUE")
            .delimiter(delimiter)
            .max_items(limit),
    )?;
    let mut floats = ListVar::floats("VALUE")
        .delimiter(delimiter)
        .max_items(limit);
    if keep_whitespace {
        floats = floats.keep_whitespace();
    }
    let expected = binder.bind(&floats).and_then(|values| {
        if values.iter().all(|value| value.is_finite()) {
            Ok(values.into_iter().map(f64::to_bits).collect::<Vec<_>>())
        } else {
            Err(BindError::validation_with_sensitivity(
                "VALUE",
                ValidationError::new("all list items must be finite"),
                true,
            ))
        }
    });
    let floats = floats.validate(validators::all_finite());
    let actual = binder
        .bind(&floats)
        .map(|values| values.into_iter().map(f64::to_bits).collect::<Vec<_>>());
    require(
        actual == expected,
        "finite list policy changed values or errors",
    )?;
    optional_contract(&binder, floats)?;
    optional_contract(
        &binder,
        ListVar::booleans("VALUE")
            .delimiter(delimiter)
            .max_items(limit),
    )?;
    optional_contract(
        &binder,
        ListVar::u16s("VALUE").delimiter(delimiter).max_items(limit),
    )?;
    optional_contract(
        &binder,
        ListVar::enumeration("VALUE", [("ready", 1_u8), ("stopped", 2)])
            .delimiter(delimiter)
            .max_items(limit),
    )?;
    optional_contract(
        &binder,
        ListVar::case_sensitive_enumeration("VALUE", [("ready", 1_u8), ("stopped", 2)])
            .delimiter(delimiter)
            .max_items(limit),
    )
}

pub fn check_list_raw_limit(extra: u8) -> Check {
    let raw_limit = 1024 * 1024;
    let raw = "x".repeat(raw_limit + usize::from(extra) + 1);
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let spec = ListVar::new("VALUE", move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)])).bind(&spec.optional());
    require(
        matches!(result, Err(BindError::ValueTooLarge { max_bytes, .. }) if max_bytes == raw_limit)
            && calls.load(Ordering::SeqCst) == 0,
        "oversized list reached parser or became absent",
    )
}

pub fn check_list_roundtrip(items: &[String], delimiter: &str) -> Check {
    let raw = items.join(delimiter);
    bounded(&raw, 0)?;
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)])).bind(
        &ListVar::strings("VALUE")
            .allow_empty()
            .keep_whitespace()
            .delimiter(delimiter)
            .max_items(items.len()),
    );
    require(
        result.as_deref() == Ok(items),
        "list round trip changed items",
    )
}

pub fn check_url(raw: &str) -> Check {
    bounded(raw, 0)?;
    let binder = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]));
    let forbidden = raw
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\');
    for require_scheme in [false, true] {
        let result = binder.bind(&StringVar::new("VALUE").allow_empty().validate(
            validators::is_url_with_options(require_scheme, ["http", "https", "postgres"]),
        ));
        match result {
            Ok(value) => require(
                !forbidden && value == raw,
                "URL accepted forbidden characters or changed text",
            )?,
            Err(error) => error_contract(&error)?,
        }
        optional_contract(
            &binder,
            StringVar::new("VALUE")
                .allow_empty()
                .sensitive(false)
                .validate(validators::is_url_with_options(
                    require_scheme,
                    ["http", "https", "postgres"],
                )),
        )?;
    }
    Ok(())
}

pub fn check_url_roundtrip(host: &str, port: u16, path: &str) -> Check {
    let raw = format!("https://{host}:{port}/{path}");
    let result = Binder::new(MapEnvironment::from_pairs([("VALUE", raw.as_str())]))
        .bind(&StringVar::new("VALUE").validate(validators::is_url()));
    require(
        result.as_deref() == Ok(raw.as_str()),
        "generated valid URL was rejected or changed",
    )
}

pub fn check_scalars(raw: &str, limit: usize) -> Check {
    bounded(raw, limit)?;
    let binder = Binder::new(MapEnvironment::from_pairs([("VALUE", raw)]));
    optional_contract(&binder, IntVar::new("VALUE"))?;
    optional_contract(&binder, FloatVar::new("VALUE"))?;
    let expected = binder.bind(&FloatVar::new("VALUE")).and_then(|value| {
        if value.is_finite() {
            Ok(value.to_bits())
        } else {
            Err(BindError::validation_with_sensitivity(
                "VALUE",
                ValidationError::new("value must be finite"),
                true,
            ))
        }
    });
    let finite = FloatVar::new("VALUE").validate(validators::is_finite());
    let actual = binder.bind(&finite).map(f64::to_bits);
    require(
        actual == expected,
        "finite scalar policy changed values or errors",
    )?;
    optional_contract(&binder, finite)?;
    optional_contract(&binder, BoolVar::new("VALUE"))?;
    optional_contract(&binder, U16Var::new("VALUE"))?;
    optional_contract(
        &binder,
        EnumVar::new("VALUE", [("ready", 1_u8), ("stopped", 2)]),
    )?;
    optional_contract(&binder, StringVar::new("VALUE").max_bytes(limit))?;
    optional_contract(&binder, OptionalStringVar::new("VALUE").max_bytes(limit))?;
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let result = binder.bind(
        &StringVar::new("VALUE")
            .allow_empty()
            .max_bytes(limit)
            .validate(move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
    );
    if raw.len() > limit {
        require(
            matches!(result, Err(BindError::ValueTooLarge { max_bytes, .. }) if max_bytes == limit)
                && calls.load(Ordering::SeqCst) == 0,
            "oversized string reached validation",
        )?;
    }
    Ok(())
}

pub fn check_scalar_roundtrip(integer: i64, port: u16, boolean: bool, float: f64) -> Check {
    let binder = Binder::new(MapEnvironment::from_pairs([
        ("INT", integer.to_string()),
        ("PORT", port.to_string()),
        ("BOOL", boolean.to_string()),
        ("FLOAT", float.to_string()),
    ]));
    require(
        binder.bind(&IntVar::new("INT")) == Ok(integer),
        "integer round trip changed value",
    )?;
    require(
        binder.bind(&U16Var::new("PORT")) == Ok(port),
        "u16 round trip changed value",
    )?;
    require(
        binder.bind(&BoolVar::new("BOOL")) == Ok(boolean),
        "boolean round trip changed value",
    )?;
    require(
        binder.bind(&FloatVar::new("FLOAT")).map(f64::to_bits) == Ok(float.to_bits()),
        "finite float round trip changed value",
    )
}

struct FailingEnvironment(String);

impl Environment for FailingEnvironment {
    fn get(&self, _name: &str) -> Result<Option<String>, EnvironmentError> {
        Err(EnvironmentError::read(self.0.clone()))
    }
}

pub fn check_redaction(raw: &str) -> Check {
    bounded(raw, 0)?;
    let secret = format!("{MARKER}:{raw}");
    let binder = Binder::new(MapEnvironment::from_pairs([("VALUE", secret.as_str())]));
    let results = [
        binder.bind(&JsonVar::new("VALUE")).map(|_| ()),
        binder.bind(&B64DecodedStringVar::new("VALUE")).map(|_| ()),
        binder.bind(&IntVar::new("VALUE")).map(|_| ()),
        binder.bind(&FloatVar::new("VALUE")).map(|_| ()),
        binder.bind(&U16Var::new("VALUE")).map(|_| ()),
        binder.bind(&BoolVar::new("VALUE")).map(|_| ()),
        binder.bind(&ListVar::integers("VALUE")).map(|_| ()),
        binder
            .bind(&EnumVar::new("VALUE", [("ready", 1_u8)]))
            .map(|_| ()),
        binder
            .bind(&StringVar::new("VALUE").validate(|value| Err(ValidationError::new(value))))
            .map(|_| ()),
        binder
            .bind(
                &OptionalStringVar::new("VALUE").validate(|value| Err(ValidationError::new(value))),
            )
            .map(|_| ()),
    ];
    for result in results {
        let error = result
            .err()
            .ok_or("synthetic invalid input unexpectedly succeeded")?;
        error_contract(&error)?;
    }
    optional_contract(
        &binder,
        StringVar::new("VALUE").validate(|value| Err(ValidationError::new(value))),
    )?;
    let adapter = Binder::new(FailingEnvironment(secret));
    optional_contract(
        &adapter,
        StringVar::new("VALUE").default("fallback").sensitive(false),
    )?;
    optional_contract(&adapter, IntVar::new("VALUE").default(1))?;
    optional_contract(&adapter, OptionalStringVar::new("VALUE"))?;
    let credential_url = format!("https://user:{MARKER}@[invalid]/{raw}");
    let error = Binder::new(MapEnvironment::from_pairs([("VALUE", credential_url)]))
        .bind(
            &StringVar::new("VALUE")
                .sensitive(false)
                .validate(validators::is_url()),
        )
        .err()
        .ok_or("invalid credential URL unexpectedly succeeded")?;
    error_contract(&error)
}
