#![allow(missing_docs)]

use envbind::{BindError, Binder, Binding, Environment, MapEnvironment};

struct EchoBinding;

impl Binding<String> for EchoBinding {
    fn bind<E: Environment>(&self, environment: &E) -> Result<String, BindError> {
        environment
            .get("NAME")
            .map_err(|source| BindError::environment("NAME", source))?
            .ok_or_else(|| BindError::missing("NAME"))
    }
}

#[test]
fn binder_returns_underlying_environment() {
    let value = Binder::new(MapEnvironment::from_pairs([("NAME", "api")]))
        .environment()
        .get("NAME");

    assert_eq!(value, Ok(Some("api".to_owned())));
}

#[test]
fn binder_delegates_to_binding_spec() {
    let value = Binder::new(MapEnvironment::from_pairs([("NAME", "api")])).bind(&EchoBinding);

    assert_eq!(value, Ok("api".to_owned()));
}
