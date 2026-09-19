"""Release handoff tests using local Git and dependency-free Cargo fixtures."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest

import test_publish_workflow as workflow


class ReleaseHandoffTests(unittest.TestCase):
    setUp = workflow.PublishWorkflowTests.setUp
    git = workflow.PublishWorkflowTests.git
    manifest = workflow.PublishWorkflowTests.manifest
    step = workflow.PublishWorkflowTests.step
    run_step = workflow.PublishWorkflowTests.run_step

    def cargo(self, *arguments):
        return subprocess.run(
            ["cargo", *arguments], cwd=self.root, env=self.environment,
            text=True, capture_output=True, timeout=60,
        )

    def prepare_package(self):
        self.environment.update({
            "RUSTUP_HOME": os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup")),
            "CARGO_HOME": str(self.root / "target" / "cargo-home"),
            "CARGO_NET_OFFLINE": "true",
        })
        (self.root / "src").mkdir()
        (self.root / "src" / "lib.rs").write_text("pub fn fixture() -> bool { true }\n")
        (self.root / ".gitignore").write_text("/target/\nCargo.lock\n")
        self.git("add", ".")
        self.git("commit", "--quiet", "-m", "Package fixture")
        self.git("tag", "--force", "v0.1.1")
        result = self.cargo("generate-lockfile", "--offline")
        if result.returncode:
            raise RuntimeError(result.stderr)
        self.bundle = self.root / "target" / "release"
        self.bundle.mkdir(parents=True)
        (self.bundle / "Cargo.lock").write_bytes((self.root / "Cargo.lock").read_bytes())
        (self.bundle / "audit.json").write_text('{"vulnerabilities":{"found":false}}\n')
        result = self.cargo("package", "--locked", "--offline")
        if result.returncode:
            raise RuntimeError(result.stderr)
        self.sha = self.git("rev-parse", "HEAD")

    def record(self):
        result = self.run_step(
            self.step("validate", "Record validated release"),
            "v0.1.1",
        )
        if result.returncode:
            raise RuntimeError(result.stderr)
        self.metadata_digest = hashlib.sha256(
            (self.bundle / "release.json").read_bytes()
        ).hexdigest()
        self.toolchain = json.loads((self.bundle / "release.json").read_text())["toolchain"]

    def restore(self):
        return self.run_step(
            self.step("publish", "Verify release bundle and restore lockfile"),
            "v0.1.1",
            context_values={
                "needs.validate.outputs.commit": self.sha,
                "needs.validate.outputs.metadata_sha256": self.metadata_digest,
            },
        )

    def test_publishing_checks_out_the_validation_commit(self):
        checkout = next(
            step for step in self.workflow["jobs"]["publish"]["steps"]
            if step.get("uses", "").startswith("actions/checkout@")
        )
        self.assertEqual(checkout["with"]["ref"], "${{ needs.validate.outputs.commit }}")

    def test_a_moved_tag_cannot_substitute_a_different_commit(self):
        validated = self.git("rev-parse", "HEAD")
        self.git("commit", "--quiet", "--allow-empty", "-m", "Different source")
        self.git("tag", "--force", "v0.1.1")
        result = self.run_step(
            self.step("publish", "Check tag matches Cargo.toml version"),
            "v0.1.1",
            context_values={"needs.validate.outputs.commit": validated},
        )
        self.assertNotEqual(result.returncode, 0)

    def test_the_validated_commit_remains_publishable_after_a_tag_moves(self):
        validated = self.git("rev-parse", "HEAD")
        self.git("commit", "--quiet", "--allow-empty", "-m", "Different source")
        self.git("tag", "--force", "v0.1.1")
        self.git("checkout", "--quiet", "--detach", validated)
        result = self.run_step(
            self.step("publish", "Check tag matches Cargo.toml version"),
            "v0.1.1",
            context_values={"needs.validate.outputs.commit": validated},
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_source_output_records_the_full_checked_out_commit(self):
        result = self.run_step(
            self.step("validate", "Check tag matches Cargo.toml version"), "v0.1.1"
        )
        self.assertEqual(
            (result.returncode, self.output.read_text()),
            (0, f"commit={self.git('rev-parse', 'HEAD')}\n"),
        )

    def test_recorded_metadata_connects_source_lock_audit_and_package(self):
        self.prepare_package()
        self.record()
        metadata = json.loads((self.bundle / "release.json").read_text())
        self.assertEqual(
            (
                metadata["commit"], metadata["tag"],
                metadata["lockfile_sha256"], metadata["crate_sha256"],
                metadata["audit_sha256"], metadata["run_id"], metadata["run_attempt"],
            ),
            (
                self.sha, "v0.1.1",
                hashlib.sha256((self.bundle / "Cargo.lock").read_bytes()).hexdigest(),
                hashlib.sha256((self.bundle / "package.crate").read_bytes()).hexdigest(),
                hashlib.sha256((self.bundle / "audit.json").read_bytes()).hexdigest(),
                "123", "1",
            ),
        )

    def test_recording_rejects_a_lockfile_changed_after_audit(self):
        self.prepare_package()
        with (self.root / "Cargo.lock").open("a") as lock:
            lock.write("\n# Drift after audit\n")
        result = self.run_step(
            self.step("validate", "Record validated release"), "v0.1.1"
        )
        self.assertNotEqual(result.returncode, 0)

    def test_verified_audited_lockfile_is_restored_exactly(self):
        self.prepare_package()
        self.record()
        (self.root / "Cargo.lock").unlink()
        result = self.restore()
        self.assertEqual(
            (result.returncode, (self.root / "Cargo.lock").read_bytes()),
            (0, (self.bundle / "Cargo.lock").read_bytes()),
            result.stderr,
        )

    def test_tampered_artifacts_fail_before_restoring_the_lockfile(self):
        self.prepare_package()
        self.record()
        for filename in ["release.json", "Cargo.lock", "audit.json", "package.crate"]:
            with self.subTest(filename=filename):
                (self.root / "Cargo.lock").unlink(missing_ok=True)
                path = self.bundle / filename
                original = path.read_bytes()
                path.write_bytes(original + b"\n")
                result = self.restore()
                path.write_bytes(original)
                self.assertEqual(
                    (result.returncode != 0, (self.root / "Cargo.lock").exists()),
                    (True, False),
                    result.stderr,
                )

    def test_same_commit_and_lock_rebuild_the_same_package_after_tag_movement(self):
        self.prepare_package()
        self.record()
        self.git("commit", "--quiet", "--allow-empty", "-m", "Moved tag")
        self.git("tag", "--force", "v0.1.1")
        self.git("checkout", "--quiet", "--detach", self.sha)
        restored = self.restore()
        rebuilt = self.cargo("package", "--locked", "--offline")
        compared = self.run_step(
            self.step("publish", "Compare rebuilt package"), "v0.1.1"
        )
        self.assertEqual(
            (restored.returncode, rebuilt.returncode, compared.returncode),
            (0, 0, 0),
            restored.stderr + rebuilt.stderr + compared.stderr,
        )

    def test_rebuilt_package_drift_is_rejected(self):
        self.prepare_package()
        self.record()
        package = self.root / "target" / "package" / "fixture-0.1.1.crate"
        package.write_bytes(package.read_bytes() + b"\n")
        result = self.run_step(
            self.step("publish", "Compare rebuilt package"), "v0.1.1"
        )
        self.assertNotEqual(result.returncode, 0)

    def test_locked_cargo_fails_instead_of_resolving_new_dependencies(self):
        self.prepare_package()
        original = (self.root / "Cargo.lock").read_bytes()
        dependency = self.root / "dependency"
        (dependency / "src").mkdir(parents=True)
        (dependency / "Cargo.toml").write_text(
            '[package]\nname="dependency"\nversion="0.1.0"\nedition="2024"\n'
        )
        (dependency / "src" / "lib.rs").write_text("pub fn dependency() {}\n")
        with (self.root / "Cargo.toml").open("a") as manifest:
            manifest.write('\n[dependencies]\ndependency = { path = "dependency" }\n')
        result = self.cargo("check", "--locked", "--offline")
        self.assertEqual(
            (result.returncode != 0, (self.root / "Cargo.lock").read_bytes()),
            (True, original),
        )

    def test_release_commands_preserve_the_audited_resolution(self):
        validate = self.workflow["jobs"]["validate"]["steps"]
        audit = self.step("validate", "Audit dependencies")["run"]
        commands = [
            self.step("validate", name)["run"]
            for name in ["Verify", "Inspect package contents", "Package", "Publish dry run"]
        ]
        commands.extend(
            self.step("publish", name)["run"]
            for name in ["Rebuild validated package", "Publish"]
        )
        self.assertEqual(
            (
                all("--locked" in command for command in commands),
                "rm -f Cargo.lock" in audit,
                "cp Cargo.lock target/release/Cargo.lock" in audit,
                any(step.get("name") == "Record validated release" for step in validate),
            ),
            (True, False, True, True),
        )

    def test_publish_uses_the_exact_artifact_id_from_validation(self):
        download = self.step("publish", "Download validated release")
        self.assertEqual(
            download["with"],
            {
                "artifact-ids": "${{ needs.validate.outputs.artifact_id }}",
                "path": "target/release",
            },
        )

    def test_bundle_verification_and_comparison_precede_authentication(self):
        names = [step.get("name") for step in self.workflow["jobs"]["publish"]["steps"]]
        self.assertTrue(
            names.index("Verify release bundle and restore lockfile")
            < names.index("Rebuild validated package")
            < names.index("Compare rebuilt package")
            < names.index("Authenticate with crates.io")
            < names.index("Publish")
        )

    def fake_publisher(self):
        # Replace only the upload invocation; fixture packaging uses real Cargo.
        directory = self.root / ".git" / "fake-bin"
        directory.mkdir()
        version = json.loads((self.bundle / "release.json").read_text())["cargo_version"]
        script = directory / "cargo"
        script.write_text(
            f"#!{sys.executable}\n"
            "import json, sys\nfrom pathlib import Path\n"
            "if sys.argv[1:] == ['--version']:\n"
            f"    print({version!r})\n"
            "elif sys.argv[1:] == ['publish', '--locked', '--no-verify']:\n"
            "    Path('.git/publish-command').write_text(json.dumps(sys.argv[1:]))\n"
            "else:\n    sys.exit('Unexpected Cargo command')\n"
        )
        script.chmod(0o755)
        self.environment["PATH"] = str(directory) + os.pathsep + self.environment["PATH"]

    def test_final_publication_uses_locked_inputs_without_another_build(self):
        self.prepare_package()
        self.record()
        self.fake_publisher()
        result = self.run_step(
            self.step("publish", "Publish"), "v0.1.1",
            context_values={"steps.auth.outputs.token": "fixture-token"},
        )
        self.assertEqual(
            (
                result.returncode,
                json.loads((self.root / ".git" / "publish-command").read_text()),
                "fixture-token" in result.stdout + result.stderr,
            ),
            (0, ["publish", "--locked", "--no-verify"], False),
            result.stderr,
        )

    def test_final_guard_rejects_drift_before_invoking_publication(self):
        self.prepare_package()
        self.record()
        self.fake_publisher()
        (self.root / "Cargo.lock").write_text("changed after authentication")
        result = self.run_step(
            self.step("publish", "Publish"), "v0.1.1",
            context_values={"steps.auth.outputs.token": "fixture-token"},
        )
        self.assertEqual(
            (
                result.returncode != 0,
                (self.root / ".git" / "publish-command").exists(),
                "fixture-token" in result.stdout + result.stderr,
            ),
            (True, False, False),
        )

    def test_metadata_toolchain_must_match_the_validation_output(self):
        self.prepare_package()
        self.record()
        self.toolchain = "0.0.0"
        result = self.restore()
        self.assertNotEqual(result.returncode, 0)

    def test_validation_handoff_exports_source_toolchain_and_artifact_identity(self):
        self.assertEqual(
            self.workflow["jobs"]["validate"]["outputs"],
            {
                "commit": "${{ steps.source.outputs.commit }}",
                "toolchain": "${{ steps.record.outputs.toolchain }}",
                "metadata_sha256": "${{ steps.record.outputs.metadata_sha256 }}",
                "artifact_id": "${{ steps.upload.outputs.artifact-id }}",
            },
        )
