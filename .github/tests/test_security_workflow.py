"""Exercise advisory failure propagation and preserve the full pull request gate."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_text())
JOB = WORKFLOW["jobs"]["security"]
AUDIT = next(step for step in JOB["steps"] if step.get("name") == "Audit dependencies")


class SecurityAuditTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="envbind-security-tests-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "fuzz").mkdir()
        (self.root / "fuzz/Cargo.lock").write_text("committed fuzz graph\n")
        (self.root / "Cargo.lock").write_text("stale local graph\n")
        tools = self.root / "tools"
        tools.mkdir()
        cargo = tools / "cargo"
        cargo.write_text(f"#!{sys.executable}\n" + '''
import json, os, pathlib, sys
args = sys.argv[1:]
with open("trace.jsonl", "a") as trace:
    trace.write(json.dumps(args) + "\\n")
if args == ["generate-lockfile"]:
    if pathlib.Path("Cargo.lock").exists():
        sys.exit(91)
    mode = os.environ["CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS"]
    if mode == os.environ.get("FAIL_RESOLUTION"):
        sys.exit(92)
    pathlib.Path("Cargo.lock").write_text(mode + "\\n")
elif args[0] == "audit":
    lock = pathlib.Path(args[args.index("--file") + 1])
    print("Auditing " + lock.read_text().strip())
    if lock.stem == os.environ.get("FAIL_GRAPH"):
        print("Synthetic advisory, warning, or fetch failure", file=sys.stderr)
        sys.exit(93)
else:
    sys.exit(94)
''')
        cargo.chmod(0o755)
        self.environment = {"PATH": str(tools) + os.pathsep + os.environ["PATH"]}

    def run_audit(self, **overrides):
        return subprocess.run(["bash", "--noprofile", "--norc", "-e", "-o", "pipefail",
                               "-c", AUDIT["run"]], cwd=self.root,
                              env={**self.environment, **overrides},
                              capture_output=True, text=True, check=False)

    def audit_calls(self):
        calls = [json.loads(line) for line in (self.root / "trace.jsonl").read_text().splitlines()]
        return [call for call in calls if call[0] == "audit"]

    def test_success_audits_fresh_resolutions_and_unchanged_fuzz_lock(self):
        result = self.run_audit()
        evidence = self.root / "target/security-audit"
        self.assertEqual(
            (result.returncode,
             [(evidence / f"{graph}.lock").read_text() for graph in ["root-fallback", "root-allow", "fuzz"]],
             (self.root / "fuzz/Cargo.lock").read_text(), (self.root / "Cargo.lock").exists()),
            (0, ["fallback\n", "allow\n", "committed fuzz graph\n"], "committed fuzz graph\n", False),
            result.stderr,
        )

    def test_every_audit_denies_warnings_without_fetch_target_or_advisory_bypasses(self):
        self.run_audit()
        self.assertEqual(self.audit_calls(), [
            ["audit", "--deny", "warnings", "--json", "--file", f"target/security-audit/{graph}.lock"]
            for graph in ["root-fallback", "root-allow", "fuzz"]
        ])

    def test_failure_in_any_graph_fails_job_and_still_checks_other_graphs(self):
        for graph in ["root-fallback", "root-allow", "fuzz"]:
            with self.subTest(graph=graph):
                (self.root / "trace.jsonl").write_text("")
                result = self.run_audit(FAIL_GRAPH=graph)
                report = self.root / f"target/security-audit/{graph}.json"
                self.assertEqual((result.returncode, len(self.audit_calls()),
                                  report.exists(),
                                  "Synthetic advisory, warning, or fetch failure" in result.stderr),
                                 (1, 3, True, True), result.stderr)

    def test_resolution_failure_cannot_be_reported_as_clean_audit(self):
        for mode in ["fallback", "allow"]:
            with self.subTest(mode=mode):
                (self.root / "trace.jsonl").write_text("")
                result = self.run_audit(FAIL_RESOLUTION=mode)
                self.assertEqual((result.returncode, self.audit_calls()), (92, []))


class SecurityWorkflowTests(unittest.TestCase):
    def test_schedule_and_manual_audit_only_mode_keep_existing_triggers(self):
        # PyYAML's YAML 1.1 loader reads the Actions `on` key as True.
        self.assertEqual(WORKFLOW[True], {
            "pull_request": None, "push": {"branches": ["main"]},
            "workflow_dispatch": {"inputs": {"security_only": {
                "description": "Run only the dependency advisory audit", "type": "boolean", "default": False,
            }}}, "schedule": [{"cron": "23 7 * * *"}],
        })

    def test_full_ci_is_skipped_only_for_schedule_or_explicit_manual_audit(self):
        condition = ("github.event_name != 'schedule' && "
                     "!(github.event_name == 'workflow_dispatch' && inputs.security_only)")
        self.assertEqual(
            {name: job.get("if") for name, job in WORKFLOW["jobs"].items()},
            {"workflow-tests": condition, "rust": condition, "fuzz": condition,
             "rust-stable": "${{ always() && " + condition + " }}", "security": None},
        )

    def test_audit_cannot_cancel_full_ci_or_a_different_event(self):
        self.assertEqual(WORKFLOW["concurrency"]["group"],
                         "ci-${{ github.workflow }}-${{ github.event_name }}-${{ github.ref }}-${{ inputs.security_only || false }}")

    def test_security_job_is_read_only_bounded_and_cannot_hide_failures(self):
        self.assertEqual(
            (WORKFLOW["permissions"], JOB.get("permissions"), JOB["name"], JOB["timeout-minutes"],
             JOB.get("needs"), JOB["defaults"]["run"]["shell"], JOB.get("continue-on-error", False),
             any(step.get("continue-on-error", False) for step in JOB["steps"]), AUDIT.get("if"),
             JOB["steps"][0]["with"]["persist-credentials"]),
            ({"contents": "read"}, None, "Security audit", 15, None, "bash", False, False, None, False),
        )

    def test_audit_evidence_is_retained_on_failure_with_bounded_retention(self):
        step = next(step for step in JOB["steps"] if step.get("name") == "Retain audited resolutions and reports")
        self.assertEqual((step["if"], step["with"]["path"], step["with"]["retention-days"]),
                         ("${{ !cancelled() }}", "target/security-audit/", 14))

    def test_weekly_updates_still_cover_all_manifests(self):
        updates = yaml.safe_load((ROOT / ".github/dependabot.yml").read_text())["updates"]
        self.assertEqual({(item["package-ecosystem"], item["directory"], item["schedule"]["interval"])
                          for item in updates},
                         {("cargo", "/", "weekly"), ("cargo", "/fuzz", "weekly"),
                          ("github-actions", "/", "weekly"), ("pip", "/.github/tests", "weekly")})
