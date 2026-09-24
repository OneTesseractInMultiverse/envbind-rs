# Binding Contract Test Matrix

This matrix maps the public field contracts to deterministic regression tests.
The fixtures use `MapEnvironment` or a synthetic failing adapter, require no
network or host configuration, and exercise the public binding API. Each test
uses one assertion. Shared macros keep common expectations aligned while each
invocation supplies the field's input, typed fallback, and expected results.

## Field Coverage

Every module below is in [field_contracts.rs](../tests/field_contracts.rs).
Each runs the common contracts in the next table. Missing and empty fallback
validation is also covered for all ten types by the corresponding modules in
[default_validation.rs](../tests/default_validation.rs).

| Public field | Contract module | Additional parser and boundary coverage |
| --- | --- | --- |
| `StringVar` | `string` | `string_ascii`, `string_utf8`, `string_zero` in [defensive_limits.rs](../tests/defensive_limits.rs); [fields_string.rs](../tests/fields_string.rs). |
| `OptionalStringVar` | `optional_string` | `optional_string_ascii`, `optional_string_utf8`, `optional_string_zero` in defensive limits; optional-string cases in fields_string. |
| `BoolVar` | `boolean` | `boolean_raw` in defensive limits; all accepted tokens in [parser_contracts.rs](../tests/parser_contracts.rs); [fields_bool.rs](../tests/fields_bool.rs). |
| `IntVar` | `integer` | `integer_raw` in defensive limits; both `i64` bounds and overflow in parser contracts. |
| `FloatVar` | `float` | `float_raw` in defensive limits; finite scientific notation and finite extremes in parser contracts. |
| `U16Var` | `u16_value` | `u16_raw` in defensive limits; zero, maximum, negative, and overflow input in parser contracts; [fields_u16.rs](../tests/fields_u16.rs). |
| `JsonVar` | `json_value` | `json_ascii`, `json_utf8`, malformed/trailing data, and nesting in defensive limits. |
| `B64DecodedStringVar` | `base64` | Encoded/decoded limits in defensive limits; padding, trailing bits, alphabet, and invalid decoded UTF-8 in [fields_b64.rs](../tests/fields_b64.rs). |
| `EnumVar` | `enumeration` | `enum_raw` in defensive limits; empty labels and wrong-case rejection in parser contracts; aliases and names/values in [python_parity.rs](../tests/python_parity.rs). |
| `ListVar` | `list` (integer items) | Raw/item limits in defensive limits; each built-in item parser, custom parsers, delimiters, and whitespace in parser contracts and python_parity. |

`parameter_source_composes_all_public_field_types` in
[parameter_source.rs](../tests/parameter_source.rs) composes all ten field types
into one settings object and checks every typed result.
`parameter_source_identifies_a_late_field_failure` verifies the error code and
variable name when the final field fails after earlier fields succeed.

## Common Contracts

The names below are test-name suffixes within each field module. Failures check
both the stable error code and the originating variable name.

| Contract | Tests |
| --- | --- |
| Valid, missing, and explicitly empty input | `present_input_parses`, `missing_input_has_the_documented_result`, `empty_input_has_the_documented_result`. |
| Whitespace remains present input even with a fallback | `whitespace_is_input_not_absence`. |
| Optional success, missing/empty absence, and missing/empty fallbacks | `optional_wraps_successful_values`, `optional_handles_missing_and_empty_without_defaults`, `optional_keeps_missing_and_empty_defaults`. |
| Allowed empty input reaches the parser instead of selecting a fallback | `allowed_empty_bypasses_default_and_reaches_the_parser`. |
| Allowing empty input does not change missing-input fallback behavior | `allowed_empty_does_not_replace_a_missing_default`. |
| Adapter failure is neither a fallback nor optional absence | `adapter_failure_ignores_defaults_and_propagates_through_optional`. |
| Present malformed input is neither a fallback nor optional absence | `malformed_input_does_not_fall_back_or_become_optional_absence` for every parsing field. |
| Successful validation runs once on a parsed value | `successful_validator_runs_on_the_parsed_value`. |
| Validation failure propagates through optional wrappers with the selected sensitivity | `sensitive_validation_failure_propagates_without_diagnostics`, `nonsensitive_validation_failure_keeps_explicit_diagnostics`. |
| Validators run in registration order and stop at the first failure | `validators_stop_in_registration_order_at_first_failure`. |
| Typed defaults skip validation unless enabled; enabled validation runs once per binding | The `defaults_skip_validators_unless_enabled`, `missing_input_validates_default`, `empty_input_validates_default`, and `each_validator_runs_once_per_binding` cases in default_validation. |

Sensitive-error assertions use synthetic markers from real adapter reads,
parsed-value validators, malformed input, and custom list item parsers.
They check `Display`, `Debug`, alternate `Debug`, and error-source formatting
where applicable. Adapter diagnostics remain redacted even if the field is
marked nonsensitive; explicitly nonsensitive validation details are retained.

## Defensive Boundaries

| Contract | Test evidence |
| --- | --- |
| String/optional-string bytes | The ASCII and UTF-8 modules accept exactly the configured byte count, reject one additional byte through optional wrappers, and prove rejected input skips validators. Zero-byte limits accept allowed empty strings only. |
| JSON bytes and parsing order | The JSON modules accept exact ASCII/UTF-8 byte limits and reject one extra byte. `oversized_malformed_json_is_rejected_by_size_before_parsing` distinguishes the size error from a parse error. |
| Fixed scalar raw limit | `boolean_raw`, `integer_raw`, `float_raw`, and `u16_raw` accept valid one-MiB inputs, reject one additional byte, and count validator calls. `enum_raw` covers the same limit for labels. |
| Fixed list raw limit | `list_raw_limit_accepts_exact_bytes_and_rejects_one_more_before_parser` proves the parser receives exactly one allowed item and no oversized input. |
| List item counts | `list_item_limit_never_calls_parser_for_an_excess_item` covers zero, one, and a configured maximum of two. `default_list_item_limit_accepts_1024_and_stops_before_item_1025` covers the default boundary. |
| List delimiters and whitespace | Empty delimiters fail before callbacks; multi-character and Unicode delimiters retain consecutive/trailing empty items. Allowed empty strings produce one empty item. Tests distinguish split trimming from item-parser trimming. |
| Base64 encoded/decoded bytes | Existing padding-length tests accept exact decoded limits of one, two, and three bytes. New tests distinguish encoded-size rejection from decoded-size rejection within an allowed encoded block, count skipped validators, cover multibyte UTF-8 and zero limits, and check saturating bound arithmetic at `usize::MAX`. |
| Base64 malformed/UTF-8 input | Existing tests reject malformed padding, noncanonical trailing bits, the wrong alphabet, and decoded invalid UTF-8. Oversized invalid decoded UTF-8 produces the size error before conversion to text. |
| JSON malformed/deep input | Tests reject incomplete JSON, trailing data, and trailing commas. A small fixture accepts 32 nested arrays and rejects 256, exercising the parser's recursion guard without exhausting memory or the stack. |
| Item parse failures | Typed-list failures use `validation_failed`; custom-parser failure stops subsequent items and skips list validators. Synthetic item details follow sensitivity through optional wrappers. |

These are bounded regression cases, not performance benchmarks. The largest
individual raw-input fixture is one MiB plus one byte. List item-limit enforcement
is incremental: earlier permitted items may already have been parsed when a
later item exceeds the cap. The excess item itself never reaches the parser.

## Applicability and Separate Policies

- Strings arrive from `Environment` as Rust `String` values, so there is no
  malformed-text parser case for `StringVar` or `OptionalStringVar`. Invalid
  native Unicode is an adapter error, covered by the separate
  [process integration tests](testing-guide.md#process-environment-tests).
- `OptionalStringVar` succeeds with `None` for missing/empty input without a
  fallback. Wrapping that binding in `.optional()` returns `Some(None)`.
  The wrapper converts only `missing_variable` and `empty_variable` errors to
  absence; it wraps every successful result, including an existing `None`.
- `allow_empty()` sends empty input to the parser. It does not guarantee a
  successful parse: numbers, booleans, JSON, and numeric list items reject it.
  An enum can accept it when an empty label is configured. An allowed empty
  string list contains one empty string and therefore exceeds a zero-item cap.
- Typed defaults are not reparsed, decoded, serialized, or checked against input
  limits. They skip validators unless `.validate_default()` is enabled, following
  [#12](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/12).
  The focused default_validation tests also cover empty typed fallbacks,
  builder order, and fallback validation through optional wrappers.
- `keep_whitespace()` disables the list splitting stage's trim. Boolean and
  enum item parsers independently trim their input; numeric parsers do not.
- The fixed scalar/list raw-byte cap has no public override. JSON nesting is
  guarded by its parser; there is no public configurable depth limit, and the
  nesting test does not establish an exact public depth threshold.
- Integer overflow has explicit rejection tests. A finite-only float policy,
  including nonfinite tokens and overflow to infinity, remains
  [#19](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/19).
  These tests do not establish that unresolved policy. The separate
  [property tests and fuzz targets](fuzzing.md) extend this deterministic matrix.

## Coverage Inspection

Line coverage was compared on macOS/aarch64 using Rust 1.98.1 and
`cargo-llvm-cov` 0.9.1, with all targets and the same local dependency lock.
The baseline is commit `d03d965` (after #16); the comparison run includes #17's
test additions. Runtime source files are identical in both runs.

| Source file | Before: covered/total lines | After: covered/total lines |
| --- | --- | --- |
| `fields/b64.rs` | 74/80 | 80/80 |
| `fields/bool.rs` | 53/57 | 57/57 |
| `fields/enumeration.rs` | 99/106 | 105/106 |
| `fields/float.rs` | 50/56 | 56/56 |
| `fields/int.rs` | 52/56 | 56/56 |
| `fields/json.rs` | 60/67 | 67/67 |
| `fields/list.rs` | 187/211 | 210/211 |
| `fields/raw.rs` | 37/38 | 38/38 |
| `fields/string.rs` | 174/176 | 175/176 |
| `fields/u16.rs` | 72/72 | 72/72 |
| `binder.rs` | 20/20 | 20/20 |

The previously unexecuted `allow_empty()` setters, float-list constructor and
failure mapping, list whitespace option, and item-parser error mapping now
execute. These are LLVM line counts, not proof of complete branch coverage or
every generic instantiation. The contract assertions and callback counters
establish behavior; the numbers only help locate omissions.

The custom subprocess harnesses clear inherited environment variables,
including the profiler destination. Their child profiles are not collected by
this command, so it understates process-adapter coverage. Their separate tests
remain part of the normal verification gate. General validator-helper coverage
is outside this field-contract comparison.

To inspect coverage locally, install `cargo-llvm-cov` and the toolchain's
`llvm-tools-preview` component, then run:

```sh
cargo llvm-cov --all-targets --locked --lcov \
  --output-path /tmp/envbind-coverage.lcov \
  --ignore-filename-regex '/(tests|examples)/'
```

Keep profiler output out of commits. Coverage is a review aid, not an additional
required developer dependency or a percentage gate. Run focused contracts with:

```sh
cargo test --test field_contracts --test default_validation
cargo test --test defensive_limits --test parser_contracts --test fields_b64
cargo test --test parameter_source
```
