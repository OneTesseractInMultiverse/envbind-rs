# Contributing

Contributions are welcome.

## Development Setup

Install a stable Rust toolchain that supports the crate's `rust-version`. Then
run the local quality gate:

```sh
make verify
```

## Contribution Rules

- Keep changes focused and small.
- Preserve the coordinator-versus-computation split in `docs/architecture.md`.
- Add or update tests for every behavior change.
- Update docs for public API, examples, or error behavior changes.
- Keep unit tests free of external configuration.
- Explain each new dependency and its maintenance cost.

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
