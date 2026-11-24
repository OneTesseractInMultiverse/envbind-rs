# Release Checklist

Use this checklist before publishing a new crate version. See
[Publishing](publishing.md) for the GitHub Actions release workflow.

## Before Tagging

Run the [release control preflight](release-controls.md#preflight) with an
authenticated GitHub CLI, and compare the crates.io Trusted Publisher row with
the documented repository, workflow, and environment. Resolve drift before
creating a release tag. Release tags cannot be updated or deleted under the
normal protection policy.

The published 0.2.0 release is documented in the
[readiness record](release-readiness.md) and
[migration guide](migrating-to-0.2.md). It contains stricter URL and process-name
validation and therefore used a new pre-1.0 minor line. The former 0.1.1
metadata change was never published. Recheck registry state and choose a new
unused version before preparing another release; do not reuse `v0.2.0`.

Update `version` in `Cargo.toml`, the local package entry in `fuzz/Cargo.lock`,
and `CHANGELOG.md`. Confirm that
`repository` and `documentation` metadata point to the public project
locations. Confirm that public API changes appear in `README.md` and `/docs`.

Commit and review the release changes first. Run the local gates from a clean
checkout of the selected commit, using a reviewed stable compiler and the
cargo-audit version in [dependency maintenance](dependencies.md). Generate one
release resolution, audit it, then keep that same lockfile for every check:

```sh
cargo generate-lockfile
cargo audit --deny warnings --json > /tmp/envbind-release-audit.json
make verify CARGO='cargo --locked'
make package-list CARGO='cargo --locked'
make package CARGO='cargo --locked'
cargo publish --dry-run --locked
```

Stop on any failure. Retain `Cargo.lock`, the audit report, the package archive,
the full source commit, and the tool versions with the candidate evidence.
Do not regenerate the lock between auditing and packaging. A new source,
metadata, dependency, or toolchain revision requires new validation.

Inspect the package list. The package includes source, tests, examples, README,
docs, and the MIT license. It excludes build output, editor metadata, secrets,
and machine-specific files.

Confirm that CI passes on stable Rust and on the declared minimum Rust version.
Confirm that the RustSec advisory audit passes for both fresh root resolutions
and the committed fuzz lock, and that package documentation examples pass as
described in the [readiness gates](release-readiness.md#final-candidate-gates).

Review the [dependency maintenance guidance](dependencies.md), including the
manually pinned cargo-audit version in both workflows and any MSRV constraints.

Confirm that the package name and Rust library name are both `envbind`.

## Versioning

Use semantic versioning. Patch releases fix behavior without public API
changes. Minor releases add compatible APIs. Major releases can change public
API or error contracts.

Before 1.0, an incompatible behavior policy uses a new `0.y` minor line;
compatible fixes can use `0.y.z` patch releases. Make the decision explicit in
the changelog and migration notes instead of relying on the pre-1.0 label.

`BindError::error_code()` values are compatibility-sensitive. Treat changes to
existing codes as breaking changes. A clear bug fix is the only exception.

## Final Checks

Confirm the release tag matches the package version. Use
`vMAJOR.MINOR.PATCH`, such as `v0.2.0`, with any SemVer prerelease or build
suffix matching `Cargo.toml` exactly. The `v` prefix is required. Manual runs
also accept `refs/tags/v0.2.0`; the ref must exist as a tag, not just a branch.
Confirm that the release workflow regression tests pass in CI.

For manual publication, dispatch the workflow from `main` or the intended
version tag. The environment checks that workflow ref, independently of the
`tag` input. Confirm that publication waits for the configured reviewer;
administrator bypass is disabled and self-review is intentionally allowed for
the sole maintainer.

Before approving publication, inspect the validation run's release bundle.
Confirm that `release.json` identifies the intended full commit SHA, tag,
package version, Rust toolchain, and audit results. Publishing uses that commit
and audited lockfile even if the tag later moves. The rebuilt package must
match the validated archive before authentication. If artifacts expire or
verification fails, rerun validation instead of bypassing the integrity checks.

Confirm that `SECURITY.md` lists the private report address. Confirm that
README links use relative paths for repository docs.

Keep new versions marked unreleased during candidate review. Record the
intended date in the final release-preparation commit and validate that exact
source before tagging. Keep availability claims conditional until registry
publication is confirmed; then record the actual publication date and update
installation and supported-version wording. Preserve the immutable release
source if the preparation and publication dates differ. Earlier artifacts do
not inherit validation results from a newer release.
