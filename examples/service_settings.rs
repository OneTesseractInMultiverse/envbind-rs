#![allow(missing_docs)]

use envbind::{Binder, BoolVar, Environment, ParameterSource, StringVar, U16Var, validators};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceSettings {
    host: String,
    port: u16,
    service_name: String,
    tracing_enabled: bool,
}

impl ParameterSource for ServiceSettings {
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, envbind::BindError> {
        Ok(Self {
            host: binder.bind(&StringVar::new("SERVICE_HOST").default("127.0.0.1"))?,
            port: binder.bind(
                &U16Var::new("SERVICE_PORT")
                    .default(8080)
                    .validate(validators::u16_in_range(1, 65_535)),
            )?,
            service_name: binder.bind(&StringVar::new("SERVICE_NAME").default("api"))?,
            tracing_enabled: binder.bind(&BoolVar::new("SERVICE_TRACING").default(false))?,
        })
    }
}

fn main() -> Result<(), envbind::BindError> {
    let settings = ServiceSettings::from_process_environment()?;
    println!("{settings:?}");
    Ok(())
}
