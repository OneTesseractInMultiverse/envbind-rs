"""Check that every supported platform contributes to the required Rust gate."""

import os
from pathlib import Path
import subprocess
import unittest

import yaml


WORKFLOW = Path(__file__).resolve().parents[1] / "workflows" / "ci.yml"


class CiWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.jobs = yaml.safe_load(WORKFLOW.read_text())["jobs"]

    def run_gate(self, result):
        step = self.jobs["rust-stable"]["steps"][0]
        environment = {"PATH": os.environ["PATH"]}
        environment.update({
            key: value.replace("${{ needs.rust.result }}", result)
            for key, value in step["env"].items()
        })
        return subprocess.run(
            ["bash", "-euo", "pipefail", "-c", step["run"]],
            env=environment, capture_output=True, text=True, check=False,
        )

    def test_stable_covers_every_supported_platform(self):
        entries = self.jobs["rust"]["strategy"]["matrix"]["include"]
        self.assertEqual(
            {entry["os"] for entry in entries if entry["toolchain"] == "stable"},
            {"ubuntu-latest", "macos-latest", "windows-latest"},
        )

    def test_msrv_keeps_its_required_check_name(self):
        entries = self.jobs["rust"]["strategy"]["matrix"]["include"]
        self.assertEqual(
            [(entry["name"], entry["os"]) for entry in entries if entry["toolchain"] == "1.85.0"],
            [("Rust 1.85.0", "ubuntu-latest")],
        )

    def test_required_stable_gate_runs_after_unsuccessful_matrix(self):
        gate = self.jobs["rust-stable"]
        self.assertEqual(
            (gate["name"], gate["needs"], gate["if"], gate.get("continue-on-error", False)),
            ("Rust stable", "rust", "${{ always() }}", False),
        )

    def test_gate_accepts_successful_matrix(self):
        self.assertEqual(self.run_gate("success").returncode, 0)

    def test_gate_rejects_failure_cancellation_skipping_and_unknown_results(self):
        for result in ["failure", "cancelled", "skipped", "", "unknown"]:
            with self.subTest(result=result):
                self.assertNotEqual(self.run_gate(result).returncode, 0)

    def test_matrix_cannot_ignore_failing_platforms(self):
        rust = self.jobs["rust"]
        gate = self.jobs["rust-stable"]
        self.assertFalse(any(
            job.get("continue-on-error", False)
            or any(step.get("continue-on-error", False) for step in job["steps"])
            for job in [rust, gate]
        ))

    def test_all_targets_run_without_platform_conditions(self):
        step = next(step for step in self.jobs["rust"]["steps"] if step.get("name") == "Test")
        self.assertEqual((step["run"], step.get("if")), ("cargo test --all-targets", None))
