//! Typed variable specs used by settings objects.
//!
//! # Fallback validation
//!
//! Every field skips validators for typed defaults unless `.validate_default()`
//! is enabled. With it, a selected fallback runs the same attached validators
//! as a parsed value, once each in registration order until the first failure.
//! Failures use `validation_failed` and the field's sensitivity policy. Builder
//! order does not matter; validation happens on each binding, not construction.
//!
//! Missing input selects a configured default. Explicit empty input also selects
//! it unless `.allow_empty()` is enabled, in which case the empty input is
//! parsed and validated. Invalid present input never selects a default.
//! [`OptionalStringVar`] only validates `Some` values. [`BindingExt::optional`]
//! preserves default validation failures and wraps successful defaults in `Some`.
//!
//! Defaults are already typed: they are not parsed, decoded, or serialized.
//! Raw byte limits, decoded byte limits, list item limits, and enum label lookup
//! apply only to environment input. Even an empty string default is passed
//! directly to attached validators, regardless of `.allow_empty()`. Attach
//! validators for any length, membership, or other constraints that must also
//! apply to defaults. With no attached validators, enabling this policy has no
//! effect.
//!
//! ```
//! use envbind::{Binder, MapEnvironment, U16Var, validators};
//!
//! let port = U16Var::new("PORT")
//!     .default(0)
//!     .validate_default()
//!     .validate(validators::u16_in_range(1, 65_535));
//! let results = [MapEnvironment::new(), MapEnvironment::from_pairs([("PORT", "0")])]
//!     .map(|environment| Binder::new(environment).bind(&port).map_err(|e| e.error_code()));
//! assert_eq!(results, [Err("validation_failed"), Err("validation_failed")]);
//! ```
//!
//! [`BindingExt::optional`]: crate::BindingExt::optional

mod b64;
mod bool;
mod enumeration;
mod float;
mod int;
mod json;
mod list;
mod raw;
mod string;
mod u16;

pub use b64::B64DecodedStringVar;
pub use bool::BoolVar;
pub use enumeration::EnumVar;
pub use float::FloatVar;
pub use int::IntVar;
pub use json::JsonVar;
pub use list::ListVar;
pub use string::{OptionalStringVar, StringVar};
pub use u16::U16Var;
