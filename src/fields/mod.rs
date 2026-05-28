//! Typed variable specs used by settings objects.

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
