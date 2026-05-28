# Publishing

This project publishes to crates.io from GitHub Actions.

The manifest includes docs.rs metadata. Hosted docs build all features and use
the default docs.rs target. Rustdoc receives the `docsrs` cfg.

## Release Flow

- Pull requests and pushes to `main` run the `CI` workflow.
- CI runs a RustSec dependency audit.
- Published GitHub releases run the `Publish` workflow.
- Manual `Publish` runs can validate a tag.
- Manual `Publish` runs can publish with the `publish` input.

Release tags match the package version in `Cargo.toml`. Use
`vMAJOR.MINOR.PATCH`, such as `v0.1.0`.

## Trusted Publishing

The publishing workflow uses crates.io Trusted Publishing through
`rust-lang/crates-io-auth-action`. The workflow pins third-party actions to
commit SHAs. It requests a short-lived crates.io token through GitHub OpenID
Connect, then passes that token to `cargo publish`.

For routine releases, configure the crate on crates.io with these values:

- GitHub owner or organization: public repository owner
- repository: public repository name
- workflow: `publish.yml`
- environment: `crates-io`

The GitHub repository uses a `crates-io` environment. Add required reviewers
there for explicit release approval.

## First Publish

crates.io Trusted Publishing starts after the crate exists on crates.io. For
the first release, publish manually from a clean tree:

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

1. Update `version` in `Cargo.toml`.
2. Update `CHANGELOG.md`.
3. Confirm `repository` and `documentation` metadata in `Cargo.toml`.
4. Run `make verify`.
5. Run `make package-list` and inspect included files.
6. Run `make publish-dry-run`.
7. Commit the release changes.
8. Tag the commit, such as `git tag v0.1.0`.
9. Push the branch and tag.
10. Create a GitHub release from the tag.

The `Publish` workflow validates, packages, uploads the `.crate` archive, runs
a publish dry run, and publishes to crates.io from the protected `crates-io`
environment.
