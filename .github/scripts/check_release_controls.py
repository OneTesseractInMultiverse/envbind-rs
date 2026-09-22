#!/usr/bin/env python3
"""Inspect GitHub release controls without modifying settings or requesting tokens."""

import argparse
import json
from pathlib import Path
import subprocess
import sys


def gh_json(endpoint, *, paginate=False, runner=subprocess.run):
    command = ["gh", "api", endpoint]
    if paginate:
        command.extend(["--paginate", "--slurp"])
    result = runner(command, capture_output=True, text=True, check=False)
    if result.returncode:
        raise RuntimeError(f"Cannot inspect {endpoint}; check gh authentication and repository access")
    return json.loads(result.stdout)


def check_controls(policy, environment, refs, tags, main):
    errors = []

    def require(condition, message):
        if not condition:
            errors.append(message)

    def rules_by_type(ruleset):
        return {rule["type"]: rule.get("parameters", {}) for rule in ruleset.get("rules", [])}

    expected = policy["environment"]
    require(environment.get("name") == expected["name"], "Environment identity does not match policy")
    require(
        environment.get("can_admins_bypass") is expected["can_admins_bypass"],
        "Environment administrator bypass does not match policy",
    )
    reviewers = [rule for rule in environment.get("protection_rules", []) if rule.get("type") == "required_reviewers"]
    review = reviewers[0] if len(reviewers) == 1 else {}
    actual_reviewers = [
        (entry.get("type"), entry.get("reviewer", {}).get("id"), entry.get("reviewer", {}).get("login"))
        for entry in review.get("reviewers", [])
    ]
    require(
        actual_reviewers == [("User", expected["reviewer_id"], expected["reviewer"])],
        "Required reviewers do not match policy",
    )
    require(review.get("prevent_self_review") is expected["prevent_self_review"], "Self-review policy does not match policy")
    require(
        environment.get("deployment_branch_policy") == {"protected_branches": False, "custom_branch_policies": True},
        "Selected deployment refs are not enforced",
    )
    require(
        sorted((entry.get("type", ""), entry.get("name", "")) for entry in refs.get("branch_policies", []))
        == sorted((entry["type"], entry["name"]) for entry in policy["deployment_refs"]),
        "Deployment branch/tag patterns do not match policy",
    )

    expected_tags = policy["tag_ruleset"]
    require(
        tags.get("target") == "tag" and tags.get("enforcement") == "active",
        "Release-tag ruleset is not active for tags",
    )
    require(
        tags.get("conditions", {}).get("ref_name") == {
            "include": expected_tags["include"], "exclude": expected_tags["exclude"],
        },
        "Release-tag patterns do not match policy",
    )
    require(tags.get("bypass_actors") == expected_tags["bypass_actors"], "Release-tag bypass actors do not match policy")
    require(set(rules_by_type(tags)) == {"update", "deletion"}, "Release tags must prohibit updates and deletion")

    require(
        main.get("target") == "branch" and main.get("enforcement") == "active"
        and main.get("conditions", {}).get("ref_name") == {"include": ["~DEFAULT_BRANCH"], "exclude": []},
        "Main ruleset is not active for the default branch",
    )
    main_rules = rules_by_type(main)
    require({"deletion", "non_fast_forward"}.issubset(main_rules), "Main deletion/force-push protection is missing")
    pull_request = main_rules.get("pull_request", {})
    require(pull_request.get("required_approving_review_count", 0) >= 1, "Main requires at least one approving review")
    for option in ["dismiss_stale_reviews_on_push", "require_code_owner_review", "require_last_push_approval", "required_review_thread_resolution"]:
        require(pull_request.get(option) is True, f"Main review protection is missing: {option}")
    status = main_rules.get("required_status_checks", {})
    require(status.get("strict_required_status_checks_policy") is True, "Main strict required checks are not enabled")
    required_checks = {(entry.get("context"), entry.get("integration_id")) for entry in status.get("required_status_checks", [])}
    require(
        {(name, 15368) for name in policy["main_ruleset"]["required_checks"]}.issubset(required_checks),
        "Main required checks do not match policy",
    )
    return errors


def inspect(policy):
    prefix = "repos/" + policy["repository"]
    environment_path = prefix + "/environments/" + policy["environment"]["name"]
    environment = gh_json(environment_path)
    ref_pages = gh_json(environment_path + "/deployment-branch-policies?per_page=100", paginate=True)
    refs = {"branch_policies": [entry for page in ref_pages for entry in page["branch_policies"]]}
    rulesets = [entry for page in gh_json(prefix + "/rulesets?per_page=100", paginate=True) for entry in page]

    def ruleset(name):
        matches = [entry for entry in rulesets if entry["name"] == name]
        if len(matches) != 1:
            raise RuntimeError(f"Expected exactly one repository ruleset named {name!r}")
        return gh_json(prefix + "/rulesets/" + str(matches[0]["id"]))

    repository = gh_json(prefix)
    if repository.get("default_branch") != "main":
        raise RuntimeError("Default branch changed; review deployment ref policy")
    return check_controls(policy, environment, refs, ruleset(policy["tag_ruleset"]["name"]), ruleset(policy["main_ruleset"]["name"]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--policy", type=Path, default=Path(__file__).resolve().parents[1] / "release-policy.json")
    args = parser.parse_args()
    try:
        policy = json.loads(args.policy.read_text())
        errors = inspect(policy)
    except (OSError, ValueError, KeyError, TypeError, RuntimeError) as error:
        print(f"Release controls could not be verified: {error}", file=sys.stderr)
        return 1
    if errors:
        for error in errors:
            print(f"Release control drift: {error}", file=sys.stderr)
        return 1
    print("GitHub release controls match .github/release-policy.json.")
    publisher = policy["trusted_publisher"]
    print("Separately verify the crates.io Trusted Publisher row: " + json.dumps(publisher, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
