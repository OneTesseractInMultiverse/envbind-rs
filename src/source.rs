//! Trait implemented by service-specific settings objects.

use crate::binder::Binder;
use crate::environment::{Environment, ProcessEnvironment};
use crate::error::BindError;

/// Build a settings object from an environment source.
pub trait ParameterSource: Sized {
    /// Bind the settings object through a binder.
    fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, BindError>;

    /// Build the settings object from an arbitrary environment source.
    fn from_environment<E: Environment>(environment: E) -> Result<Self, BindError> {
        Self::bind(&Binder::new(environment))
    }

    /// Build the settings object from the process environment.
    fn from_process_environment() -> Result<Self, BindError> {
        Self::from_environment(ProcessEnvironment)
    }
}
