#![allow(missing_docs)]

use envbind::{Environment, MapEnvironment};

#[test]
fn empty_map_environment_returns_none() {
    let value = MapEnvironment::new().get("MISSING");

    assert_eq!(value, Ok(None));
}

#[test]
fn map_environment_from_pairs_returns_value() {
    let value = MapEnvironment::from_pairs([("HOST", "localhost")]).get("HOST");

    assert_eq!(value, Ok(Some("localhost".to_owned())));
}

#[test]
fn map_environment_insert_replaces_value() {
    let mut environment = MapEnvironment::new();

    environment.insert("HOST", "mongodb");

    assert_eq!(environment.get("HOST"), Ok(Some("mongodb".to_owned())));
}
