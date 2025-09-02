#!/usr/bin/env python3
# file: analyze_issues_for_projects.py
# version: 1.0.0
# guid: 9a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d

"""
Analyze issues across repositories to generate project assignment recommendations.
Uses the processed issue data from earlier operations.
"""

import os
import glob
from typing import Dict, List, Set
from dataclasses import dataclass


@dataclass
class IssueCategory:
    """Categories for project assignment"""

    name: str
    keywords: Set[str]
    description: str


# Define project categories
CATEGORIES = [
    IssueCategory(
        "Backend Services",
        {
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
            "protobuf",
            "buf",
        },
        "Authentication, APIs, databases, gRPC services, metrics",
    ),
    IssueCategory(
        "Web & UI",
        {
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
            "dashboard",
        },
        "Web interfaces, frontend components, UI/UX",
    ),
    IssueCategory(
        "Infrastructure",
        {
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
            "automation",
            "cosign",
        },
        "CI/CD, deployments, build systems, Docker, automation",
    ),
    IssueCategory(
        "Documentation",
        {
            "documentation",
            "docs",
            "examples",
            "readme",
            "tutorial",
            "guide",
            "help",
            "wiki",
            "manual",
            "changelog",
        },
        "Documentation, examples, guides, READMEs",
    ),
    IssueCategory(
        "Testing",
        {
            "testing",
            "integration",
            "unit-tests",
            "test",
            "qa",
            "validation",
            "e2e",
            "regression",
            "benchmark",
        },
        "Unit tests, integration tests, QA, validation",
    ),
    IssueCategory(
        "SDKs & Tools",
        {
            "sdk",
            "tools",
            "cli",
            "library",
            "utility",
            "script",
            "generator",
            "parser",
            "copilot-agent-util",
            "protoc",
        },
        "CLI tools, SDKs, utilities, development tools",
    ),
]


def categorize_by_labels_and_content(
    labels: List[str], title: str, body: str = ""
) -> str:
    """Categorize an issue based on labels and content"""
    text_content = f"{title} {body}".lower()
    label_set = {label.lower() for label in labels}

    # Score each category
    category_scores = {}
    for category in CATEGORIES:
        score = 0
        # Label matches (higher weight)
        score += len(label_set.intersection(category.keywords)) * 3
        # Text content matches (lower weight)
        score += sum(1 for keyword in category.keywords if keyword in text_content)
        category_scores[category.name] = score

    # Return highest scoring category
    if max(category_scores.values()) > 0:
        return max(category_scores, key=category_scores.get)
    return "General"


def analyze_repository_structure():
    """Analyze the repository structure and issue distribution"""
    print("🔍 REPOSITORY ANALYSIS")
    print("=" * 50)

    repositories = [
        "copilot-agent-util-rust",
        "subtitle-manager",
        "gcommon",
        "ghcommon",
    ]

    total_analysis = {category.name: [] for category in CATEGORIES}
    total_analysis["General"] = []

    for repo in repositories:
        print(f"\n📊 Analyzing {repo}")
        print("-" * 30)

        # Check for processed issue files
        repo_path = f"/Users/jdfalk/repos/github.com/jdfalk/{repo}"
        if not os.path.exists(repo_path):
            print(f"⚠️  Repository path not found: {repo_path}")
            continue

        # Look for issue-related files
        issue_files = glob.glob(f"{repo_path}/*issue*") + glob.glob(
            f"{repo_path}/.github/*issue*"
        )

        if issue_files:
            print(f"Found issue files: {len(issue_files)}")
            for file in issue_files:
                print(f"  - {os.path.basename(file)}")

        # Check labels file for category hints
        labels_file = f"{repo_path}/labels.json"
        if os.path.exists(labels_file):
            print("✅ Found labels file: labels.json")
            try:
                with open(labels_file, "r") as f:
                    content = f.read()
                    # Count label types
                    for category in CATEGORIES:
                        count = sum(
                            1
                            for keyword in category.keywords
                            if keyword in content.lower()
                        )
                        if count > 0:
                            print(f"  {category.name}: ~{count} relevant labels")
            except Exception as e:
                print(f"  Error reading labels: {e}")

        # Sample issue categorization (based on common patterns)
        sample_issues = get_sample_issues_for_repo(repo)
        repo_categories = {category.name: 0 for category in CATEGORIES}
        repo_categories["General"] = 0

        for issue in sample_issues:
            category = categorize_by_labels_and_content(
                issue.get("labels", []), issue.get("title", ""), issue.get("body", "")
            )
            repo_categories[category] += 1

        print("\nSample issue distribution:")
        for cat, count in repo_categories.items():
            if count > 0:
                print(f"  {cat}: {count} issues")


def get_sample_issues_for_repo(repo: str) -> List[Dict]:
    """Get sample issues based on repository type"""
    # Define sample issues based on what we know about each repo
    samples = {
        "copilot-agent-util-rust": [
            {
                "title": "Support placeholder numbers for parent GUID updates",
                "labels": ["bug", "enhancement", "github-actions"],
            },
            {
                "title": "Add cross-platform build support",
                "labels": ["enhancement", "ci-cd"],
            },
            {
                "title": "Improve CLI argument parsing",
                "labels": ["tools", "enhancement"],
            },
        ],
        "subtitle-manager": [
            {
                "title": "Add gRPC health service",
                "labels": ["enhancement", "module:api"],
            },
            {
                "title": "Finish metrics proto migration",
                "labels": ["enhancement", "module:metrics", "refactor"],
            },
            {
                "title": "Implement gcommon logrus provider",
                "labels": ["enhancement", "module:config"],
            },
            {
                "title": "Add test for /api/sync/batch invalid JSON",
                "labels": ["test", "module:web"],
            },
            {"title": "Fix dependabot labels", "labels": ["bug", "module:config"]},
        ],
        "gcommon": [
            {
                "title": "Protocol buffer generation workflow",
                "labels": ["protobuf", "ci-cd"],
            },
            {
                "title": "Add protovalidate support",
                "labels": ["protobuf", "validation"],
            },
            {"title": "SDK documentation updates", "labels": ["documentation", "sdk"]},
        ],
        "ghcommon": [
            {
                "title": "Workflow template updates",
                "labels": ["github-actions", "ci-cd"],
            },
            {"title": "Repository sync automation", "labels": ["automation", "tools"]},
            {
                "title": "Documentation system improvements",
                "labels": ["documentation", "tools"],
            },
        ],
    }
    return samples.get(repo, [])


def generate_project_recommendations():
    """Generate recommendations for GitHub Projects setup"""
    print("\n🎯 PROJECT RECOMMENDATIONS")
    print("=" * 50)

    print("\nBased on the analysis, here are the recommended GitHub Projects:")
    print()

    for i, category in enumerate(CATEGORIES, 1):
        print(f"{i}. **{category.name}**")
        print(f"   Description: {category.description}")
        print(f"   Keywords: {', '.join(sorted(list(category.keywords)[:10]))}")
        print()

    print("7. **General**")
    print("   Description: Issues that don't fit into specific categories")
    print()

    print("📋 SETUP INSTRUCTIONS:")
    print("1. Create these projects in your GitHub organization or repositories")
    print("2. Set up automation rules based on issue labels")
    print("3. Use the keywords above to configure auto-assignment")
    print("4. Review and adjust categories based on your specific needs")


def main():
    """Main execution function"""
    print("🎯 GITHUB PROJECTS ANALYSIS & RECOMMENDATIONS")
    print("=" * 60)

    analyze_repository_structure()
    generate_project_recommendations()

    print("\n✅ ANALYSIS COMPLETE")
    print("\nNext steps:")
    print("1. Review the category recommendations above")
    print("2. Create projects using the GitHub web interface")
    print("3. Set up automation rules for each project")
    print("4. Manually assign existing issues to appropriate projects")


if __name__ == "__main__":
    main()
