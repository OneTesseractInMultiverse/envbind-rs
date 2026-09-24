# Property Tests and Bounded Fuzzing

The regression matrix covers named contracts and exact boundaries. Generated
tests add combinations of Unicode, malformed syntax, limits, and valid values
through the same public `MapEnvironment` binding boundary. They do not read
production configuration, contact services, or use real credentials.

## Normal Test Gate

[properties.rs](../tests/properties.rs) contains 15 deterministic properties.
Fourteen run 128 cases each; the larger raw-list property runs 32 cases. They
use ChaCha with the fixed seed `0x7e1c20260918`, bounded shrinking (1,024
iterations), and no filesystem failure persistence, forking, or timeouts.
Case counts, seed, RNG, rejection limits, and shrinking settings are explicitly
configured instead of depending on developer environment settings.

Run them with stable Rust or the supported minimum compiler:

```sh
cargo test --test properties
cargo +1.85.0 test --test properties
make verify
```

The sole new development dependency is
[proptest 1.11](https://docs.rs/proptest/1.11.0/proptest/), licensed MIT or
Apache-2.0 and supporting Rust 1.85. Only its `std` feature is enabled.
Strategies and shrinking justify the additional test dependency; it is not a
library runtime dependency. Dependency updates must preserve the MSRV and
rerun these properties on every supported CI platform.

## Shared Properties and Bounds

[robustness.rs](../tests/support/robustness.rs) supplies invariant checks to both
the ordinary properties and the [fuzz targets](../fuzz/fuzz_targets). A panic
from a built-in parser or validator fails the test/run normally; it is not
caught and converted into a successful outcome.

| Area | Properties | Bounds |
| --- | --- | --- |
| JSON | Structured errors, byte limit before parsing/validation, optional-error preservation, generated object round trips, safe complete/incomplete nesting. | Arbitrary property text: 128 Unicode scalars (at most 512 bytes); generated depth: 0–256; fuzz payload: 4,096 bytes. Configured limits vary from zero to the input bound. |
| Base64 | Encoded-size precheck, decoded-byte cap on success, UTF-8 text round trips, arbitrary decoded bytes require UTF-8, optional-error preservation. | Property byte vectors: at most 256 bytes. Fuzz raw text: 4,096 bytes; derived round-trip encoding: at most 5,464 bytes. |
| Lists | Successful length stays within item cap; excess items never reach the custom parser; an empty delimiter invokes no parser; valid joined items round-trip at the exact cap. Typed-item failures remain errors through optional wrappers. | Item caps: 0–64. Property delimiters: 0–2 Unicode scalars (at most 8 bytes). Generated valid lists: 1–16 items, at most 16 characters each. |
| Raw list limit | Oversized raw input is rejected before any item callback and remains an error through optional wrapping. | Derived fixture: one MiB plus 1–256 bytes. The fuzz list target triggers this check only when its selector is zero. |
| URLs | Arbitrary text returns safely; forbidden whitespace, controls, and backslashes cannot succeed; generated HTTPS URLs preserve the original string and accept valid `u16` ports. | Fuzz payload: 4,096 bytes; generated host labels: at most 16 characters plus a safe prefix/suffix. No network lookup occurs. |
| Scalars and optional composition | Integer, boolean, finite-float, and `u16` round trips; arbitrary scalar text and enum labels preserve errors; string size rejection skips validation. | Ordinary arbitrary text: at most 512 bytes; fuzz payload: 4,096 bytes. Scalar round trips cover the full generated integer/port range. |
| Redaction | Synthetic markers from malformed input, parsed-value validators, URL credentials, and adapter diagnostics never appear in display/debug/source formatting. | The marker is injected into data only, never a variable name. Derived diagnostic inputs add a small fixed prefix to the bounded payload. |

Optional checks compare complete structured errors with the underlying binding;
only missing/empty errors become `None`. Successful float comparisons use their
debug representation to accommodate NaN without defining a finite-only policy.
Finite round trips compare exact bits, including signed zero. The policy in
[#19](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/19) remains
separate.

There is no guarantee that arbitrary caller-provided callbacks cannot panic.
The harness uses only known, bounded callbacks. It also does not claim that
every input substring must be absent from diagnostics: a value could itself
equal a generic error word. The deliberately unique synthetic marker makes
the redaction assertion meaningful. Native invalid-Unicode environment values
remain covered by the separate subprocess tests.

## Isolated Fuzz Workspace

The non-publishable [fuzz manifest](../fuzz/Cargo.toml) has its own workspace and
committed lockfile. It uses `libfuzzer-sys` 0.4.13 (MIT/Apache-2.0 with NCSA
runtime code) and reuses the library's base64/JSON dependencies for round-trip
oracles. None of the fuzz tooling enters the root package or normal stable
build. All targets use AddressSanitizer with debug assertions and overflow
checks; nightly is required only for this tooling. See the upstream
[setup guide](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html).

Install the pinned tools without changing the default Rust toolchain:

```sh
rustup toolchain install nightly-2026-09-24 --profile minimal --component rustfmt
cargo +stable install cargo-fuzz --locked --version 0.13.2
bash fuzz/run.sh 20 20260924
```

The five targets are `json`, `base64`, `list`, `url`, and `scalars`. Their input
files are plain UTF-8, with no control header. The first two bytes also select
limits, delimiters, and whitespace options. JSON and base64 separately exercise
the parser with a large enough cap so a selected small limit cannot hide it.
The harness rejects inputs larger than 4,096 bytes and byte sequences that
cannot be represented by the `Environment` trait's Rust `String` values.
Decoded invalid UTF-8 is still tested through valid base64 text.

## Seed Corpus

The 38 reviewed files under [fuzz/corpus](../fuzz/corpus) contain only synthetic
data. They include:

- Valid multibyte JSON, malformed objects, trailing data, a byte-limit fixture,
  and 256 nested arrays.
- Base64 padding lengths, missing padding, trailing bits, wrong alphabet,
  multibyte text, and invalid decoded UTF-8 (`//8=`).
- Consecutive/trailing delimiters, multi-character and Unicode separators,
  Unicode whitespace, numeric overflow, 65 list items, and the raw-limit trigger.
- Valid URLs with synthetic credentials, malformed IPv6 and ports, a NUL,
  scheme-less authorities, Unicode hosts, and disallowed explicit schemes.
- Scalar boundaries, overflow, whitespace, scientific notation, NaN, and
  multibyte text. Empty files document empty-input cases; libFuzzer also supplies
  empty input itself.

Git attributes preserve corpus bytes across checkouts, including intentional
whitespace and line endings.

The first corpus argument is a separate writable directory under
`fuzz/results/corpus`; committed seeds are the second, read-only input corpus.
Mutations and crash files are ignored by Git. Never seed a run with a real
environment dump, access token, credential, production URL, or customer payload.

## Budgets, CI, and Reproduction

[run.sh](../fuzz/run.sh) runs all five targets sequentially and fails immediately
if a build, invariant, sanitizer, timeout, memory limit, or lock check fails.
It validates its duration/seed arguments and preserves nonzero exits through
the log pipeline. Each target receives:

- The requested time budget (1–3,600 seconds), an explicit seed, and a 4,096-byte
  input cap.
- A five-second per-input timeout, 1,024-MiB RSS limit, and 16-MiB single-allocation
  limit.
- An explicit crash-artifact directory and final execution statistics.

The CI `Fuzz smoke` job gives each target 20 seconds, with a 15-minute outer
job timeout covering installation and compilation too. Nightly and cargo-fuzz
versions are pinned. The runner fetches against `fuzz/Cargo.lock` using
`--locked`, then builds and runs offline and compares the lock before fuzzing
and after each target. This compensates for cargo-fuzz lacking a `--locked`
option. Security CI audits the fuzz lock in addition to both root resolutions.
Runner tests simulate command failures and lock drift without starting a fuzzer.

For a longer local run, choose a fresh output directory to start from just the
committed seeds:

```sh
FUZZ_OUTPUT_DIR=/tmp/envbind-fuzz-long bash fuzz/run.sh 300 12345
```

This allows five minutes per target (about 25 minutes plus compilation).
Reusing an output directory also reuses its discovered corpus. The runner saves
compiler/host details, cargo-fuzz version, source commit, tracked source patch,
dependency lock, budgets, logs, and failure inputs. Preserve these locally for
reproduction. Record/stage new harness files before recording a local run so
the source patch includes them. A seed alone does not guarantee an identical
time-limited run across machines: use the same source, lock, tools, flags,
starting corpus, and a saved failing input to reproduce a defect directly.

```sh
cargo +nightly-2026-09-24 fuzz run json /path/to/crash-input
cargo +nightly-2026-09-24 fuzz tmin json /path/to/crash-input -- -max_total_time=60
```

Replace `json` with the failing target. Minimize only synthetic inputs. After
fixing a defect, add a small ordinary regression test, then add the reviewed
minimized seed if useful. If a reproduction contains real data, replace it with
synthetic equivalents before committing or sharing it. The original remains
local and must not be uploaded.

CI uploads only failure logs, metadata, lock, source patch, and crash files,
retained for seven days. It never uploads a developer environment or the
growing mutation corpus. GitHub's normal workflow-log retention applies to job
logs separately. CI starts from reviewed synthetic seeds in an isolated hosted
job; review seed changes with the same care as test code. Successful-run logs
remain in the job output; local files remain until manually removed. See the
upstream [fuzzing guide](https://rust-fuzz.github.io/book/cargo-fuzz/guide.html)
for additional commands.

## Recorded Bounded Run

On 2026-09-24, the initial full run for this change (based on `a831dc0`) completed on
`aarch64-apple-darwin` with nightly `1.100.0-nightly (6eeff9a52 2026-09-23)`,
cargo-fuzz 0.13.2, the committed fuzz lock, AddressSanitizer, seed `20260924`,
and fresh mutation directories. Each target had a 30-second budget; libFuzzer
reported 31 elapsed seconds per target, checking the time boundary between runs.

| Target | Executions | Peak RSS (MiB) | Result |
| --- | ---: | ---: | --- |
| JSON | 1,345,959 | 557 | Passed |
| Base64 | 2,127,176 | 678 | Passed |
| List | 1,236,637 | 424 | Passed |
| URL | 1,013,280 | 588 | Passed |
| Scalars/redaction | 760,003 | 617 | Passed |

All 6,483,055 executions completed without an invariant, sanitizer, timeout, or
memory-limit failure. Counts include inputs rejected by the UTF-8/domain guard;
they are not counts of unique valid parser inputs. No library defect was found
in this bounded run, so no defect-specific minimized regression was needed.
This is a smoke result, not proof that undiscovered parser defects are absent.

After extending the list checks to both enum matching policies, a fresh
30-second list run with the same tools, seed, and limits passed another
336,083 executions (31 seconds elapsed, 364-MiB peak RSS). The affected property
tests passed again on stable and Rust 1.85. Together these runs completed
6,819,138 executions without a failure.
