# Publishing

This project publishes to crates.io from GitHub Actions. The crates.io package
publishes as `envbind`. Rust code imports the crate as `envbind`. The GitHub
repository is `OneTesseractInMultiverse/envbind-rs`.

The manifest contains the package name, repository URL, docs.rs URL, license,
author metadata, and crate include list.

The docs.rs metadata builds all features and uses the default docs.rs target.
Rustdoc receives the `docsrs` cfg, so code can document docs.rs-specific
conditions without affecting normal builds.

The package include list ships source, tests, examples, README, `/docs`,
project policy files, `Makefile`, `SUPPORT.md`, and `LICENSE`. It leaves out build
output and local machine files.

## Release Flow

Pull requests and pushes to `main` run the `CI` workflow. CI checks formatting,
type checking, Clippy, tests, doc tests, docs, package contents, and RustSec
advisories.

Published GitHub releases run the `Publish` workflow. Manual `Publish` runs can
validate a tag without publishing. Manual runs can publish with the `publish`
input after approval through the protected environment.

Release tags match the package version in `Cargo.toml`. Use
`vMAJOR.MINOR.PATCH`, such as `v0.1.1`. SemVer prerelease and build suffixes are
supported, for example `v1.2.3-rc.1+build.001`. Manual runs also accept the fully
qualified form `refs/tags/v0.1.1`. The `v` prefix is required; bare versions,
branch names, commit hashes, and malformed versions are rejected.

A separate job validates the tag before either checkout. Tag inputs travel
through environment variables and are read as data, never inserted into a
shell script. Validation checks out the fully qualified `refs/tags/...` ref,
verifies that it points to the checked-out commit, and compares its version
with the manifest. It records the full commit SHA for publishing. Annotated
and lightweight tags are supported. A branch with the same name cannot
substitute for a missing tag.

CI runs local regression tests for the release workflow, including hostile
tag inputs and manual validation-only behavior. See the
[workflow test instructions](testing-guide.md#release-workflow-tests).

## Validated Release Inputs

Validation generates `Cargo.lock` once, audits that resolution, and retains
the exact file. Subsequent Cargo checks, tests, documentation, packaging, and
publish dry runs use `--locked`. A changed or missing resolution fails instead
of silently selecting new dependencies. The library's lockfile remains ignored
by Git; it is preserved as a release artifact rather than committed.

Each successful validation uploads an immutable artifact named
`envbind-release-<run-id>-<attempt>`, containing:

| File | Purpose |
| --- | --- |
| `Cargo.lock` | The exact audited dependency resolution. |
| `audit.json` | RustSec audit results, including advisory database identity. |
| `package.crate` | The validated crate archive, including the audited lockfile. |
| `release.json` | Tag, full commit SHA, package name/version, Rust and Cargo versions, run identity, and SHA-256 hashes of the other files. |

The validation job exports the commit SHA, exact Rust version, artifact ID,
and the SHA-256 hash of `release.json`. Publishing checks out that SHA directly
and downloads the artifact by ID from the same workflow run. It does not
resolve the tag again. Moving or deleting a tag while environment approval is
pending cannot substitute another commit; if the recorded commit or artifact
is unavailable, the run fails.

Before authentication, publishing verifies the metadata hash and each artifact
hash, checks source identity and package version, restores the audited lockfile,
and installs the recorded Rust version. It rebuilds with `cargo package
--locked` and requires the resulting archive to match the validated archive
byte for byte. Rust and Cargo versions must also match validation.

Cargo's stable publishing command recreates the archive rather than accepting
an existing `.crate` file. After the protected environment approval and archive
comparison, Trusted Publishing supplies the token. A final integrity check
runs immediately before `cargo publish --locked --no-verify`. The same source,
lockfile, and toolchain reproduce the validated package; `--no-verify` skips
another build-script execution because the package was already built and
verified before authentication. See the official
[Cargo publishing behavior](https://doc.rust-lang.org/cargo/commands/cargo-publish.html).

Any checksum mismatch, dependency drift, toolchain mismatch, or unavailable
artifact stops publication. Rerun validation to produce a new release bundle;
do not regenerate a lockfile or bypass the comparison in the publishing job.
Review the run's `release.json` and audit results when approving a release.

## Trusted Publishing

The publishing workflow uses crates.io Trusted Publishing through
`rust-lang/crates-io-auth-action`. The workflow pins third-party actions to
commit SHAs. It requests a short-lived crates.io token through GitHub OpenID
Connect, then passes that token to `cargo publish`.

The `envbind` crate's Trusted Publisher must match these values exactly:

| Setting | Value |
| --- | --- |
| GitHub owner or organization | `OneTesseractInMultiverse` |
| Repository | `envbind-rs` |
| Workflow | `publish.yml` |
| Environment | `crates-io` |

The GitHub repository's `crates-io` environment requires approval by
`OneTesseractInMultiverse`. Self-review is allowed because the repository has
one maintainer; this is an explicit confirmation gate, not independent review
by a second person. Administrator bypass is disabled. The environment permits
the `main` branch and tags matching `v*`; it rejects other branches and tags.

For manual dispatch, select `main` or a permitted version tag as the **workflow
ref**. The separate `tag` input selects the source to validate and publish; it
does not control the environment's ref check. A run dispatched from an issue
branch cannot publish even if its tag input is valid. Release events use the
release tag as the workflow ref. The workflow performs the stricter version
and manifest checks described above; the environment's `v*` glob is not a
SemVer validator.

The active `Protect release tags` ruleset blocks updates and deletion for
`refs/tags/v*`, with no standing bypass actors. Tag creation remains permitted.
Use a new version for a correction; do not move an existing release tag.

These environment, ruleset, and registry settings are external configuration.
The workflow's `environment:` declaration alone does not install protections
or configure Trusted Publishing. Before each release, run the read-only
preflight and inspect the registry row using the
[release control runbook](release-controls.md). That runbook also documents
configuration recovery and safe approval-gate testing.

## First Publish

crates.io Trusted Publishing starts after the crate exists on crates.io. The
first release uses a manual publish from a clean tree.

```sh
make verify
make package-list
make package
make publish-dry-run
cargo publish
```

After the first version appears on crates.io, configure Trusted Publishing in
the crate settings. Later versions publish through the GitHub Actions
`Publish` workflow. The `envbind` crate already exists; this bootstrap procedure
is only for a new registry package, not routine releases.

## Routine Release

Update `version` in `Cargo.toml`, then update `CHANGELOG.md`. Confirm the
`repository` and `documentation` metadata. Run `make verify`, inspect
`make package-list`, run `make package`, and run `make publish-dry-run`.

Complete the [external release control preflight](release-controls.md#preflight).
Commit the release changes. Tag the commit with a version tag, such as
`git tag v0.1.0`. Push the branch and tag, then create a GitHub release from
the tag.

The `Publish` workflow validates and audits the release, runs a locked publish
dry run, and uploads its release bundle. The publishing job verifies the
recorded commit and artifacts, reproduces the package, and publishes from the
protected `crates-io` environment.
