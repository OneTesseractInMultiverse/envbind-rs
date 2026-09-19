"""Exercise release workflow scripts without network calls or publishing."""

import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

import yaml


WORKFLOW = Path(__file__).resolve().parents[1] / "workflows" / "publish.yml"


class PublishWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.workflow = yaml.safe_load(WORKFLOW.read_text())
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.output = self.root / ".git" / "workflow-output"
        self.environment = {
            "PATH": str(Path(sys.executable).parent) + os.pathsep + os.environ["PATH"],
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_AUTHOR_NAME": "Release Test",
            "GIT_AUTHOR_EMAIL": "test@example.invalid",
            "GIT_COMMITTER_NAME": "Release Test",
            "GIT_COMMITTER_EMAIL": "test@example.invalid",
        }
        self.git("init", "--quiet", "--initial-branch=main", "--template=")
        self.manifest("0.1.1")
        self.git("add", "Cargo.toml")
        self.git("commit", "--quiet", "-m", "Fixture")
        self.git("tag", "v0.1.1")

    def git(self, *args):
        return subprocess.run(
            ["git", *args],
            cwd=self.root,
            env=self.environment,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()

    def manifest(self, version):
        (self.root / "Cargo.toml").write_text(
            f'[package]\nname = "fixture"\nversion = "{version}"\n'
        )

    def step(self, job, name):
        return next(
            step for step in self.workflow["jobs"][job]["steps"]
            if step.get("name") == name
        )

    def run_step(self, step, tag, event="workflow_dispatch", release_tag=None, context_values=None):
        # Substitute only the contexts used by these steps, as the runner does
        # before executing a script. This also reproduces unsafe interpolation.
        context = {
            "github.event_name": event,
            "inputs.tag": tag if event == "workflow_dispatch" else "",
            "github.event.release.tag_name": release_tag if release_tag is not None else tag,
            "needs.resolve-tag.outputs.tag": tag,
            "needs.validate.outputs.commit": getattr(self, "sha", self.git("rev-parse", "HEAD")),
            "needs.validate.outputs.metadata_sha256": getattr(self, "metadata_digest", ""),
            "needs.validate.outputs.toolchain": getattr(self, "toolchain", "1.85.0"),
            "steps.source.outputs.commit": self.git("rev-parse", "HEAD"),
            "github.run_id": "123",
            "github.run_attempt": "1",
        }
        context.update(context_values or {})

        def render(value):
            return re.sub(
                r"[$][{][{]\s*([^{}]+?)\s*[}][}]",
                lambda match: context[match.group(1)],
                value,
            )

        self.output.write_text("")
        environment = dict(self.environment)
        environment.update({
            "GITHUB_OUTPUT": str(self.output),
            "GITHUB_REF_NAME": tag,
        })
        job = next(job for job in self.workflow["jobs"].values() if step in job["steps"])
        for values in [job.get("env", {}), step.get("env", {})]:
            environment.update({key: render(value) for key, value in values.items()})
        command = (
            [sys.executable, "-c"] if step.get("shell") == "python"
            else ["bash", "--noprofile", "--norc", "-e", "-o", "pipefail", "-c"]
        )
        return subprocess.run(
            command + [render(step["run"])],
            cwd=self.root,
            env=environment,
            text=True,
            capture_output=True,
            timeout=60,
        )

    def test_no_context_expressions_are_embedded_in_release_scripts(self):
        scripts = [
            step["run"]
            for job in self.workflow["jobs"].values()
            for step in job["steps"]
            if "run" in step
        ]
        self.assertFalse(any("${{" in script for script in scripts))

    def test_version_guards_do_not_execute_tag_contents(self):
        inputs = [
            "v0.1.1$(touch${IFS}MARKER)",
            "v0.1.1" + chr(96) + "touch${IFS}MARKER" + chr(96),
            'v0.1.1";touch MARKER;#',
            "v0.1.1\n$(touch MARKER)",
        ]
        for job in ["validate", "publish"]:
            for event in ["workflow_dispatch", "release"]:
                for tag in inputs:
                    with self.subTest(job=job, event=event, tag=tag):
                        marker = self.root / "MARKER"
                        marker.unlink(missing_ok=True)
                        result = self.run_step(
                            self.step(job, "Check tag matches Cargo.toml version"),
                            tag,
                            event,
                        )
                        self.assertEqual(
                            (result.returncode != 0, marker.exists()), (True, False)
                        )

    def test_tag_preflight_accepts_supported_release_versions(self):
        inputs = [
            ("workflow_dispatch", "v0.1.1", "v0.1.1"),
            ("workflow_dispatch", "refs/tags/v0.1.1", "v0.1.1"),
            ("release", "v0.1.1", "v0.1.1"),
            ("release", "v1.2.3-rc.1+build.001", "v1.2.3-rc.1+build.001"),
            ("workflow_dispatch", "v10.20.30-alpha-1.0+sha-1", "v10.20.30-alpha-1.0+sha-1"),
        ]
        step = self.step("resolve-tag", "Validate release tag input")
        for event, tag, expected in inputs:
            with self.subTest(event=event, tag=tag):
                result = self.run_step(step, tag, event)
                self.assertEqual(
                    (result.returncode, self.output.read_text()),
                    (0, f"tag={expected}\nref=refs/tags/{expected}\n"),
                    result.stderr,
                )

    def test_tag_preflight_rejects_invalid_input_without_execution_or_outputs(self):
        inputs = [
            "", "main", "0.1.1", "refs/heads/v0.1.1", "refs/tags/refs/tags/v0.1.1",
            "a" * 40, "v01.1.1", "v0.01.1", "v0.1.01", "v0.1", "v0.1.1.1",
            "v0.1.1-01", "v0.1.1-alpha..1", "v0.1.1+build..1", "v0.1.1-",
            "v0.1.1+", "v0.1.1-foo.lock", "v0.1.1/extra", "v0.1.1 ",
            "v0.1.1\n", "v0.1.1\nref=refs/heads/main", "v0.1.1-\u00e9",
            "v0.1.1$(touch${IFS}MARKER)",
            "v0.1.1" + chr(96) + "touch${IFS}MARKER" + chr(96),
            'v0.1.1";touch MARKER;#', "v0.1.1\n$(touch MARKER)",
        ]
        step = self.step("resolve-tag", "Validate release tag input")
        for event in ["workflow_dispatch", "release"]:
            for tag in inputs:
                with self.subTest(event=event, tag=tag):
                    result = self.run_step(step, tag, event)
                    self.assertEqual(
                        (
                            result.returncode != 0,
                            self.output.read_text(),
                            (self.root / "MARKER").exists(),
                            "Invalid release tag" in result.stderr,
                        ),
                        (True, "", False, True),
                        result.stderr,
                    )

    def test_tag_preflight_rejects_unsupported_events(self):
        result = self.run_step(
            self.step("resolve-tag", "Validate release tag input"),
            "v0.1.1",
            "push",
        )
        self.assertNotEqual(result.returncode, 0)

    def test_manual_tag_does_not_fall_back_to_release_context(self):
        result = self.run_step(
            self.step("resolve-tag", "Validate release tag input"),
            "",
            release_tag="v0.1.1",
        )
        self.assertNotEqual(result.returncode, 0)

    def test_checkouts_use_the_selected_tag_then_the_validated_commit(self):
        for job in ["validate", "publish"]:
            with self.subTest(job=job):
                checkout = next(
                    step for step in self.workflow["jobs"][job]["steps"]
                    if step.get("uses", "").startswith("actions/checkout@")
                )
                self.assertEqual(
                    checkout["with"],
                    {
                        "ref": (
                            "${{ needs.resolve-tag.outputs.ref }}" if job == "validate"
                            else "${{ needs.validate.outputs.commit }}"
                        ),
                        "persist-credentials": False,
                    },
                )

    def test_tag_preflight_runs_before_checkout_and_publishing(self):
        jobs = self.workflow["jobs"]
        self.assertEqual(
            (
                any("uses" in step for step in jobs["resolve-tag"]["steps"]),
                jobs["validate"]["needs"],
                jobs["publish"]["needs"],
            ),
            (False, "resolve-tag", ["resolve-tag", "validate"]),
        )

    def test_version_guards_accept_matching_lightweight_and_annotated_tags(self):
        for annotated in [False, True]:
            for version in ["0.1.1", "1.2.3-rc.1+build.001"]:
                self.manifest(version)
                self.git("add", "Cargo.toml")
                self.git("commit", "--quiet", "--allow-empty", "-m", "Version fixture")
                args = ["tag", "--force"]
                if annotated:
                    args.extend(["--annotate", "--message", "Release fixture"])
                self.git(*args, f"v{version}")
                for job in ["validate", "publish"]:
                    with self.subTest(job=job, annotated=annotated, version=version):
                        result = self.run_step(
                            self.step(job, "Check tag matches Cargo.toml version"),
                            f"v{version}",
                        )
                        self.assertEqual(result.returncode, 0, result.stderr)

    def test_version_guards_reject_a_different_manifest_version(self):
        self.manifest("0.2.0")
        for job in ["validate", "publish"]:
            with self.subTest(job=job):
                result = self.run_step(
                    self.step(job, "Check tag matches Cargo.toml version"), "v0.1.1"
                )
                self.assertNotEqual(result.returncode, 0)

    def test_version_guards_reject_a_branch_without_a_tag(self):
        self.git("tag", "--delete", "v0.1.1")
        self.git("branch", "v0.1.1")
        result = self.run_step(
            self.step("validate", "Check tag matches Cargo.toml version"), "v0.1.1"
        )
        self.assertNotEqual(result.returncode, 0)

    def test_version_guards_reject_a_checkout_different_from_the_tag(self):
        self.git("commit", "--quiet", "--allow-empty", "-m", "Different commit")
        self.git("branch", "v0.1.1")
        result = self.run_step(
            self.step("validate", "Check tag matches Cargo.toml version"), "v0.1.1"
        )
        self.assertNotEqual(result.returncode, 0)

    def test_version_guards_reject_a_tag_pointing_to_a_tree(self):
        self.git("tag", "--force", "v0.1.1", self.git("rev-parse", "HEAD^{tree}"))
        result = self.run_step(
            self.step("validate", "Check tag matches Cargo.toml version"), "v0.1.1"
        )
        self.assertNotEqual(result.returncode, 0)

    def test_authentication_follows_tag_verification(self):
        names = [
            step.get("name") for step in self.workflow["jobs"]["publish"]["steps"]
        ]
        self.assertLess(
            names.index("Check tag matches Cargo.toml version"),
            names.index("Authenticate with crates.io"),
        )

    def test_release_permissions_and_manual_validation_mode_are_preserved(self):
        # BaseLoader preserves YAML's "on" key and reads scalar values as text.
        workflow = yaml.load(WORKFLOW.read_text(), Loader=yaml.BaseLoader)
        publish = workflow["jobs"]["publish"]
        self.assertEqual(
            (
                workflow["permissions"],
                publish["permissions"],
                publish["environment"],
                publish["if"],
                workflow["on"]["workflow_dispatch"]["inputs"]["publish"]["default"],
            ),
            (
                {"contents": "read"},
                {"contents": "read", "id-token": "write"},
                "crates-io",
                "github.event_name == 'release' || inputs.publish",
                "false",
            ),
        )

    def test_actions_remain_pinned_to_commit_shas(self):
        actions = [
            step["uses"]
            for job in self.workflow["jobs"].values()
            for step in job["steps"]
            if "uses" in step
        ]
        self.assertTrue(
            all(re.fullmatch(r"[^@]+@[0-9a-f]{40}", action) for action in actions)
        )


if __name__ == "__main__":
    unittest.main()
