#![allow(missing_docs)]

use std::error::Error;
use std::process::Command;

use envbind::{Binder, Environment, EnvironmentError, StringVar};

struct FailingAdapter;

impl Environment for FailingAdapter {
    fn get(&self, _name: &str) -> Result<Option<String>, EnvironmentError> {
        Err(EnvironmentError::read("synthetic-secret-value"))
    }
}

// A custom harness lets the child return a binding error from main, exercising
// Rust's real termination formatter without mutating the process environment.
fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args_os().any(|argument| argument == "--adapter-read-failure") {
        Binder::new(FailingAdapter).bind(&StringVar::new("TOKEN"))?;
        return Ok(());
    }

    let output = Command::new(std::env::current_exe()?)
        .arg("--adapter-read-failure")
        .output()?;

    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        ),
        (
            Some(1),
            String::new(),
            "Error: Environment { name: \"TOKEN\", source: Read { message: \"[redacted]\" } }\n"
                .to_owned(),
        )
    );
    Ok(())
}
