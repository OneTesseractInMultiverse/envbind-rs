#![allow(missing_docs)]

use envbind::{
    B64DecodedStringVar, BindError, Binder, BoolVar, EnumVar, Environment, FloatVar, IntVar,
    JsonVar, ListVar, MapEnvironment, OptionalStringVar, ParameterSource, StringVar, U16Var,
};
use serde_json::{Value, json};

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

#[derive(Debug, PartialEq)]
struct AllFieldSettings {
    name: String,
    description: Option<String>,
    enabled: bool,
    count: i64,
    ratio: f64,
    metadata: Value,
    decoded: String,
    mode: u8,
    labels: Vec<String>,
    port: u16,
}

impl ParameterSource for AllFieldSettings {
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, BindError> {
        Ok(Self {
            name: binder.bind(&StringVar::new("NAME"))?,
            description: binder.bind(&OptionalStringVar::new("DESCRIPTION"))?,
            enabled: binder.bind(&BoolVar::new("ENABLED"))?,
            count: binder.bind(&IntVar::new("COUNT"))?,
            ratio: binder.bind(&FloatVar::new("RATIO"))?,
            metadata: binder.bind(&JsonVar::new("METADATA"))?,
            decoded: binder.bind(&B64DecodedStringVar::new("ENCODED"))?,
            mode: binder.bind(&EnumVar::new("MODE", [("ready", 1)]))?,
            labels: binder.bind(&ListVar::strings("LABELS"))?,
            port: binder.bind(&U16Var::new("PORT"))?,
        })
    }
}

#[test]
fn parameter_source_composes_all_public_field_types() {
    let settings = AllFieldSettings::from_environment(MapEnvironment::from_pairs([
        ("NAME", "gateway"),
        ("DESCRIPTION", "service description"),
        ("ENABLED", "yes"),
        ("COUNT", "42"),
        ("RATIO", "0.25"),
        ("METADATA", "{\"region\":\"west\"}"),
        ("ENCODED", "aW5wdXQ="),
        ("MODE", "READY"),
        ("LABELS", "public, http"),
        ("PORT", "8080"),
    ]));
    assert_eq!(
        settings,
        Ok(AllFieldSettings {
            name: "gateway".to_owned(),
            description: Some("service description".to_owned()),
            enabled: true,
            count: 42,
            ratio: 0.25,
            metadata: json!({"region": "west"}),
            decoded: "input".to_owned(),
            mode: 1,
            labels: vec!["public".to_owned(), "http".to_owned()],
            port: 8080,
        })
    );
}

#[test]
fn parameter_source_identifies_a_late_field_failure() {
    let result = AllFieldSettings::from_environment(MapEnvironment::from_pairs([
        ("NAME", "gateway"),
        ("ENABLED", "yes"),
        ("COUNT", "42"),
        ("RATIO", "0.25"),
        ("METADATA", "null"),
        ("ENCODED", "aW5wdXQ="),
        ("MODE", "ready"),
        ("LABELS", "public"),
        ("PORT", "65536"),
    ]))
    .map_err(|error| (error.error_code(), error.variable_name().to_owned()));
    assert_eq!(result, Err(("parse_variable", "PORT".to_owned())));
}
