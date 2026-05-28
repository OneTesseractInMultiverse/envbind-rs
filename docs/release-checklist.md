# Release Checklist

Use this checklist before publishing a new crate version. See
[Publishing](publishing.md) for the GitHub Actions release workflow.

## Before Tagging

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

Confirm that the package name in `Cargo.toml` is
`onetesseractinmultiverse-envbind`. The Rust library name remains `envbind`.

## Versioning

Use semantic versioning. Patch releases fix behavior without public API
changes. Minor releases add compatible APIs. Major releases can change public
API or error contracts.

`BindError::error_code()` values are compatibility-sensitive. Treat changes to
existing codes as breaking changes. A clear bug fix is the only exception.

## Final Checks

Confirm the release tag matches the package version. Use
`vMAJOR.MINOR.PATCH`, such as `v0.1.0`.

Confirm that `SECURITY.md` lists the private report address. Confirm that
README links use relative paths for repository docs.
