#![allow(missing_docs)]

use std::error::Error;
use std::process::Command;

use envbind::{Binder, BindingExt, Environment, OptionalStringVar, ProcessEnvironment, StringVar};

// Only the child receives fixture variables. The parent environment is never
// modified, so missing-name checks cannot depend on the test runner.
fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args_os().any(|argument| argument == "--process-name-fixture") {
        valid_missing_name_returns_absence();
        valid_missing_name_fails_required_binding();
        valid_missing_name_selects_default();
        valid_missing_name_becomes_optional_absence();
        valid_missing_name_uses_optional_string_absence();
        unicode_name_reads_value();
        name_outside_shell_identifier_syntax_reads_value();
        name_whitespace_is_preserved();
        native_name_case_behavior_is_preserved();
        return Ok(());
    }

    let output = Command::new(std::env::current_exe()?)
        .arg("--process-name-fixture")
        .env_clear()
        .env("ENVBIND_服務_名前", "unicode-name-value")
        .env("1envbind.name-with space", "non-shell-name-value")
        .env(" ENVBIND_SPACED ", "spaced-name-value")
        .env("ENVBIND_CASE_NAME", "case-value")
        .output()?;

    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ),
        (Some(0), String::new(), String::new()),
    );
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
    #[cfg(windows)]
    assert_eq!(
        ProcessEnvironment.get("envbind_case_name"),
        Ok(Some("case-value".to_owned())),
    );
    #[cfg(not(windows))]
    assert_eq!(ProcessEnvironment.get("envbind_case_name"), Ok(None));
}
