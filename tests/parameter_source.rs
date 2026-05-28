#![allow(missing_docs)]

use envbind::{
    BindError, Binder, BoolVar, Environment, MapEnvironment, ParameterSource, StringVar,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceSettings {
    service_name: String,
    tracing_enabled: bool,
}

impl ParameterSource for ServiceSettings {
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, BindError> {
        Ok(Self {
            service_name: binder.bind(&StringVar::new("SERVICE_NAME").default("svc"))?,
            tracing_enabled: binder.bind(&BoolVar::new("TRACE_ENABLED").default(false))?,
        })
    }
}

#[test]
fn parameter_source_builds_from_map_environment() {
    let settings = ServiceSettings::from_environment(MapEnvironment::from_pairs([(
        "SERVICE_NAME",
        "gateway",
    )]));

    assert_eq!(
        settings,
        Ok(ServiceSettings {
            service_name: "gateway".to_owned(),
            tracing_enabled: false,
        })
    );
}

#[test]
fn parameter_source_uses_defaults_for_empty_map_environment() {
    let settings = ServiceSettings::from_environment(MapEnvironment::new());

    assert_eq!(
        settings,
        Ok(ServiceSettings {
            service_name: "svc".to_owned(),
            tracing_enabled: false,
        })
    );
}
