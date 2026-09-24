"""Exercise fuzz budgets, failure propagation, and retention without fuzzing."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[2]
TARGETS = ["json", "base64", "list", "url", "scalars"]


class FuzzRunnerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="envbind-fuzz-tests-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        (self.root / "fuzz").mkdir()
        shutil.copyfile(ROOT / "fuzz/run.sh", self.root / "fuzz/run.sh")
        (self.root / "fuzz/Cargo.lock").write_text("# Synthetic lock fixture\n")
        for target in TARGETS:
            corpus = self.root / "fuzz/corpus" / target
            corpus.mkdir(parents=True)
            (corpus / "seed").write_text("synthetic-seed")
        self.trace = self.root / "trace.jsonl"
        tools = self.root / "tools"
        tools.mkdir()
        cargo = tools / "cargo"
        cargo.write_text(f"#!{sys.executable}\n" + '''
import json, os, pathlib, sys
args = sys.argv[1:]
with open(os.environ["TRACE"], "a") as output:
    output.write(json.dumps({"args": args, "offline": os.environ.get("CARGO_NET_OFFLINE")}) + "\\n")
if args[1:3] == ["fuzz", "--version"]:
    print("cargo-fuzz 0.13.2")
if args[1:3] == ["fuzz", "build"] and os.environ.get("DRIFT_LOCK"):
    with open("fuzz/Cargo.lock", "a") as lock:
        lock.write("drift\\n")
if args[1:3] == ["fuzz", "build"] and os.environ.get("FAIL_BUILD"):
    sys.exit(77)
if args[1:3] == ["fuzz", "run"] and args[3] == os.environ.get("FAIL_TARGET"):
    sys.exit(77)
''')
        cargo.chmod(0o755)
        rustc = tools / "rustc"
        rustc.write_text("#!/bin/sh\nprintf 'synthetic compiler metadata\\n'\n")
        rustc.chmod(0o755)
        self.environment = {
            "PATH": str(tools) + os.pathsep + os.environ["PATH"],
            "HOME": str(self.root),
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "TRACE": str(self.trace),
        }
        self.git("init", "-q")
        self.git("add", ".")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.test",
                 "-c", "commit.gpgsign=false", "commit", "-qm", "Synthetic fuzz fixture")

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.root, env=self.environment,
                              capture_output=True, text=True, check=True)

    def run_runner(self, *args, **overrides):
        return subprocess.run(["bash", "fuzz/run.sh", *args], cwd=self.root,
                              env={**self.environment, **overrides},
                              capture_output=True, text=True, check=False)

    def invocations(self):
        if not self.trace.exists():
            return []
        return [json.loads(line) for line in self.trace.read_text().splitlines()]

    def fuzz_runs(self):
        return [call for call in self.invocations() if call["args"][1:3] == ["fuzz", "run"]]

    def test_every_target_receives_bounds_and_separate_writable_corpus(self):
        result = self.run_runner("2", "12345")
        observed = []
        for call in self.fuzz_runs():
            args = call["args"]
            target = args[3]
            observed.append((target, call["offline"], args[4:6], args[7:]))
        output = self.root / "fuzz/results"
        expected = [(target, "true", [str(output / "corpus" / target), str(self.root / "fuzz/corpus" / target)],
                     ["-max_total_time=2", "-seed=12345", "-max_len=4096", "-timeout=5",
                      "-rss_limit_mb=1024", "-malloc_limit_mb=16",
                      f"-artifact_prefix={output}/artifacts/{target}/", "-print_final_stats=1"])
                    for target in TARGETS]
        self.assertEqual((result.returncode, observed), (0, expected), result.stderr)

    def test_failure_reaches_ci_through_log_pipeline_and_stops_later_targets(self):
        result = self.run_runner("1", "1", FAIL_TARGET="list")
        self.assertEqual((result.returncode, [call["args"][3] for call in self.fuzz_runs()]),
                         (77, ["json", "base64", "list"]))

    def test_build_failure_does_not_start_fuzzing(self):
        result = self.run_runner("1", "1", FAIL_BUILD="1")
        self.assertEqual((result.returncode, self.fuzz_runs()), (77, []))

    def test_lock_drift_blocks_fuzzing(self):
        result = self.run_runner("1", "1", DRIFT_LOCK="1")
        self.assertEqual((result.returncode, self.fuzz_runs()), (1, []))

    def test_invalid_budgets_and_seeds_do_not_execute_tools(self):
        cases = [(value, "1") for value in ["0", "-1", "3601", "99999", "1;exit 0", "$(exit 0)"]]
        cases += [("1", value) for value in ["0", "-1", "1000000000", "x", "$(exit 0)"]]
        statuses = [self.run_runner(*args).returncode for args in cases]
        self.assertEqual((statuses, self.invocations()), ([2] * len(cases), []))

    def test_seed_corpus_remains_unchanged_and_metadata_is_retained(self):
        result = self.run_runner("1", "1")
        output = self.root / "fuzz/results"
        retained = {name: (output / name).exists() for name in
                    ["toolchain.txt", "cargo-fuzz.txt", "source-commit.txt", "source.patch", "budget.txt", "Cargo.lock"]}
        seeds = [(self.root / "fuzz/corpus" / target / "seed").read_text() for target in TARGETS]
        self.assertEqual((result.returncode, seeds, all(retained.values())),
                         (0, ["synthetic-seed"] * len(TARGETS), True))


class FuzzWorkflowTests(unittest.TestCase):
    def test_smoke_job_has_bounded_time_and_cannot_ignore_failure(self):
        workflow = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_text())
        job = workflow["jobs"]["fuzz"]
        step = next(step for step in job["steps"] if step.get("name") == "Run bounded fuzz targets")
        self.assertEqual((workflow["permissions"], job["timeout-minutes"], step["run"],
                          job.get("continue-on-error", False), step.get("continue-on-error", False)),
                         ({"contents": "read"}, 15, "bash fuzz/run.sh 20 20260924", False, False))

    def test_failure_artifacts_have_short_retention_and_exclude_mutated_corpus(self):
        job = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_text())["jobs"]["fuzz"]
        step = next(step for step in job["steps"] if step.get("name") == "Retain synthetic failure evidence")
        self.assertEqual((step["if"], step["with"]["retention-days"], step["with"]["path"].splitlines()),
                         ("failure()", 7, ["fuzz/results/*.log", "fuzz/results/*.txt",
                                          "fuzz/results/Cargo.lock", "fuzz/results/source.patch",
                                          "fuzz/results/artifacts/**"]))

    def test_committed_seeds_match_targets_and_fit_input_domain(self):
        corpus = ROOT / "fuzz/corpus"
        files = [path for path in corpus.rglob("*") if path.is_file()]
        invalid = [str(path) for path in files if len(path.read_bytes()) > 4096]
        for path in files:
            path.read_bytes().decode("utf-8")
        unseeded = [target for target in TARGETS
                    if not any(path.is_file() and path.stat().st_size > 0
                               for path in (corpus / target).iterdir())]
        self.assertEqual((sorted(path.name for path in corpus.iterdir()), invalid, unseeded),
                         (sorted(TARGETS), [], []))
