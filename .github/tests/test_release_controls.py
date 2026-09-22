"""Read-only release preflight policy checks using local API fixtures."""

import copy
import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "check_release_controls.py"
SPEC = importlib.util.spec_from_file_location("release_controls", SCRIPT)
controls = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(controls)


class ReleaseControlsTests(unittest.TestCase):
    def setUp(self):
        self.policy = {
            "environment": {
                "name": "registry", "reviewer": "maintainer", "reviewer_id": 7,
                "prevent_self_review": False, "can_admins_bypass": False,
            },
            "deployment_refs": [{"type": "branch", "name": "main"}, {"type": "tag", "name": "v*"}],
            "tag_ruleset": {
                "name": "Release tags", "include": ["refs/tags/v*"],
                "exclude": [], "bypass_actors": [],
            },
            "main_ruleset": {"name": "Main", "required_checks": ["Rust stable"]},
        }
        self.environment = {
            "name": "registry", "can_admins_bypass": False,
            "deployment_branch_policy": {"protected_branches": False, "custom_branch_policies": True},
            "protection_rules": [{
                "type": "required_reviewers", "prevent_self_review": False,
                "reviewers": [{"type": "User", "reviewer": {"id": 7, "login": "maintainer"}}],
            }],
        }
        self.refs = {"branch_policies": copy.deepcopy(self.policy["deployment_refs"])}
        self.tags = {
            "name": "Release tags", "target": "tag", "enforcement": "active",
            "conditions": {"ref_name": {"include": ["refs/tags/v*"], "exclude": []}},
            "bypass_actors": [], "rules": [{"type": "update"}, {"type": "deletion"}],
        }
        self.main = {
            "name": "Main", "target": "branch", "enforcement": "active",
            "conditions": {"ref_name": {"include": ["~DEFAULT_BRANCH"], "exclude": []}},
            "rules": [
                {"type": "deletion"}, {"type": "non_fast_forward"},
                {"type": "pull_request", "parameters": {
                    "required_approving_review_count": 1, "dismiss_stale_reviews_on_push": True,
                    "require_code_owner_review": True, "require_last_push_approval": True,
                    "required_review_thread_resolution": True,
                }},
                {"type": "required_status_checks", "parameters": {
                    "strict_required_status_checks_policy": True,
                    "required_status_checks": [{"context": "Rust stable", "integration_id": 15368}],
                }},
            ],
        }

    def check(self):
        return controls.check_controls(self.policy, self.environment, self.refs, self.tags, self.main)

    def test_configured_policy_passes(self):
        self.assertEqual(self.check(), [])

    def test_missing_environment_cannot_pass(self):
        self.environment = {}
        self.assertIn("Environment identity does not match policy", self.check())

    def test_admin_bypass_must_be_explicitly_disabled(self):
        for value in [True, None]:
            with self.subTest(value=value):
                self.environment["can_admins_bypass"] = value
                self.assertIn("Environment administrator bypass does not match policy", self.check())

    def test_required_reviewer_cannot_be_removed_or_replaced(self):
        for reviewers in [[], [{"type": "User", "reviewer": {"id": 8, "login": "other"}}]]:
            with self.subTest(reviewers=reviewers):
                self.environment["protection_rules"][0]["reviewers"] = reviewers
                self.assertIn("Required reviewers do not match policy", self.check())

    def test_self_review_policy_drift_is_detected(self):
        self.environment["protection_rules"][0]["prevent_self_review"] = True
        self.assertIn("Self-review policy does not match policy", self.check())

    def test_unrestricted_deployment_refs_are_rejected(self):
        self.environment["deployment_branch_policy"] = None
        self.assertIn("Selected deployment refs are not enforced", self.check())

    def test_ref_type_matters_even_when_names_match(self):
        self.refs["branch_policies"][0]["type"] = "tag"
        self.assertIn("Deployment branch/tag patterns do not match policy", self.check())

    def test_additional_allowed_refs_are_detected(self):
        self.refs["branch_policies"].append({"type": "branch", "name": "*"})
        self.assertIn("Deployment branch/tag patterns do not match policy", self.check())

    def test_inactive_tag_rules_are_rejected(self):
        self.tags["enforcement"] = "disabled"
        self.assertIn("Release-tag ruleset is not active for tags", self.check())

    def test_tag_exceptions_are_detected(self):
        self.tags["conditions"]["ref_name"]["exclude"] = ["refs/tags/v*"]
        self.assertIn("Release-tag patterns do not match policy", self.check())

    def test_tag_bypass_is_detected(self):
        self.tags["bypass_actors"] = [{"actor_type": "RepositoryRole", "actor_id": 5, "bypass_mode": "always"}]
        self.assertIn("Release-tag bypass actors do not match policy", self.check())

    def test_both_tag_update_and_deletion_rules_are_required(self):
        self.tags["rules"] = [{"type": "non_fast_forward"}]
        self.assertIn("Release tags must prohibit updates and deletion", self.check())

    def test_main_review_requirement_cannot_disappear(self):
        self.main["rules"][2]["parameters"]["required_approving_review_count"] = 0
        self.assertIn("Main requires at least one approving review", self.check())

    def test_main_required_checks_cannot_disappear(self):
        self.main["rules"][3]["parameters"]["required_status_checks"] = []
        self.assertIn("Main required checks do not match policy", self.check())

    def test_missing_api_access_fails_closed(self):
        with self.assertRaises(RuntimeError):
            controls.gh_json("repos/example/missing", runner=lambda *args, **kwargs: type(
                "Failure", (), {"returncode": 1, "stderr": "HTTP 403", "stdout": ""}
            )())
