# Dependency Maintenance

The library supports Rust 1.85 and current stable Rust. Keep runtime dependency
requirements compatible with that minimum and review CI tooling separately.
`Cargo.lock` remains untracked; release validation preserves its generated,
audited lockfile in the release artifact.

## Review on 2026-10-25

Reviewed the four updates proposed in
[PR #43](https://github.com/OneTesseractInMultiverse/envbind-rs/pull/43),
[PR #44](https://github.com/OneTesseractInMultiverse/envbind-rs/pull/44),
[PR #45](https://github.com/OneTesseractInMultiverse/envbind-rs/pull/45), and
[PR #46](https://github.com/OneTesseractInMultiverse/envbind-rs/pull/46).
This review covers those updates and their required lockfile changes; the
older full dependency inventories below retain their original review dates.

- `yoke-derive = "=0.8.4"` replaces 0.8.2. Its published manifest declares
  Rust 1.82 support, and the source no longer calls the Rust 1.87-only
  `str::from_utf8` function that broke 0.8.3. The library keeps Rust 1.85 as
  its minimum. The obsolete Dependabot exclusion for 0.8.3 is removed; the
  new root requirement already excludes that version.
- The root requirement and committed fuzz lock now select the same derive
  version. Updating the requirement alone caused #43's locked fuzz fetch to
  fail before any fuzz target ran. The lock update also removes the unused
  syn 2 and synstructure 0.13 copies; the derive now shares the existing
  syn 3 and synstructure 0.14 dependencies.
- The fuzz lock selects `serde_json` 1.0.154, which declares Rust 1.71 support.
  The existing root and fuzz requirements, `serde_json = "1"`, already allow
  this release. Other fuzz dependency versions are retained.

The artifact action tags resolve to these full commits:

| Action | Release | Commit |
| --- | --- | --- |
| actions/upload-artifact | [v7.0.2](https://github.com/actions/upload-artifact/releases/tag/v7.0.2) | `cf430e030ddbb5b0abf93d22962f4752f3646cd9` |
| actions/download-artifact | [v8.0.2](https://github.com/actions/download-artifact/releases/tag/v8.0.2) | `9000827ccba6bdab643e8b6fd33ac0654aef8333` |

Both updates retain their action inputs and Node 24 runtime. The upstream
changes update the artifact client and improve rate-limit retries. Release
uploads still use `archive: true`, downloads select the validated artifact by
ID, and permissions, environment approval, and checksum checks are unchanged.
Checkout and registry authentication action pins are unchanged.

Validate these changes with the Rust 1.85 and stable gates, both fresh root
dependency audits, the committed fuzz audit, a locked fuzz fetch and bounded
sanitizer run, and the release workflow regressions. Record the exact commit
and CI results in the pull request; earlier readiness results do not certify
this changed candidate.

## yoke-derive 0.8.3 Review on 2026-09-27

[PR #30](https://github.com/OneTesseractInMultiverse/envbind-rs/pull/30) proposes
changing the exact constraint from 0.8.2 to 0.8.3. Its
[CI run](https://github.com/OneTesseractInMultiverse/envbind-rs/actions/runs/36335531450)
fails the Rust 1.85 job in `yoke-derive/src/lib.rs:202`: the macro calls the
inherent `str::from_utf8` function, which was
[stabilized in Rust 1.87](https://doc.rust-lang.org/std/primitive.str.html#method.from_utf8).
The release declares no `rust-version`, so Cargo's MSRV-aware resolver cannot
filter it out. An isolated crate depending only on `yoke-derive = "=0.8.3"`
reproduced error `E0599` with Rust 1.85.0 and compiled with Rust 1.98.1.

At that review, `yoke-derive = "=0.8.2"` preserved the Rust 1.85 contract.
Dependabot excluded exactly `=0.8.3` in both Cargo workspaces, leaving
subsequent versions eligible. The leading `=` was intentional: a bare
Cargo requirement would also match later compatible versions. See the
[Dependabot ignore options](https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference#ignore).
The [0.8.4 review](#review-on-2026-10-25) supersedes that constraint and removes
the exclusion. This was a compiler-compatibility decision, not a vulnerability
exception; advisory audits and maintainer triage continue.

The same PR also fails fuzzing because `fuzz/Cargo.lock` still selects 0.8.2.
For an accepted root dependency change, refresh and review the committed fuzz
lock in the same change, then run its locked fetch, advisory audit, and bounded
smoke gate. Updating that lock alone would not fix the compiler failure.

## Review on 2026-09-26

Rechecked every resolved registry package against the latest non-yanked stable
release on crates.io, plus the pinned tools, PyPI requirements, and GitHub
Actions releases. The runtime requirements still resolve base64 0.23.1,
regex 1.13.1, serde_json 1.0.151, and url 2.5.8. Proptest 1.11.0,
libfuzzer-sys 0.4.13, cargo-fuzz 0.13.2, cargo-audit 0.22.2, and PyYAML 6.0.3
remain current. All four action versions and full commit pins below were
reverified and remain current.

Refreshed the committed fuzz lockfile to cc 1.5.1, find-msvc-tools 0.1.14, and
smallvec 1.16.2. Both fresh root resolutions contain 64 registry packages; the
fuzz workspace contains 56. All three graphs passed cargo-audit 0.22.2 without
vulnerability findings or maintenance warnings. Root MSRV tests passed with
the refreshed compatible graph.

The remaining older selections are deliberate compatibility constraints:

- An isolated fresh build of yoke-derive 0.8.3 still fails on Rust 1.85 at
  `str::from_utf8`. Keep the 0.8.2 constraint until upstream fixes its compiler
  compatibility or an intentional MSRV change is made. Its syn 2 and
  synstructure 0.13 dependencies cannot be replaced with their newer major
  versions by a lockfile refresh.
- The latest proptest still requires rand/rand_core/rand_chacha 0.9 and
  rand_xorshift 0.4. That chain selects getrandom 0.3, r-efi 5, wasip2 1, and
  compatible wit-bindgen releases. Their newest incompatible release lines
  need upstream adoption; adding direct pins would not migrate these users.
- The MSRV resolver retains the older ICU/idna versions described below and
  wasip2 1.0.1+wasi-0.2.4 with wit-bindgen 0.46.0. The stable graph selects
  wasip2 1.0.4+wasi-0.2.12 with wit-bindgen 0.57.1. Wasip2 1.0.4 requires
  Rust 1.87; its latest 2.x line is outside getrandom's requirement.
- The fuzz workspace's getrandom 0.4 selects r-efi 6 rather than the latest
  incompatible 7.x line. This target-specific dependency is outside the normal
  library runtime graph.

Keep the tested `nightly-2026-09-24` fuzz compiler pin for reproducible runs;
nightly snapshots are reviewed tooling choices, not stable dependency releases.
The library's Rust 1.85 minimum is unchanged.

## Earlier Review on 2026-09-22

Versions were checked against crates.io, PyPI, and upstream GitHub release tags.
Existing compatible requirements for regex, JSON, and URL parsing did not need
changes to resolve their latest stable releases.

| Dependency | Selected version | Review |
| --- | --- | --- |
| [base64](https://crates.io/crates/base64/0.23.1) | 0.23.1 | Updated from 0.22.1; requires Rust 1.71. |
| [regex](https://crates.io/crates/regex/1.13.1) | 1.13.1 | Already allowed by `regex = "1"`. |
| [serde_json](https://crates.io/crates/serde_json/1.0.151) | 1.0.151 | Already allowed by `serde_json = "1"`. |
| [url](https://crates.io/crates/url/2.5.8) | 2.5.8 | Current stable release. |
| [yoke-derive](https://crates.io/crates/yoke-derive/0.8.2) | 0.8.2 | Compatibility constraint; latest 0.8.3 fails on Rust 1.85. |
| [cargo-audit](https://crates.io/crates/cargo-audit/0.22.2) | 0.22.2 | Updated both workflows; its Rust 1.88 requirement applies only to tooling installed on stable. |
| [PyYAML](https://pypi.org/project/PyYAML/6.0.3/) | 6.0.3 | Current stable release; used only by workflow tests. |

Base64 0.23 adds SIMD engines behind a default feature. Envbind retains its
existing `general_purpose::STANDARD` decoder and selects only the `std`
feature, leaving the unused SIMD feature disabled. Padding, alphabet, UTF-8,
size limits, and redacted error behavior remain unchanged. Regression tests
pass with both 0.22.1 and 0.23.1. See the upstream
[base64 release notes](https://docs.rs/crate/base64/0.23.1/source/RELEASE-NOTES.md).

All action tags were resolved to their full commit SHAs:

| Action | Release | Commit |
| --- | --- | --- |
| actions/checkout | [v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1) | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| actions/upload-artifact | [v7.0.1](https://github.com/actions/upload-artifact/releases/tag/v7.0.1) | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` |
| actions/download-artifact | [v8.0.1](https://github.com/actions/download-artifact/releases/tag/v8.0.1) | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` (unchanged) |
| rust-lang/crates-io-auth-action | [v1.0.5](https://github.com/rust-lang/crates-io-auth-action/releases/tag/v1.0.5) | `c6f97d42243bad5fab37ca0427f495c86d5b1a18` |

These actions use Node 24 on GitHub-hosted `ubuntu-latest` runners. Checkout
retains `persist-credentials: false`; the workflows do not use the
`pull_request_target` or `workflow_run` triggers affected by checkout v7's new
restrictions. Artifact upload explicitly uses `archive: true` for the release
bundle's four files. Download v8 continues to extract a single artifact ID
directly into `target/release`. Permissions and the protected publishing gate
remain unchanged.

## Dependency Resolution and Remaining Constraints

CI checks two fresh dependency resolutions:

- Rust 1.85 uses `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback` to select
  releases compatible with the declared minimum.
- Stable uses `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=allow` to exercise the
  newest releases permitted by the manifest, including dependencies requiring
  a newer compiler. The stable compiler used for the 2026-10-25 review is
  Rust 1.99.1.

The security job audits both resolutions. Release validation retains Cargo's
MSRV-aware default resolution and audits that exact lockfile before packaging.
The earlier graphs reviewed on 2026-09-22 contained 46 registry packages and
passed cargo-audit 0.22.2 without vulnerability findings or maintenance warnings.
The [2026-09-26 review](#review-on-2026-09-26) supersedes that package inventory
and includes the subsequently added property and fuzz tooling.

The stable dependency graph is now exercised by native Linux, macOS, and
Windows jobs. The required `Rust stable` aggregate check succeeds only after
the entire Rust matrix succeeds; `Rust 1.85.0` remains the dedicated Linux MSRV
check. See [platform CI](testing-guide.md#platform-ci-and-required-checks).

Remaining constraints are explicit:

- `yoke-derive = "=0.8.4"` selects the reviewed release for both root
  resolutions and the fuzz workspace. Future updates must keep the committed
  fuzz lock aligned and pass the MSRV gate. The 0.8.2 workaround is retired;
  the incompatible 0.8.3 release remains excluded by the root requirement.
- The derive uses syn 3 and synstructure 0.14. Older review records describe
  syn 2 and synstructure 0.13 copies that this update removes from the fuzz lock.
- The MSRV graph uses ICU collections, locale core, normalizer, normalizer data,
  and provider 2.1.1, properties and properties data 2.1.2, and idna_adapter
  1.2.1. The 2026-10-25 stable resolution exercises ICU 2.4.0 and
  idna_adapter 1.2.2; the fuzz lock retains ICU 2.3.0 (provider 2.3.1).
  The newer ICU packages require Rust 1.88, and idna_adapter 1.2.2 requires 1.86.
  These older MSRV selections are resolver choices, not additional pins.

Every other resolved registry package was checked against its latest stable
release during this review. Regenerate and inspect the graph when updating;
the review date does not freeze future dependency resolution.

## Ongoing Advisory Checks

The `CI` workflow runs the same `Security audit` job for pull requests, pushes
to `main`, manual runs, and a daily schedule at 07:23 UTC. Scheduled runs use
the latest commit on the default branch and become active when the workflow
change reaches that branch. Schedules and explicit manual audit-only runs skip
the Rust matrix, workflow tests, aggregate gate, and fuzzing. Normal pull
request, push, and manual runs retain every check. Separate concurrency groups
prevent an audit-only run from cancelling full CI.

The job has a 15-minute timeout, read-only repository permissions, checkout
credentials disabled, and no publishing credentials. It installs the pinned
cargo-audit version on stable Rust, generates two fresh root lockfiles using
`fallback` and `allow`, and audits those plus the unchanged `fuzz/Cargo.lock`.
Every audit fetches the current RustSec database; no cached-only, stale,
target-filtered, or advisory-ignore mode is used. Vulnerabilities and all
warnings fail the job (`--deny warnings`). A failed audit still allows the
other graphs to be checked, while preserving the failing job status. Registry,
database, or dependency-resolution errors never count as a clean audit.

Each run retains the three lockfiles and JSON reports in
`security-audit-<run-id>-<attempt>` for 14 days, including evidence available
when a check fails. The reports record database freshness and findings. Copy
the artifact into the advisory investigation before it expires; rerunning a
fresh resolution may select different versions.

Run only this job manually with the local GitHub CLI:

```sh
gh workflow run ci.yml --ref main -f security_only=true
gh run list --workflow ci.yml --event workflow_dispatch --limit 5
gh run view RUN_ID --log
gh run download RUN_ID --name security-audit-RUN_ID-1 --dir /tmp/envbind-audit
```

Replace `RUN_ID` and the attempt suffix with the selected run's values. To test
a workflow change before merging, use its branch in place of `main`. Leave
`security_only` unchecked or omit the input to run full CI. The scheduled job
and this manual mode are the same job definition, not separate audit scripts.

### Scope and Retained Release Resolutions

Fresh maintained-branch resolutions detect advisories in dependencies selected
today. They do not cover every older version permitted by the manifest, every
consumer lockfile, or previously published release resolutions. Dependabot's
dependency graph covers supported checked-in manifests and lockfiles; it does
not replace the two explicit fresh root audits or scans of downstream apps.
The scheduled RustSec job covers Rust dependencies. Existing weekly Dependabot
checks also cover GitHub Actions and the Python workflow-test requirements.

Release validation saves the exact audited `Cargo.lock`, JSON audit result,
package archive, and source/checksum metadata together. The publishing handoff
verifies and reuses this resolution; see [publishing](publishing.md). A retained
audit report is evidence from its run date, not a guarantee against later
advisories. When triaging a new advisory, retrieve the supported release's
validation bundle, verify its recorded checksums, and audit its lockfile again
with the current database:

```sh
cargo audit --deny warnings --file /path/to/verified-release/Cargo.lock
```

Archive release evidence for the supported version's lifetime before the
workflow artifact expires. If an old release lockfile is unavailable, record
the coverage gap and investigate the published manifest and downstream locks;
do not substitute a newly generated lockfile and claim the old release passed.

### Control and Notification Verification

On 2026-09-27, the repository API confirmed vulnerability alerts enabled
(HTTP 204), automated security fixes enabled and not paused, and a populated
dependency-graph SBOM. Secret scanning and push protection were still enabled.
No Dependabot alerts were open at that check. Recheck these controls with:

```sh
gh api -i repos/OneTesseractInMultiverse/envbind-rs/vulnerability-alerts
gh api repos/OneTesseractInMultiverse/envbind-rs/automated-security-fixes
gh api repos/OneTesseractInMultiverse/envbind-rs/dependency-graph/sbom
gh api repos/OneTesseractInMultiverse/envbind-rs --jq .security_and_analysis
```

Pedro Guzmán (`@OneTesseractInMultiverse`) owns triage. His existing account
settings were verified to enable failed-workflow email and new Dependabot
alert notifications. GitHub routes scheduled-workflow notifications to the
schedule's author, the last cron editor, or the user who re-enabled it; this
ownership must be checked after changes. Account notification preferences are
separate from repository configuration. Preserve the configured destination
and recheck them in [notification settings](https://github.com/settings/notifications).
The verification did not send an email-delivery probe.

Follow the [security policy](../SECURITY.md#dependency-monitoring-and-response)
for triage, schedule health checks, and the time-bounded exception policy.
GitHub documents [workflow notifications](https://docs.github.com/en/actions/concepts/workflows-and-actions/notifications-for-workflow-runs)
and [schedule behavior](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#schedule).

## Future Updates

Property tests use proptest 1.11 as a development-only dependency, with only
`std` enabled to avoid subprocess/fork tooling. It supports Rust 1.85 and uses
the MIT/Apache-2.0 license. The separate non-publishable `fuzz` workspace pins
libfuzzer-sys 0.4.13 and retains its own lockfile; its LLVM runtime adds the NCSA
license alongside MIT/Apache-2.0. Both choices and maintenance costs are
documented in the [fuzzing guide](fuzzing.md).

Dependabot also checks the `/fuzz` Cargo workspace weekly. Security CI audits
that committed lockfile. Review cargo-fuzz 0.13.2 and the pinned
`nightly-2026-09-24` toolchain manually with other CI tooling updates; these
pins are not Cargo dependencies managed by Dependabot. Update the workflow,
runner default, and guide together, rerun the bounded targets, and record the
new result. These tools do not change the library's minimum Rust version.

Dependabot checks Cargo, GitHub Actions, and the Python workflow test
requirements weekly. Keep action references pinned to full release commit SHAs
with matching version comments, and review major-version migration notes.

The `cargo install cargo-audit --locked --version ...` commands are not covered
by Dependabot's Cargo manifest updates. At each dependency refresh and before
a release, check the current stable cargo-audit release and update both
`ci.yml` and `publish.yml`. Install and run that exact version, review advisories
and warnings, and check that its required Rust version is supported by the
stable toolchain used for CI tooling.

For runtime updates, inspect `cargo tree`, test both resolution modes, and run
the local verification, package, and publish dry-run gates. Confirm the release
workflow tests and a validation-only workflow run before relying on upgraded
actions for publication. Never publish a package merely to test an upgrade.
