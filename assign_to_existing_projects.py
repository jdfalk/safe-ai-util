#!/usr/bin/env python3
# file: assign_to_existing_projects.py
# version: 1.0.0
# guid: 8f3e4a2b-7d6c-1e4f-9a8b-3c5d7e9f1a2b

"""
Direct assignment of issues to existing GitHub Projects.
Assumes projects already exist and focuses on categorizing and assigning issues.
"""

import requests
import os
from typing import Dict, List, Set
from dataclasses import dataclass


@dataclass
class ProjectMapping:
    """Maps issue characteristics to project categories"""

    backend_keywords: Set[str]
    web_ui_keywords: Set[str]
    infrastructure_keywords: Set[str]
    documentation_keywords: Set[str]
    testing_keywords: Set[str]
    tools_keywords: Set[str]


# Define project categorization based on labels
PROJECT_MAPPING = ProjectMapping(
    backend_keywords={
        "module:auth",
        "module:cache",
        "module:config",
        "module:database",
        "module:api",
        "module:security",
        "backend",
        "server",
        "grpc",
        "module:metrics",
        "module:queue",
        "module:organization",
    },
    web_ui_keywords={
        "module:web",
        "module:ui",
        "frontend",
        "web",
        "ui",
        "html",
        "css",
        "javascript",
        "react",
        "vue",
        "angular",
        "webui",
    },
    infrastructure_keywords={
        "ci-cd",
        "deployment",
        "docker",
        "kubernetes",
        "infrastructure",
        "devops",
        "build",
        "release",
        "github-actions",
        "workflow",
    },
    documentation_keywords={
        "documentation",
        "docs",
        "examples",
        "readme",
        "tutorial",
        "guide",
        "help",
        "wiki",
        "manual",
    },
    testing_keywords={
        "testing",
        "integration",
        "unit-tests",
        "test",
        "qa",
        "validation",
        "e2e",
        "regression",
    },
    tools_keywords={
        "sdk",
        "tools",
        "cli",
        "library",
        "utility",
        "script",
        "automation",
        "generator",
        "parser",
    },
)


def categorize_issue(labels: List[str], title: str, body: str) -> str:
    """Categorize issue based on labels, title, and description"""
    label_set = {label.lower() for label in labels}
    text_content = f"{title} {body}".lower()

    # Check each category
    if label_set.intersection(PROJECT_MAPPING.backend_keywords) or any(
        kw in text_content for kw in PROJECT_MAPPING.backend_keywords
    ):
        return "Backend Services"
    elif label_set.intersection(PROJECT_MAPPING.web_ui_keywords) or any(
        kw in text_content for kw in PROJECT_MAPPING.web_ui_keywords
    ):
        return "Web & UI"
    elif label_set.intersection(PROJECT_MAPPING.infrastructure_keywords) or any(
        kw in text_content for kw in PROJECT_MAPPING.infrastructure_keywords
    ):
        return "Infrastructure"
    elif label_set.intersection(PROJECT_MAPPING.documentation_keywords) or any(
        kw in text_content for kw in PROJECT_MAPPING.documentation_keywords
    ):
        return "Documentation"
    elif label_set.intersection(PROJECT_MAPPING.testing_keywords) or any(
        kw in text_content for kw in PROJECT_MAPPING.testing_keywords
    ):
        return "Testing"
    elif label_set.intersection(PROJECT_MAPPING.tools_keywords) or any(
        kw in text_content for kw in PROJECT_MAPPING.tools_keywords
    ):
        return "SDKs & Tools"
    else:
        return "General"


def analyze_repository_issues(owner: str, repo: str, github_token: str) -> Dict:
    """Analyze all issues in a repository and categorize them"""
    headers = {
        "Authorization": f"token {github_token}",
        "Accept": "application/vnd.github.v3+json",
    }

    issues_url = f"https://api.github.com/repos/{owner}/{repo}/issues"

    categories = {
        "Backend Services": [],
        "Web & UI": [],
        "Infrastructure": [],
        "Documentation": [],
        "Testing": [],
        "SDKs & Tools": [],
        "General": [],
    }

    page = 1
    while True:
        response = requests.get(
            f"{issues_url}?page={page}&per_page=100&state=all", headers=headers
        )
        if response.status_code != 200:
            print(f"❌ Error fetching issues: {response.status_code}")
            break

        issues = response.json()
        if not issues:
            break

        for issue in issues:
            if "pull_request" in issue:  # Skip pull requests
                continue

            labels = [label["name"] for label in issue.get("labels", [])]
            category = categorize_issue(labels, issue["title"], issue.get("body", ""))

            categories[category].append(
                {
                    "number": issue["number"],
                    "title": issue["title"],
                    "labels": labels,
                    "state": issue["state"],
                    "url": issue["html_url"],
                }
            )

        page += 1
        if len(issues) < 100:  # Last page
            break

    return categories


def generate_assignment_commands(owner: str, repo: str, categories: Dict) -> List[str]:
    """Generate commands for assigning issues to projects"""
    commands = []

    for category, issues in categories.items():
        if not issues:
            continue

        commands.append(f"\n# === {category} ({len(issues)} issues) ===")
        for issue in issues[:5]:  # Limit to first 5 for readability
            commands.append(f"# Issue #{issue['number']}: {issue['title']}")
            commands.append(f"# Labels: {', '.join(issue['labels'])}")
            commands.append(f"# URL: {issue['url']}")
            commands.append(f"# Assign to: {category} project")
            commands.append("")

    return commands


def main():
    """Main execution function"""
    print("🎯 GITHUB PROJECTS ISSUE ASSIGNMENT")
    print("=" * 50)

    # Get GitHub token
    github_token = os.getenv("GITHUB_TOKEN") or os.getenv("JF_CI_GH_PAT")
    if not github_token:
        print(
            "❌ No GitHub token found. Set GITHUB_TOKEN or JF_CI_GH_PAT environment variable."
        )
        return

    repositories = [
        ("jdfalk", "copilot-agent-util-rust"),
        ("jdfalk", "subtitle-manager"),
        ("jdfalk", "gcommon"),
        ("jdfalk", "ghcommon"),
    ]

    for owner, repo in repositories:
        print(f"\n📊 ANALYZING {repo.upper()}")
        print("-" * 40)

        categories = analyze_repository_issues(owner, repo, github_token)

        # Print summary
        total_issues = sum(len(issues) for issues in categories.values())
        print(f"Total issues found: {total_issues}")

        for category, issues in categories.items():
            if issues:
                print(f"  {category}: {len(issues)} issues")

        # Generate assignment file
        assignment_file = f"project_assignments_{repo}.txt"
        commands = generate_assignment_commands(owner, repo, categories)

        with open(assignment_file, "w") as f:
            f.write(f"# Project Assignments for {repo}\n")
            f.write("# Generated by assign_to_existing_projects.py\n")
            f.write("=" * 60 + "\n\n")
            f.write("\n".join(commands))

        print(f"📝 Assignment file created: {assignment_file}")

    print("\n✅ ANALYSIS COMPLETE")
    print("Next steps:")
    print("1. Review the generated assignment files")
    print("2. Use GitHub Projects web interface to assign issues")
    print("3. Set up automation rules for future issues")


if __name__ == "__main__":
    main()
