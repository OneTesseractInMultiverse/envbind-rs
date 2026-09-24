#![allow(missing_docs)]

use std::error::Error;
#[cfg(any(unix, windows))]
use std::ffi::OsString;
use std::process::{Command, Output};

use envbind::{
    BindError, Binder, BindingExt, BoolVar, Environment, EnvironmentError, OptionalStringVar,
    ParameterSource, ProcessEnvironment, StringVar, U16Var,
};

#[derive(Debug, PartialEq, Eq)]
struct ProcessSettings {
    host: String,
    port: u16,
    tracing: bool,
    region: String,
    credential: Option<String>,
}

impl ParameterSource for ProcessSettings {
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, BindError> {
        Ok(Self {
            host: binder.bind(&StringVar::new("ENVBIND_HOST"))?,
            port: binder.bind(&U16Var::new("ENVBIND_PORT").default(8080))?,
            tracing: binder.bind(&BoolVar::new("ENVBIND_TRACE").default(false))?,
            region: binder.bind(&StringVar::new("ENVBIND_REGION").default("fallback-region"))?,
            credential: binder.bind(&StringVar::new("ENVBIND_CREDENTIAL").optional())?,
        })
    }
}

// Only the child receives fixture variables. The parent environment is never
// modified, so missing-name checks cannot depend on the test runner.
fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("--process-name-fixture") => {
            valid_missing_name_returns_absence();
            valid_missing_name_fails_required_binding();
            valid_missing_name_selects_default();
            valid_missing_name_becomes_optional_absence();
            valid_missing_name_uses_optional_string_absence();
            unicode_name_reads_value();
            name_outside_shell_identifier_syntax_reads_value();
            name_whitespace_is_preserved();
            native_name_case_behavior_is_preserved();
        }
        Some("--process-value-fixture") => {
            present_value_is_read();
            empty_value_is_present();
            empty_value_fails_required_binding();
            empty_value_selects_default();
            empty_value_becomes_optional_absence();
            empty_value_can_be_preserved();
            unicode_value_is_preserved();
        }
        Some("--settings-fixture") => production_settings_are_composed(),
        Some("--settings-failure") => {
            ProcessSettings::from_process_environment()?;
        }
        #[cfg(any(unix, windows))]
        Some("--non-unicode-fixture") => {
            non_unicode_value_is_an_adapter_error();
            non_unicode_value_does_not_select_default();
            non_unicode_value_does_not_become_optional_absence();
            non_unicode_value_does_not_select_optional_string_default();
            non_unicode_value_fails_typed_binding();
            non_unicode_source_formatting_has_no_raw_value();
        }
        _ => {
            process_names_fixture_passes()?;
            process_values_fixture_passes()?;
            production_settings_fixture_passes()?;
            production_parse_failure_is_redacted()?;
            #[cfg(any(unix, windows))]
            {
                non_unicode_fixture_passes()?;
                production_read_failure_is_redacted()?;
            }
        }
    }
    Ok(())
}

fn child(mode: &str) -> Result<Command, std::io::Error> {
    let mut command = Command::new(std::env::current_exe()?);
    command.arg(mode).env_clear();
    Ok(command)
}

fn captured(output: Output) -> (Option<i32>, String, String) {
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
    )
}

fn process_names_fixture_passes() -> Result<(), Box<dyn Error>> {
    let output = child("--process-name-fixture")?
        .env("ENVBIND_服務_名前", "unicode-name-value")
        .env("1envbind.name-with space", "non-shell-name-value")
        .env(" ENVBIND_SPACED ", "spaced-name-value")
        .env("ENVBIND_CASE_NAME", "case-value")
        .output()?;

    assert_eq!(captured(output), (Some(0), String::new(), String::new()));
    Ok(())
}

fn valid_missing_name_returns_absence() {
    assert_eq!(ProcessEnvironment.get("ENVBIND_MISSING"), Ok(None));
}

fn valid_missing_name_fails_required_binding() {
    let value = Binder::new(ProcessEnvironment)
        .bind(&StringVar::new("ENVBIND_MISSING"))
        .map_err(|error| error.error_code());

    assert_eq!(value, Err("missing_variable"));
}

fn valid_missing_name_selects_default() {
    let value = Binder::new(ProcessEnvironment)
        .bind(&StringVar::new("ENVBIND_MISSING").default("fallback"));

    assert_eq!(value, Ok("fallback".to_owned()));
}

fn valid_missing_name_becomes_optional_absence() {
    let value = Binder::new(ProcessEnvironment).bind(&StringVar::new("ENVBIND_MISSING").optional());

    assert_eq!(value, Ok(None));
}

fn valid_missing_name_uses_optional_string_absence() {
    let value = Binder::new(ProcessEnvironment).bind(&OptionalStringVar::new("ENVBIND_MISSING"));

    assert_eq!(value, Ok(None));
}

fn unicode_name_reads_value() {
    assert_eq!(
        ProcessEnvironment.get("ENVBIND_服務_名前"),
        Ok(Some("unicode-name-value".to_owned())),
    );
}

fn name_outside_shell_identifier_syntax_reads_value() {
    assert_eq!(
        ProcessEnvironment.get("1envbind.name-with space"),
        Ok(Some("non-shell-name-value".to_owned())),
    );
}

fn name_whitespace_is_preserved() {
    assert_eq!(
        ProcessEnvironment.get(" ENVBIND_SPACED "),
        Ok(Some("spaced-name-value".to_owned())),
    );
}

fn native_name_case_behavior_is_preserved() {
    // The adapter delegates case matching to the OS after syntax validation.
    let expected = if cfg!(windows) {
        Some("case-value".to_owned())
    } else {
        None
    };
    assert_eq!(ProcessEnvironment.get("envbind_case_name"), Ok(expected));
}

fn process_values_fixture_passes() -> Result<(), Box<dyn Error>> {
    let output = child("--process-value-fixture")?
        .env("ENVBIND_PRESENT", "synthetic-present-value")
        .env("ENVBIND_EMPTY", "")
        .env("ENVBIND_UNICODE", "café 東京 🦀")
        .output()?;

    assert_eq!(captured(output), (Some(0), String::new(), String::new()));
    Ok(())
}

fn present_value_is_read() {
    assert_eq!(
        ProcessEnvironment.get("ENVBIND_PRESENT"),
        Ok(Some("synthetic-present-value".to_owned()))
    );
}

fn empty_value_is_present() {
    assert_eq!(
        ProcessEnvironment.get("ENVBIND_EMPTY"),
        Ok(Some(String::new()))
    );
}

fn empty_value_fails_required_binding() {
    let value = Binder::new(ProcessEnvironment)
        .bind(&StringVar::new("ENVBIND_EMPTY"))
        .map_err(|error| error.error_code());
    assert_eq!(value, Err("empty_variable"));
}

fn empty_value_selects_default() {
    let value =
        Binder::new(ProcessEnvironment).bind(&StringVar::new("ENVBIND_EMPTY").default("fallback"));
    assert_eq!(value, Ok("fallback".to_owned()));
}

fn empty_value_becomes_optional_absence() {
    let value = Binder::new(ProcessEnvironment).bind(&StringVar::new("ENVBIND_EMPTY").optional());
    assert_eq!(value, Ok(None));
}

fn empty_value_can_be_preserved() {
    let value =
        Binder::new(ProcessEnvironment).bind(&StringVar::new("ENVBIND_EMPTY").allow_empty());
    assert_eq!(value, Ok(String::new()));
}

fn unicode_value_is_preserved() {
    assert_eq!(
        ProcessEnvironment.get("ENVBIND_UNICODE"),
        Ok(Some("café 東京 🦀".to_owned()))
    );
}

fn production_settings_fixture_passes() -> Result<(), Box<dyn Error>> {
    let output = child("--settings-fixture")?
        .env("ENVBIND_HOST", "服務.example")
        .env("ENVBIND_PORT", "9090")
        .env("ENVBIND_TRACE", "true")
        .env("ENVBIND_REGION", "")
        .output()?;
    assert_eq!(captured(output), (Some(0), String::new(), String::new()));
    Ok(())
}

fn production_settings_are_composed() {
    assert_eq!(
        ProcessSettings::from_process_environment(),
        Ok(ProcessSettings {
            host: "服務.example".to_owned(),
            port: 9090,
            tracing: true,
            region: "fallback-region".to_owned(),
            credential: None,
        })
    );
}

fn production_parse_failure_is_redacted() -> Result<(), Box<dyn Error>> {
    let output = child("--settings-failure")?
        .env("ENVBIND_HOST", "synthetic-secret-host")
        .env("ENVBIND_PORT", "synthetic-secret-port")
        .output()?;
    assert_eq!(
        captured(output),
        (
            Some(1),
            String::new(),
            "Error: ParseVariable { name: \"ENVBIND_PORT\", expected: \"u16\" }\n".to_owned(),
        )
    );
    Ok(())
}

#[cfg(unix)]
fn non_unicode_value() -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(b"synthetic-secret-\xff".to_vec())
}

#[cfg(windows)]
fn non_unicode_value() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    let mut units: Vec<u16> = "synthetic-secret-".encode_utf16().collect();
    units.push(0xD800); // Unpaired high surrogate, preserved by OsString.
    OsString::from_wide(&units)
}

#[cfg(any(unix, windows))]
fn non_unicode_fixture_passes() -> Result<(), Box<dyn Error>> {
    let output = child("--non-unicode-fixture")?
        .env("ENVBIND_NON_UNICODE", non_unicode_value())
        .output()?;
    assert_eq!(captured(output), (Some(0), String::new(), String::new()));
    Ok(())
}

#[cfg(any(unix, windows))]
fn non_unicode_value_is_an_adapter_error() {
    assert_eq!(
        ProcessEnvironment.get("ENVBIND_NON_UNICODE"),
        Err(EnvironmentError::NotUnicode)
    );
}

#[cfg(any(unix, windows))]
fn non_unicode_value_does_not_select_default() {
    let value = Binder::new(ProcessEnvironment)
        .bind(&StringVar::new("ENVBIND_NON_UNICODE").default("fallback"))
        .map_err(|error| error.error_code());
    assert_eq!(value, Err("environment_error"));
}

#[cfg(any(unix, windows))]
fn non_unicode_value_does_not_become_optional_absence() {
    let value = Binder::new(ProcessEnvironment)
        .bind(
            &StringVar::new("ENVBIND_NON_UNICODE")
                .default("fallback")
                .optional(),
        )
        .map_err(|error| error.error_code());
    assert_eq!(value, Err("environment_error"));
}

#[cfg(any(unix, windows))]
fn non_unicode_value_does_not_select_optional_string_default() {
    let value = Binder::new(ProcessEnvironment)
        .bind(&OptionalStringVar::new("ENVBIND_NON_UNICODE").default("fallback"))
        .map_err(|error| error.error_code());
    assert_eq!(value, Err("environment_error"));
}

#[cfg(any(unix, windows))]
fn non_unicode_value_fails_typed_binding() {
    let value = Binder::new(ProcessEnvironment)
        .bind(&U16Var::new("ENVBIND_NON_UNICODE").default(8080).optional())
        .map_err(|error| error.error_code());
    assert_eq!(value, Err("environment_error"));
}

#[cfg(any(unix, windows))]
fn non_unicode_source_formatting_has_no_raw_value() {
    let result = Binder::new(ProcessEnvironment)
        .bind(&StringVar::new("ENVBIND_NON_UNICODE"))
        .map_err(|error| {
            error.source().map(|source| {
                (
                    source.to_string(),
                    format!("{source:?}"),
                    format!("{source:#?}"),
                )
            })
        });
    assert_eq!(
        result,
        Err(Some((
            "value is not valid Unicode".to_owned(),
            "NotUnicode".to_owned(),
            "NotUnicode".to_owned(),
        )))
    );
}

#[cfg(any(unix, windows))]
fn production_read_failure_is_redacted() -> Result<(), Box<dyn Error>> {
    let output = child("--settings-failure")?
        .env("ENVBIND_HOST", "synthetic-secret-host")
        .env("ENVBIND_PORT", non_unicode_value())
        .output()?;
    assert_eq!(
        captured(output),
        (
            Some(1),
            String::new(),
            "Error: Environment { name: \"ENVBIND_PORT\", source: NotUnicode }\n".to_owned(),
        )
    );
    Ok(())
}
