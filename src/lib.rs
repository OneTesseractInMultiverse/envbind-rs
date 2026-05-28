//! Typed environment binding primitives for Rust services.
//!
//! Envbind is the configuration binding library under the Subvertic namespace.
//! It is intended for application-boundary configuration in services that
//! follow hexagonal architecture. Domain and use-case code should receive typed
//! settings from the boundary instead of reading process environment variables
//! directly.
//!
//! The public API is deliberately small: environment readers, a binder
//! coordinator, typed variable specs, reusable validators, and a trait for
//! settings structs.
//!
//! # Core API
//!
//! - [`Environment`] abstracts raw value lookup.
//! - [`ProcessEnvironment`] reads from the process environment.
//! - [`MapEnvironment`] provides deterministic in-memory values for tests.
//! - [`Binder`] coordinates one [`Binding`] against an environment source.
//! - [`BindingExt::optional`] turns any binding spec into an optional binding.
//! - [`ParameterSource`] lets application settings structs compose typed values.
//! - [`StringVar`], [`IntVar`], [`FloatVar`], [`BoolVar`], [`ListVar`],
//!   [`JsonVar`], [`EnumVar`], [`B64DecodedStringVar`], and [`U16Var`] parse
//!   supported field types.
//! - [`BindError`] reports binding failures without exposing raw values.
//!
//! Values are treated as sensitive by default. Raw inputs have defensive size
//! limits, and larger JSON, base64, string, or list values must opt in through
//! field-specific limit setters.
//!
//! # Feature Flags
//!
//! Envbind currently has no Cargo feature flags. Runtime dependencies are kept
//! focused on parser and validator support for base64, JSON, regular
//! expressions, and URLs.
//!
//! # Example
//!
//! ```
//! use envbind::{
//!     Binder, BoolVar, Environment, MapEnvironment, ParameterSource, StringVar, U16Var,
//!     validators,
//! };
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct ServiceSettings {
//!     host: String,
//!     port: u16,
//!     service_name: String,
//!     tracing_enabled: bool,
//! }
//!
//! impl ParameterSource for ServiceSettings {
//!     fn bind<E: Environment>(binder: &Binder<E>) -> Result<Self, envbind::BindError> {
//!         Ok(Self {
//!             host: binder.bind(&StringVar::new("SERVICE_HOST").default("127.0.0.1"))?,
//!             port: binder.bind(
//!                 &U16Var::new("SERVICE_PORT")
//!                     .default(8080)
//!                     .validate(validators::u16_in_range(1, 65_535)),
//!             )?,
//!             service_name: binder.bind(&StringVar::new("SERVICE_NAME").default("api"))?,
//!             tracing_enabled: binder.bind(&BoolVar::new("SERVICE_TRACING").default(false))?,
//!         })
//!     }
//! }
//!
//! let environment = MapEnvironment::from_pairs([
//!     ("SERVICE_HOST", "localhost"),
//!     ("SERVICE_PORT", "9090"),
//! ]);
//!
//! let settings = ServiceSettings::from_environment(environment)?;
//! assert_eq!(settings.port, 9090);
//! # Ok::<(), envbind::BindError>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unwrap_used)]
#![deny(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod binder;
pub mod environment;
pub mod error;
pub mod fields;
pub mod source;
pub mod validators;

pub use binder::{Binder, Binding, BindingExt, OptionalVar};
pub use environment::{Environment, MapEnvironment, ProcessEnvironment};
pub use error::{BindError, EnvironmentError, ValidationError, VariableName};
pub use fields::{
    B64DecodedStringVar, BoolVar, EnumVar, FloatVar, IntVar, JsonVar, ListVar, OptionalStringVar,
    StringVar, U16Var,
};
pub use source::ParameterSource;
