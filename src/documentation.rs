//! Test the published Markdown directly, without exposing additional API items.
//!
//! Register new guides here, including guides that currently contain only
//! non-Rust code blocks, so future Rust examples in them are also checked.

#[doc = include_str!("../README.md")]
mod readme {}

#[doc = include_str!("../docs/api-guide.md")]
mod api_guide {}

#[doc = include_str!("../docs/architecture.md")]
mod architecture {}

#[doc = include_str!("../docs/binding-contract-matrix.md")]
mod binding_contract_matrix {}

#[doc = include_str!("../docs/dependencies.md")]
mod dependencies {}

#[doc = include_str!("../docs/fuzzing.md")]
mod fuzzing {}

#[doc = include_str!("../docs/migrating-to-0.2.md")]
mod migrating_to_0_2 {}

#[doc = include_str!("../docs/open-source.md")]
mod open_source {}

#[doc = include_str!("../docs/publishing.md")]
mod publishing {}

#[doc = include_str!("../docs/release-checklist.md")]
mod release_checklist {}

#[doc = include_str!("../docs/release-controls.md")]
mod release_controls {}

#[doc = include_str!("../docs/release-readiness.md")]
mod release_readiness {}

#[doc = include_str!("../docs/testing-guide.md")]
mod testing_guide {}
