//! Environment readers used by binders and settings objects.

use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Debug, Formatter};

use crate::error::EnvironmentError;

/// Return raw environment values by name.
pub trait Environment {
    /// Return the raw string value for a name when present.
    fn get(&self, name: &str) -> Result<Option<String>, EnvironmentError>;
}

/// Read values from the process environment.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessEnvironment;

impl Environment for ProcessEnvironment {
    fn get(&self, name: &str) -> Result<Option<String>, EnvironmentError> {
        match env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(env::VarError::NotUnicode(_)) => Err(EnvironmentError::not_unicode()),
        }
    }
}

/// Read values from an in-memory map.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct MapEnvironment {
    values: BTreeMap<String, String>,
}

impl Debug for MapEnvironment {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MapEnvironment")
            .field("len", &self.values.len())
            .finish()
    }
}

impl MapEnvironment {
    /// Build an empty in-memory environment.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an in-memory environment from key-value pairs.
    #[must_use]
    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values = pairs
            .into_iter()
            .map(|(name, value)| (name.into(), value.into()))
            .collect::<BTreeMap<String, String>>();
        Self { values }
    }

    /// Insert or replace one key-value pair.
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.values.insert(name.into(), value.into());
    }
}

impl Environment for MapEnvironment {
    fn get(&self, name: &str) -> Result<Option<String>, EnvironmentError> {
        Ok(self.values.get(name).cloned())
    }
}
