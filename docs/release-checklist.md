# Release Checklist

Use this checklist before publishing a new crate version. See
[Publishing](publishing.md) for the GitHub Actions release workflow.

## Before Tagging

Run the [release control preflight](release-controls.md#preflight) with an
authenticated GitHub CLI, and compare the crates.io Trusted Publisher row with
the documented repository, workflow, and environment. Resolve drift before
creating a release tag. Release tags cannot be updated or deleted under the
normal protection policy.

For the upcoming 0.2.0 release, use the
[readiness record](release-readiness.md) and
[migration guide](migrating-to-0.2.md). This release contains stricter URL and
process-name validation; do not publish it as a 0.1.x patch. The published
registry currently contains 0.1.0, and the former 0.1.1 metadata change was
never published. Recheck registry state before selecting a release tag.

Update `version` in `Cargo.toml` and update `CHANGELOG.md`. Confirm that
`repository` and `documentation` metadata point to the public project
locations. Confirm that public API changes appear in `README.md` and `/docs`.

Run the local gates in order:

```sh
make verify
make package-list
make package
make publish-dry-run
```

Inspect the package list. The package includes source, tests, examples, README,
docs, and the MIT license. It excludes build output, editor metadata, secrets,
and machine-specific files.

Confirm that CI passes on stable Rust and on the declared minimum Rust version.
Confirm that the RustSec advisory audit passes. Commit or stash local changes
before release commands that reject dirty working trees.

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

Keep 0.2.0 marked unreleased until publication. At the release step, record the
release date, confirm registry availability before presenting installation as
available, and update the supported-version table. Do not label the old
published 0.1.0 artifact as covered by the new candidate's validation.
