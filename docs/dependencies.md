# Dependency Maintenance

The library supports Rust 1.85 and current stable Rust. Keep runtime dependency
requirements compatible with that minimum and review CI tooling separately.
`Cargo.lock` remains untracked; release validation preserves its generated,
audited lockfile in the release artifact.

## Review on 2026-09-26

Rechecked every resolved registry package against the latest non-yanked stable
release on crates.io, plus the pinned tools, PyPI requirements, and GitHub
Actions releases. The runtime requirements still resolve base64 0.23.1,
regex 1.13.1, serde_json 1.0.151, and url 2.5.8. Proptest 1.11.0,
libfuzzer-sys 0.4.13, cargo-fuzz 0.13.2, cargo-audit 0.22.2, and PyYAML 6.0.3
remain current. All four action versions and full commit pins below were
reverified and remain current.

Refreshed the committed fuzz lockfile to cc 1.5.1, find-msvc-tools 0.1.14, and
smallvec 1.16.2. Both fresh root resolutions contain 64 registry packages; the
fuzz workspace contains 56. All three graphs passed cargo-audit 0.22.2 without
vulnerability findings or maintenance warnings. Root MSRV tests passed with
the refreshed compatible graph.

The remaining older selections are deliberate compatibility constraints:

- An isolated fresh build of yoke-derive 0.8.3 still fails on Rust 1.85 at
  `str::from_utf8`. Keep the 0.8.2 constraint until upstream fixes its compiler
  compatibility or an intentional MSRV change is made. Its syn 2 and
  synstructure 0.13 dependencies cannot be replaced with their newer major
  versions by a lockfile refresh.
- The latest proptest still requires rand/rand_core/rand_chacha 0.9 and
  rand_xorshift 0.4. That chain selects getrandom 0.3, r-efi 5, wasip2 1, and
  compatible wit-bindgen releases. Their newest incompatible release lines
  need upstream adoption; adding direct pins would not migrate these users.
- The MSRV resolver retains the older ICU/idna versions described below and
  wasip2 1.0.1+wasi-0.2.4 with wit-bindgen 0.46.0. The stable graph selects
  wasip2 1.0.4+wasi-0.2.12 with wit-bindgen 0.57.1. Wasip2 1.0.4 requires
  Rust 1.87; its latest 2.x line is outside getrandom's requirement.
- The fuzz workspace's getrandom 0.4 selects r-efi 6 rather than the latest
  incompatible 7.x line. This target-specific dependency is outside the normal
  library runtime graph.

Keep the tested `nightly-2026-09-24` fuzz compiler pin for reproducible runs;
nightly snapshots are reviewed tooling choices, not stable dependency releases.
The library's Rust 1.85 minimum is unchanged.

## Earlier Review on 2026-09-22

Versions were checked against crates.io, PyPI, and upstream GitHub release tags.
Existing compatible requirements for regex, JSON, and URL parsing did not need
changes to resolve their latest stable releases.

| Dependency | Selected version | Review |
| --- | --- | --- |
| [base64](https://crates.io/crates/base64/0.23.1) | 0.23.1 | Updated from 0.22.1; requires Rust 1.71. |
| [regex](https://crates.io/crates/regex/1.13.1) | 1.13.1 | Already allowed by `regex = "1"`. |
| [serde_json](https://crates.io/crates/serde_json/1.0.151) | 1.0.151 | Already allowed by `serde_json = "1"`. |
| [url](https://crates.io/crates/url/2.5.8) | 2.5.8 | Current stable release. |
| [yoke-derive](https://crates.io/crates/yoke-derive/0.8.2) | 0.8.2 | Compatibility constraint; latest 0.8.3 fails on Rust 1.85. |
| [cargo-audit](https://crates.io/crates/cargo-audit/0.22.2) | 0.22.2 | Updated both workflows; its Rust 1.88 requirement applies only to tooling installed on stable. |
| [PyYAML](https://pypi.org/project/PyYAML/6.0.3/) | 6.0.3 | Current stable release; used only by workflow tests. |

Base64 0.23 adds SIMD engines behind a default feature. Envbind retains its
existing `general_purpose::STANDARD` decoder and selects only the `std`
feature, leaving the unused SIMD feature disabled. Padding, alphabet, UTF-8,
size limits, and redacted error behavior remain unchanged. Regression tests
pass with both 0.22.1 and 0.23.1. See the upstream
[base64 release notes](https://docs.rs/crate/base64/0.23.1/source/RELEASE-NOTES.md).

All action tags were resolved to their full commit SHAs:

| Action | Release | Commit |
| --- | --- | --- |
| actions/checkout | [v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1) | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| actions/upload-artifact | [v7.0.1](https://github.com/actions/upload-artifact/releases/tag/v7.0.1) | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` |
| actions/download-artifact | [v8.0.1](https://github.com/actions/download-artifact/releases/tag/v8.0.1) | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` (unchanged) |
| rust-lang/crates-io-auth-action | [v1.0.5](https://github.com/rust-lang/crates-io-auth-action/releases/tag/v1.0.5) | `c6f97d42243bad5fab37ca0427f495c86d5b1a18` |

These actions use Node 24 on GitHub-hosted `ubuntu-latest` runners. Checkout
retains `persist-credentials: false`; the workflows do not use the
`pull_request_target` or `workflow_run` triggers affected by checkout v7's new
restrictions. Artifact upload explicitly uses `archive: true` for the release
bundle's four files. Download v8 continues to extract a single artifact ID
directly into `target/release`. Permissions and the protected publishing gate
remain unchanged.

## Dependency Resolution and Remaining Constraints

CI checks two fresh dependency resolutions:

- Rust 1.85 uses `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback` to select
  releases compatible with the declared minimum.
- Stable uses `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=allow` to exercise the
  newest releases permitted by the manifest, including dependencies requiring
  a newer compiler. The stable compiler checked in this review was Rust 1.98.1.

The security job audits both resolutions. Release validation retains Cargo's
MSRV-aware default resolution and audits that exact lockfile before packaging.
Both graphs reviewed on 2026-09-22 contained 46 registry packages and passed cargo-audit 0.22.2
without vulnerability findings or maintenance warnings.

The stable dependency graph is now exercised by native Linux, macOS, and
Windows jobs. The required `Rust stable` aggregate check succeeds only after
the entire Rust matrix succeeds; `Rust 1.85.0` remains the dedicated Linux MSRV
check. See [platform CI](testing-guide.md#platform-ci-and-required-checks).

Remaining constraints are explicit:

- `yoke-derive = "=0.8.2"` is retained. Version 0.8.3 calls `str::from_utf8`,
  which does not compile on Rust 1.85, but does not declare the newer compiler
  requirement. A fresh build without the constraint reproduced this error.
  Remove the workaround once a compatible upstream release passes the MSRV
  job, or as part of an intentional, documented MSRV change.
- That macro still requires syn 2.0.119 and synstructure 0.13.2. The graphs also
  contain current syn 3.0.6 and synstructure 0.14.0 for other macros; Cargo
  cannot replace a dependency with a semver-incompatible major version.
- The MSRV graph uses ICU collections, locale core, normalizer, normalizer data,
  and provider 2.1.1, properties and properties data 2.1.2, and idna_adapter
  1.2.1. Stable exercises ICU 2.3.0 (provider 2.3.1) and idna_adapter 1.2.2.
  The newer ICU packages require Rust 1.88, and idna_adapter 1.2.2 requires 1.86.
  These older MSRV selections are resolver choices, not additional pins.

Every other resolved registry package was checked against its latest stable
release during this review. Regenerate and inspect the graph when updating;
the review date does not freeze future dependency resolution.

## Future Updates

Property tests use proptest 1.11 as a development-only dependency, with only
`std` enabled to avoid subprocess/fork tooling. It supports Rust 1.85 and uses
the MIT/Apache-2.0 license. The separate non-publishable `fuzz` workspace pins
libfuzzer-sys 0.4.13 and retains its own lockfile; its LLVM runtime adds the NCSA
license alongside MIT/Apache-2.0. Both choices and maintenance costs are
documented in the [fuzzing guide](fuzzing.md).

Dependabot also checks the `/fuzz` Cargo workspace weekly. Security CI audits
that committed lockfile. Review cargo-fuzz 0.13.2 and the pinned
`nightly-2026-09-24` toolchain manually with other CI tooling updates; these
pins are not Cargo dependencies managed by Dependabot. Update the workflow,
runner default, and guide together, rerun the bounded targets, and record the
new result. These tools do not change the library's minimum Rust version.

Dependabot checks Cargo, GitHub Actions, and the Python workflow test
requirements weekly. Keep action references pinned to full release commit SHAs
with matching version comments, and review major-version migration notes.

The `cargo install cargo-audit --locked --version ...` commands are not covered
by Dependabot's Cargo manifest updates. At each dependency refresh and before
a release, check the current stable cargo-audit release and update both
`ci.yml` and `publish.yml`. Install and run that exact version, review advisories
and warnings, and check that its required Rust version is supported by the
stable toolchain used for CI tooling.

For runtime updates, inspect `cargo tree`, test both resolution modes, and run
the local verification, package, and publish dry-run gates. Confirm the release
workflow tests and a validation-only workflow run before relying on upgraded
actions for publication. Never publish a package merely to test an upgrade.
