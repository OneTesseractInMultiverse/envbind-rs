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

Explain each new dependency. Include its purpose, maintenance cost, license,
and role in parser or validator behavior.

## Pull Request Checklist

Run these commands before opening a pull request:

```sh
make fmt
make lint
make test
make doc
make package
```

The code must compile without unsafe code and without Clippy warnings.
