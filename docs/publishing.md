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
shell script. Both jobs check out the validated `refs/tags/...` ref, verify
that it points to the checked-out commit, and compare its version with the
manifest before installing release tools or authenticating with crates.io.
Annotated and lightweight tags are supported. A branch with the same name
cannot substitute for a missing tag.

CI runs local regression tests for the release workflow, including hostile
tag inputs and manual validation-only behavior. See the
[workflow test instructions](testing-guide.md#release-workflow-tests).

## Trusted Publishing

The publishing workflow uses crates.io Trusted Publishing through
`rust-lang/crates-io-auth-action`. The workflow pins third-party actions to
commit SHAs. It requests a short-lived crates.io token through GitHub OpenID
Connect, then passes that token to `cargo publish`.

Configure the `envbind` crate on crates.io with these repository values:

| Setting | Value |
| --- | --- |
| GitHub owner or organization | `OneTesseractInMultiverse` |
| Repository | `envbind-rs` |
| Workflow | `publish.yml` |
| Environment | `crates-io` |

The GitHub repository uses a `crates-io` environment. Add required reviewers
there for explicit release approval.

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
`Publish` workflow.

## Routine Release

Update `version` in `Cargo.toml`, then update `CHANGELOG.md`. Confirm the
`repository` and `documentation` metadata. Run `make verify`, inspect
`make package-list`, run `make package`, and run `make publish-dry-run`.

Commit the release changes. Tag the commit with a version tag, such as
`git tag v0.1.0`. Push the branch and tag, then create a GitHub release from
the tag.

The `Publish` workflow validates, packages, uploads the `.crate` archive, runs
a publish dry run, and publishes to crates.io from the protected `crates-io`
environment.
