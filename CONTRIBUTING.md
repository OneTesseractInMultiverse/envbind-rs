# Contributing

Contributions are welcome. The project favors focused changes with clear tests
and clear documentation.

## Development Setup

Install a stable Rust toolchain that supports the crate's `rust-version`. Then
run the local quality gate:

```sh
make verify
```

This command checks formatting, type checking, Clippy, tests, doc tests, and
docs.rs-style docs.

## Contribution Rules

Keep changes focused and small. Preserve the coordinator-versus-computation
split in `docs/architecture.md`. Add or update tests for every behavior change.
Update docs for public API, examples, or error behavior changes.

Unit tests stay free of external configuration. They do not require files,
network access, local services, or process-specific environment variables.
Process-adapter integration tests use isolated child processes. The separate
workflow suite creates temporary repositories and executes local scripts; see
the [testing guide](docs/testing-guide.md).

Explain each new dependency. Include its purpose, maintenance cost, license,
and role in parser or validator behavior.

See [Dependency Maintenance](docs/dependencies.md) for version review, MSRV
constraints, action pins, and tooling updates.

## Pull Request Checklist

Open an issue before a large API change. Use the
[issue templates](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/new/choose)
for bugs, focused feature requests, and usage questions. See
[Support](SUPPORT.md#getting-help) for the private security-report channel.

Run these commands before opening a pull request:

```sh
make fmt
make verify
```

Commit the reviewed changes before packaging; Cargo rejects uncommitted source
changes. Then inspect and verify the package:

```sh
make package-list
make package
```

Run the [workflow regression tests](docs/testing-guide.md#release-workflow-tests)
when changing workflows, their scripts, or the documentation inventory. Lint
workflow changes with `actionlint` as described there. New Markdown guides must
be registered in `src/documentation.rs` so their Rust examples are tested.

The code must compile without unsafe code and without Clippy warnings.
