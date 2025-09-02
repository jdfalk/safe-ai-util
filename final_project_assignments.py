#!/usr/bin/env python3
# file: final_project_assignments.py
# version: 1.0.0
# guid: 1b2c3d4e-5f6a-7b8c-9d0e-1f2a3b4c5d6e

"""
Final project assignment script based on actual GitHub issues.
Generates specific assignment instructions for existing GitHub Projects.
"""

from typing import List


def categorize_real_issue(issue_number: int, title: str, labels: List[str]) -> str:
    """Categorize actual issues based on their labels and content"""
    label_set = {label.lower() for label in labels}
    title_lower = title.lower()

    # Backend Services - APIs, authentication, databases, metrics, gRPC
    backend_indicators = {
        "module:auth",
        "module:api",
        "module:database",
        "module:metrics",
        "module:cache",
        "module:config",
        "module:queue",
        "auth",
        "grpc",
        "proto",
        "protobuf",
        "migration",
    }

    # Web & UI - web services, frontend, UI components
    web_ui_indicators = {"module:web", "module:ui", "frontend", "web", "ui"}

    # Infrastructure - CI/CD, deployment, automation, GitHub Actions
    infrastructure_indicators = {
        "ci-cd",
        "github-actions",
        "deployment",
        "build",
        "release",
        "automation",
        "workflow",
        "docker",
        "cosign",
    }

    # Documentation - docs, examples, guides
    documentation_indicators = {"documentation", "docs", "examples", "guide", "readme"}

    # Testing - test cases, QA, validation
    testing_indicators = {
        "test",
        "testing",
        "integration",
        "unit-tests",
        "qa",
        "validation",
        "area:testing",
    }

    # SDKs & Tools - CLI, utilities, development tools
    tools_indicators = {
        "sdk",
        "tools",
        "cli",
        "utility",
        "script",
        "generator",
        "dependencies",
    }

    # Check categories in priority order
    if label_set.intersection(backend_indicators) or any(
        word in title_lower for word in ["grpc", "auth", "proto", "database"]
    ):
        return "Backend Services"
    elif label_set.intersection(web_ui_indicators) or "/api/" in title_lower:
        return "Web & UI"
    elif label_set.intersection(infrastructure_indicators) or any(
        word in title_lower for word in ["ci", "workflow", "build"]
    ):
        return "Infrastructure"
    elif label_set.intersection(documentation_indicators) or "doc" in title_lower:
        return "Documentation"
    elif label_set.intersection(testing_indicators) or "test" in title_lower:
        return "Testing"
    elif label_set.intersection(tools_indicators) or any(
        word in title_lower for word in ["cli", "tool", "script"]
    ):
        return "SDKs & Tools"
    else:
        return "General"


def analyze_actual_issues():
    """Analyze the actual issues we found and categorize them"""

    # Real issues from subtitle-manager
    subtitle_manager_issues = [
        {
            "number": 1789,
            "title": "[Bug]: dependabot still isn't using the right labels",
            "labels": [
                "bug",
                "enhancement",
                "priority:medium",
                "module:config",
                "module:ui",
            ],
            "category": categorize_real_issue(
                1789,
                "[Bug]: dependabot still isn't using the right labels",
                ["bug", "enhancement", "priority:medium", "module:config", "module:ui"],
            ),
        },
        {
            "number": 1772,
            "title": "Add test for /api/sync/batch invalid JSON",
            "labels": [
                "enhancement",
                "module:web",
                "priority:medium",
                "module:ui",
                "test",
            ],
            "category": categorize_real_issue(
                1772,
                "Add test for /api/sync/batch invalid JSON",
                ["enhancement", "module:web", "priority:medium", "module:ui", "test"],
            ),
        },
        {
            "number": 1771,
            "title": "Add gRPC health service",
            "labels": ["enhancement", "codex", "priority:medium", "module:ui"],
            "category": categorize_real_issue(
                1771,
                "Add gRPC health service",
                ["enhancement", "codex", "priority:medium", "module:ui"],
            ),
        },
        {
            "number": 1770,
            "title": "Finish metrics proto migration",
            "labels": [
                "enhancement",
                "priority:medium",
                "refactor",
                "module:metrics",
                "module:ui",
            ],
            "category": categorize_real_issue(
                1770,
                "Finish metrics proto migration",
                [
                    "enhancement",
                    "priority:medium",
                    "refactor",
                    "module:metrics",
                    "module:ui",
                ],
            ),
        },
        {
            "number": 1769,
            "title": "Implement gcommon logrus provider",
            "labels": ["enhancement", "priority:medium", "module:ui"],
            "category": categorize_real_issue(
                1769,
                "Implement gcommon logrus provider",
                ["enhancement", "priority:medium", "module:ui"],
            ),
        },
    ]

    # Real issues from gcommon
    gcommon_issues = [
        {
            "number": 1065,
            "title": "Integration testing framework implementation",
            "labels": ["enhancement", "priority:medium", "testing", "area:testing"],
            "category": categorize_real_issue(
                1065,
                "Integration testing framework implementation",
                ["enhancement", "priority:medium", "testing", "area:testing"],
            ),
        },
        {
            "number": 1064,
            "title": "Fix create-issue-update GH_TOKEN requirement",
            "labels": ["bug", "priority:medium", "maintenance", "needs-triage"],
            "category": categorize_real_issue(
                1064,
                "Fix create-issue-update GH_TOKEN requirement",
                ["bug", "priority:medium", "maintenance", "needs-triage"],
            ),
        },
        {
            "number": 1063,
            "title": "Standardize error handling across modules",
            "labels": [
                "bug",
                "enhancement",
                "priority:medium",
                "codex",
                "type:feature",
                "module:common",
            ],
            "category": categorize_real_issue(
                1063,
                "Standardize error handling across modules",
                [
                    "bug",
                    "enhancement",
                    "priority:medium",
                    "codex",
                    "type:feature",
                    "module:common",
                ],
            ),
        },
        {
            "number": 1040,
            "title": "Investigate Protovalidate",
            "labels": ["priority:medium", "needs-triage", "needs-info"],
            "category": categorize_real_issue(
                1040,
                "Investigate Protovalidate",
                ["priority:medium", "needs-triage", "needs-info"],
            ),
        },
    ]

    # Real issues from copilot-agent-util-rust
    copilot_util_issues = [
        {
            "number": 1,
            "title": "Support placeholder numbers for parent GUID updates",
            "labels": [
                "bug",
                "enhancement",
                "codex",
                "github-actions",
                "priority:medium",
            ],
            "category": categorize_real_issue(
                1,
                "Support placeholder numbers for parent GUID updates",
                ["bug", "enhancement", "codex", "github-actions", "priority:medium"],
            ),
        }
    ]

    return {
        "subtitle-manager": subtitle_manager_issues,
        "gcommon": gcommon_issues,
        "copilot-agent-util-rust": copilot_util_issues,
    }


def generate_assignment_instructions():
    """Generate specific assignment instructions for GitHub Projects"""

    print("🎯 GITHUB PROJECTS ASSIGNMENT INSTRUCTIONS")
    print("=" * 60)
    print()

    all_issues = analyze_actual_issues()

    # Count issues by category across all repos
    category_counts = {}
    for repo, issues in all_issues.items():
        for issue in issues:
            category = issue["category"]
            if category not in category_counts:
                category_counts[category] = {"count": 0, "issues": []}
            category_counts[category]["count"] += 1
            category_counts[category]["issues"].append(
                f"{repo}#{issue['number']}: {issue['title']}"
            )

    print("📊 ISSUE DISTRIBUTION BY CATEGORY:")
    print("-" * 40)
    for category, data in sorted(category_counts.items()):
        print(f"{category}: {data['count']} issues")
    print()

    print("📋 ASSIGNMENT INSTRUCTIONS:")
    print("-" * 40)
    print()

    for category, data in sorted(category_counts.items()):
        if data["count"] > 0:
            print(f"## {category} Project")
            print(f"**{data['count']} issues to assign:**")
            print()
            for issue_desc in data["issues"]:
                repo_issue = issue_desc.split(":", 1)
                repo_and_number = repo_issue[0]
                title = repo_issue[1].strip()
                print(f"- [ ] {repo_and_number}: {title}")
            print()

    print("🔧 SETUP STEPS:")
    print("-" * 40)
    print("1. Go to your GitHub organization or repository")
    print("2. Navigate to Projects tab")
    print("3. Create projects for each category above (if not already created)")
    print("4. For each issue listed:")
    print("   - Open the issue in GitHub")
    print("   - Add it to the corresponding project")
    print("   - Set appropriate status (Todo, In Progress, Done)")
    print("5. Set up automation rules for future issues based on labels")
    print()

    print("🏷️ AUTOMATION RULES SETUP:")
    print("-" * 40)
    for repo, issues in all_issues.items():
        print(f"\n**{repo}:**")
        labels_by_category = {}
        for issue in issues:
            category = issue["category"]
            if category not in labels_by_category:
                labels_by_category[category] = set()
            labels_by_category[category].update(issue["labels"])

        for category, labels in labels_by_category.items():
            relevant_labels = [
                label
                for label in labels
                if not label.startswith("priority:")
                and label not in ["bug", "enhancement", "codex"]
            ]
            if relevant_labels:
                print(f"  {category}: {', '.join(relevant_labels)}")


def main():
    """Main execution function"""
    generate_assignment_instructions()

    print("\n✅ ASSIGNMENT INSTRUCTIONS COMPLETE")
    print("\nNext steps:")
    print("1. Create the GitHub Projects if they don't exist")
    print("2. Follow the checklist above to assign each issue")
    print("3. Set up the automation rules for future issues")
    print("4. Celebrate having organized issues! 🎉")


if __name__ == "__main__":
    main()
