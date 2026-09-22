# Release Controls

GitHub environment rules, tag rulesets, and crates.io Trusted Publishers live
outside this repository. The expected values are recorded in
[release-policy.json](../.github/release-policy.json). Changing that file does
not change remote settings. Review policy changes together with the actual
settings and rerun the checks below.

## Approval and Ref Policy

| Control | Required setting |
| --- | --- |
| GitHub environment | `crates-io` |
| Required reviewer | `OneTesseractInMultiverse` (user ID `22035370`) |
| Prevent self-review | Disabled: the sole maintainer must be able to approve their own release |
| Administrator bypass | Disabled: an administrator must also complete the approval gate |
| Deployment branch policy | Selected branches and tags |
| Allowed branch | `main` |
| Allowed tags | `v*` |
| Release-tag ruleset | `Protect release tags`, active, target `tag`, include `refs/tags/v*`, no exclusions |
| Tag rules | Restrict updates and deletions; allow creation |
| Tag bypass actors | None |

The environment evaluates the workflow ref. For `workflow_dispatch`, this is
the ref selected when starting the workflow, not `inputs.tag`. A manual run
from `main` can validate a version tag; a run from an issue branch cannot enter
the environment. A `release: published` run uses the release tag as its ref.
The `v*` glob only selects candidate refs. `publish.yml` separately rejects
malformed SemVer tags and version mismatches. Validation-only manual runs skip
the publishing job and therefore do not exercise the approval gate.

This policy provides an explicit release confirmation by the sole maintainer.
It does not provide separation of duties. If another maintainer joins, review
the reviewer list and consider preventing self-review. Main's existing review,
code-owner, last-push, conversation-resolution, and required-check protections
remain separate and must not be relaxed to release a crate.

## Preflight

From a reviewed checkout with Python 3 and an authenticated GitHub CLI that
can read repository settings, run:

```sh
python3 -B .github/scripts/check_release_controls.py
```

The command only reads GitHub APIs. It checks the environment reviewer,
self-review and bypass policies, exact allowed branch/tag patterns, active tag
protection without exceptions or bypass actors, and minimum main protections.
Missing settings, API access failures, and policy drift return a nonzero exit
status. It checks that required status checks are tied to GitHub Actions, not
just that similarly named checks exist. It does not change settings, dispatch
workflows, request registry credentials, or publish packages. Run it before
every release and after repository ownership, reviewer, workflow, or protection
changes. It is a maintainer check, not an automatic CI or publish prerequisite.

The GitHub check cannot inspect the authenticated crates.io settings. Sign in
as a crate owner, open [envbind settings](https://crates.io/crates/envbind/settings),
and inspect every Trusted Publisher row. The intended GitHub publisher is:

| Field | Exact value |
| --- | --- |
| Repository owner | `OneTesseractInMultiverse` |
| Repository name | `envbind-rs` |
| Workflow filename | `publish.yml` |
| Environment name | `crates-io` |

An empty environment field is not equivalent. Investigate unexpected additional
publishers, including any entry that omits the environment restriction. Verify
these values visually without requesting a publishing token or copying any
credentials. GitHub's preflight success does not prove registry configuration
or a successful OIDC token exchange.

For raw evidence, the relevant read-only GitHub endpoints are:

```sh
gh api repos/OneTesseractInMultiverse/envbind-rs/environments/crates-io
gh api --paginate repos/OneTesseractInMultiverse/envbind-rs/environments/crates-io/deployment-branch-policies
gh api --paginate repos/OneTesseractInMultiverse/envbind-rs/rulesets
# Substitute each ruleset ID returned above to inspect its full rules:
gh api repos/OneTesseractInMultiverse/envbind-rs/rulesets/RULESET_ID
```

## Recovery

Never bypass environment approval or weaken main protection to recover a
release. Failed validation or expired artifacts require a fresh validation run.
Prefer a new version tag over correcting an existing one. GitHub releases and
registry publications are distinct; deleting a GitHub release does not undo
a crates.io publication.

There is no standing release-tag bypass, including for repository
administrators. If an unpublished tag must be removed or corrected, a
repository administrator must deliberately edit the release-tag ruleset.
Record the reason and exact tag in an issue first, cancel release runs for
that tag, and confirm it has not been published. Temporarily exclude only that
exact ref, make the correction, immediately remove the exception, and rerun
the preflight. Record the before/after commit IDs and restored rules in the
issue. Do not disable the entire ruleset or add a wildcard exclusion.

Administrators can edit repository settings, so disabling bypass does not
make the policy immutable against an administrator. Audit configuration
changes and use the preflight to detect drift.

## Token-Free Gate Verification

Use [.github/fixtures/release-gate.yml](../.github/fixtures/release-gate.yml) to
exercise the real `crates-io` environment. It has empty token permissions,
no checkout, no third-party actions, no secret references, and no publishing
commands. Its only step rejects unexpected credential access and prints the
event and workflow ref. Do not test approval by dispatching the real publishing
workflow with `publish=true`.

1. In an isolated clone, create a temporary commit based on `main`, replacing
   `.github/workflows/publish.yml` with the fixture. Review that entire workflow
   and run `actionlint .github/workflows/publish.yml`. Never merge this fixture
   commit into `main` or use a real release/version tag for it.
2. Create two unique temporary tags on that commit: an allowed ref such as
   `v0.0.0-release-gate-check.ISSUE`, and a denied ref such as
   `release-gate-check-ISSUE`. Ensure neither tag already exists. Push both.
   During initial setup, finish fixture cleanup before installing tag
   immutability. On later retests, use the exact-ref recovery procedure above
   to remove only the temporary allowed tag after testing.
3. Dispatch the harmless workflow on both refs:

   ```sh
   gh workflow run publish.yml --ref v0.0.0-release-gate-check.ISSUE
   gh workflow run publish.yml --ref release-gate-check-ISSUE
   gh run list --workflow publish.yml --json databaseId,event,headBranch,status,conclusion,url
   ```

4. Inspect the allowed run's pending deployments before approving:

   ```sh
   gh api repos/OneTesseractInMultiverse/envbind-rs/actions/runs/RUN_ID/pending_deployments
   ```

   It must wait for `crates-io` and the configured reviewer with no fixture
   steps executed. Approve the harmless deployment as that reviewer, then
   verify the success log identifies `workflow_dispatch` and the allowed ref.
   The denied run must fail with the environment's ref-policy error and no
   executed steps. Preserve run links and rejection annotations.
5. Create a temporary, explicitly labeled GitHub prerelease on the allowed
   fixture tag with `--latest=false` and `--verify-tag`. This tests the actual
   `release: published` event. Verify that it waits for the same reviewer,
   approve the harmless fixture, and confirm its log identifies `release`
   and the expected tag. No crate release or token exchange is involved.
6. Delete only the temporary GitHub prerelease and the two fixture tags.
   Restore any exact-ref exception immediately. Keep workflow-run evidence,
   rerun the preflight, and compare main's full ruleset before and after.

The local regression suite tests preflight failures without network access:

```sh
python3 -B -m unittest discover -s .github/tests -p test_release_controls.py -v
```

## GitHub Verification Record

The GitHub controls were configured and inspected on **2026-09-22** for
[#14](https://github.com/OneTesseractInMultiverse/envbind-rs/issues/14).
The following runs used the token-free fixture at commit
`b9c841d55523bfc6b3a1e425ab404f6b213e9c6a`:

| Scenario | Evidence | Result |
| --- | --- | --- |
| Manual dispatch, allowed version tag | [Run 35796997977](https://github.com/OneTesseractInMultiverse/envbind-rs/actions/runs/35796997977) | Waited for the configured reviewer; succeeded after approval |
| Manual dispatch, disallowed tag | [Run 35797000236](https://github.com/OneTesseractInMultiverse/envbind-rs/actions/runs/35797000236) | Rejected by environment ref policy before any steps executed |
| Published release, allowed version tag | [Run 35797055527](https://github.com/OneTesseractInMultiverse/envbind-rs/actions/runs/35797055527) | Waited for the configured reviewer; succeeded after approval |

Both successful runs reported their event and expected ref with no credential
access. The temporary GitHub prerelease and both fixture tags were deleted
before the release-tag ruleset was enabled. The preflight failed while that
ruleset was missing and passed after installation. A full API comparison
confirmed that `Protect main` was unchanged. These tests verify the GitHub
approval/ref gates; they do not exercise crates.io token exchange or publication.

## References

- [GitHub deployment environments](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments)
- [GitHub environment API](https://docs.github.com/en/rest/deployments/environments)
- [GitHub deployment ref API](https://docs.github.com/en/rest/deployments/branch-policies)
- [GitHub ruleset API](https://docs.github.com/en/rest/repos/rules)
- [crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing)
