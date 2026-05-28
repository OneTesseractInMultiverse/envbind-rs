//! Binding coordinator for typed variable specs.

use crate::environment::Environment;
use crate::error::BindError;

/// Bind one typed value from an environment source.
pub trait Binding<T> {
    /// Load the value from the given environment.
    fn bind<E: Environment>(&self, environment: &E) -> Result<T, BindError>;
}

/// Extension helpers for typed binding specs.
pub trait BindingExt: Sized {
    /// Treat missing or explicitly empty values as `None`.
    ///
    /// Parsing and validation errors still return `Err`.
    #[must_use]
    fn optional(self) -> OptionalVar<Self> {
        OptionalVar { binding: self }
    }
}

impl<B> BindingExt for B {}

/// Optional wrapper for any binding spec.
pub struct OptionalVar<B> {
    binding: B,
}

impl<T, B> Binding<Option<T>> for OptionalVar<B>
where
    B: Binding<T>,
{
    fn bind<E: Environment>(&self, environment: &E) -> Result<Option<T>, BindError> {
        match self.binding.bind(environment) {
            Ok(value) => Ok(Some(value)),
            Err(BindError::MissingVariable { .. } | BindError::EmptyVariable { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

/// Coordinate value specs against one environment source.
pub struct Binder<E>
where
    E: Environment,
{
    environment: E,
}

impl<E> Binder<E>
where
    E: Environment,
{
    /// Build a binder from an environment source.
    #[must_use]
    pub fn new(environment: E) -> Self {
        Self { environment }
    }

    /// Bind one typed value through a spec.
    pub fn bind<T, B>(&self, binding: &B) -> Result<T, BindError>
    where
        B: Binding<T>,
    {
        binding.bind(&self.environment)
    }

    /// Return the underlying environment source.
    #[must_use]
    pub fn environment(&self) -> &E {
        &self.environment
    }
}
