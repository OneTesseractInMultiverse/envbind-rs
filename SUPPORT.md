# Support

## Supported Platforms

The supported CI baseline is current stable Rust on these native targets:

| Operating system | Rust target | CI runner |
| --- | --- | --- |
| Linux (GNU) | `x86_64-unknown-linux-gnu` | `ubuntu-latest` |
| macOS (Apple silicon) | `aarch64-apple-darwin` | `macos-latest` |
| Windows (MSVC) | `x86_64-pc-windows-msvc` | `windows-latest` |

Each stable job runs all target tests, including the process-environment
subprocess fixtures, doctests, Clippy, documentation builds, and packaging.
CI also checks the declared minimum Rust version, **1.85**, on Linux. Other
operating systems, architectures, C libraries, and older OS releases may work
but are outside this tested baseline. Hosted runner images can change; the
workflow logs report the actual image and `rustc --version --verbose` host.

Process names are case-sensitive on Unix and case-insensitive on Windows.
Names must be nonempty and contain neither `=` nor NUL. Unicode names and
values are supported. Values that cannot be represented as Rust strings return
an adapter error; they are never lossily decoded or treated as missing. Tests
use invalid UTF-8 bytes on Unix and an unpaired UTF-16 surrogate on Windows.
Map and custom adapters retain their own namespace rules. See the
[process contract](docs/api-guide.md#process-environment-names) and
[test instructions](docs/testing-guide.md#process-environment-tests).

## Getting Help

Use [GitHub Issues](https://github.com/OneTesseractInMultiverse/envbind-rs/issues)
for public support. Search existing issues first, then choose the matching form:

- [Usage question](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/new?template=question.yml): include the crate version, intended configuration, and a small example using synthetic values.
- [Bug report](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/new?template=bug_report.yml): include a reproducible case and expected behavior.
- [Feature request](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/new?template=feature_request.yml): explain the use case, proposal, and current alternatives.

Report suspected vulnerabilities privately to security@subvertic.com. Do not
open a public issue for a vulnerability.
