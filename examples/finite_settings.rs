//! Finite, bounded startup settings with fallible duration conversion.

use std::time::Duration;

use envbind::{
    BindError, Binder, Environment, FloatVar, ListVar, ParameterSource, ValidationError, validators,
};

struct RuntimeSettings {
    timeout: Duration,
    requests_per_second: f64,
    retry_delays: Vec<f64>,
}

impl ParameterSource for RuntimeSettings {
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, BindError> {
        let seconds = binder.bind(
            &FloatVar::new("TIMEOUT_SECONDS")
                .default(30.0)
                .validate_default()
                .validate(validators::is_finite())
                .validate(validators::in_range(0.0, 300.0)),
        )?;
        let timeout = Duration::try_from_secs_f64(seconds).map_err(|_| {
            BindError::validation(
                "TIMEOUT_SECONDS",
                ValidationError::new("timeout must fit in a duration"),
            )
        })?;
        Ok(Self {
            timeout,
            requests_per_second: binder.bind(
                &FloatVar::new("REQUESTS_PER_SECOND")
                    .default(100.0)
                    .validate_default()
                    .validate(validators::is_finite())
                    .validate(validators::min_value(0.0)),
            )?,
            retry_delays: binder.bind(
                &ListVar::floats("RETRY_DELAYS")
                    .default(vec![0.1, 0.5, 1.0])
                    .validate_default()
                    .validate(validators::all_finite())
                    .validate(|values| {
                        values
                            .iter()
                            .copied()
                            .try_for_each(validators::in_range(0.0, 300.0))
                    }),
            )?,
        })
    }
}

fn main() -> Result<(), BindError> {
    let settings = RuntimeSettings::from_process_environment()?;
    // These specific operational settings are safe to report; avoid a struct dump.
    println!(
        "Timeout: {} seconds; request rate: {}; retry stages: {}",
        settings.timeout.as_secs_f64(),
        settings.requests_per_second,
        settings.retry_delays.len(),
    );
    Ok(())
}
