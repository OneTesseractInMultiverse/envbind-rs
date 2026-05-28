# Release Checklist

Use this checklist before publishing a new crate version. See
[Publishing](publishing.md) for the GitHub Actions release workflow.

## Before Tagging

1. Update `version` in `Cargo.toml`.
2. Update `CHANGELOG.md`.
3. Confirm `repository` and `documentation` in `Cargo.toml`.
4. Confirm public API changes in `README.md` and `/docs`.
5. Confirm a clean working tree.
6. Run `make verify`.
7. Run `make package-list`.
8. Run `make package`.
9. Run `make publish-dry-run`.
10. Confirm the RustSec advisory audit passes in CI.
11. Confirm CI passes on stable Rust and the declared minimum Rust version.

## Package Validation

```sh
make verify
make package-list
make package
make publish-dry-run
```

The package includes source, tests, examples, README, docs, and the MIT license.
It excludes build output, editor metadata, secrets, and machine-specific files.

`make package` and `make publish-dry-run` reject dirty working trees. Commit or
stash local changes first.

## Versioning

Use semantic versioning:

- Patch releases fix behavior without public API changes.
- Minor releases add compatible APIs.
- Major releases can change public API or error contracts.

`BindError::error_code()` values are compatibility-sensitive. Treat changes to
existing codes as breaking changes. A clear bug fix is the only exception.
