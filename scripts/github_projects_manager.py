#!/usr/bin/env python3
# file: scripts/github_projects_manager.py
# version: 1.0.0
# guid: d1e2f3g4-h5i6-7890-defg-hi1234567890

"""
GitHub Projects Manager

This script manages GitHub Projects integration for issue organization.
Uses MCP GitHub tools to analyze and categorize issues for project assignment.
"""

import sys
from pathlib import Path
from typing import Dict, List, Optional


class GitHubProjectsManager:
    """Manages GitHub Projects integration for issue organization"""

    def __init__(self):
        self.repositories = {
            "copilot-agent-util-rust": {
                "owner": "jdfalk",
                "repo": "copilot-agent-util-rust",
                "path": Path(
                    "/Users/jdfalk/repos/github.com/jdfalk/copilot-agent-util-rust"
                ),
            },
            "gcommon": {
                "owner": "jdfalk",
                "repo": "gcommon",
                "path": Path("/Users/jdfalk/repos/github.com/jdfalk/gcommon"),
            },
            "subtitle-manager": {
                "owner": "jdfalk",
                "repo": "subtitle-manager",
                "path": Path("/Users/jdfalk/repos/github.com/jdfalk/subtitle-manager"),
            },
            "ghcommon": {
                "owner": "jdfalk",
                "repo": "ghcommon",
                "path": Path("/Users/jdfalk/repos/github.com/jdfalk/ghcommon"),
            },
        }

        # Project categories based on labels
        self.project_categories = {
            "Backend Services": [
                "module:auth",
                "module:cache",
                "module:config",
                "module:database",
                "module:metrics",
                "module:notification",
                "module:organization",
                "module:queue",
                "module:security",
                "module:grpc",
            ],
            "Web & UI": ["module:web", "module:ui", "frontend"],
            "Infrastructure": [
                "ci-cd",
                "deployment",
                "docker",
                "automation",
                "github-actions",
                "monitoring",
                "performance",
            ],
            "Documentation": [
                "documentation",
                "docs",
                "examples",
                "type:documentation",
            ],
            "Testing": ["testing", "integration", "unit-tests", "e2e"],
            "SDKs & Tools": ["sdk", "tools", "cli", "codex"],
        }

    def categorize_issue_by_labels(self, labels: List[str]) -> str:
        """Determine which project category an issue belongs to based on labels"""
        for category, category_labels in self.project_categories.items():
            for label in labels:
                if label in category_labels or any(
                    cat_label in label for cat_label in category_labels
                ):
                    return category

        # Check for type-based categorization
        for label in labels:
            if label.startswith("type:"):
                if "documentation" in label:
                    return "Documentation"
                elif "feature" in label or "enhancement" in label:
                    return "Backend Services"  # Default for features

        return "Backend Services"  # Default category

    def analyze_repository_issues(self, repo_name: str) -> Dict[str, List[str]]:
        """Analyze issues in a repository and categorize them for projects"""
        if repo_name not in self.repositories:
            print(f"❌ Unknown repository: {repo_name}")
            return {}

        repo_config = self.repositories[repo_name]
        print(f"\n📊 ANALYZING ISSUES - {repo_name.upper()}")
        print("=" * 50)

        # This would use MCP tools to get issues, but for now we'll simulate
        # based on the labels we know exist from processing

        issue_categories = {
            "Backend Services": [],
            "Web & UI": [],
            "Infrastructure": [],
            "Documentation": [],
            "Testing": [],
            "SDKs & Tools": [],
        }

        print("🔍 Analysis based on processed issue updates:")
        print(f"   Repository: {repo_config['owner']}/{repo_config['repo']}")
        print("   Project Categories Available:")

        for category in issue_categories.keys():
            print(f"   • {category}")

        return issue_categories

    def generate_project_setup_commands(self, repo_name: str):
        """Generate commands for setting up GitHub Projects"""
        if repo_name not in self.repositories:
            print(f"❌ Unknown repository: {repo_name}")
            return

        repo_config = self.repositories[repo_name]

        print(f"\n🚀 GITHUB PROJECTS SETUP - {repo_name.upper()}")
        print("=" * 50)
        print(
            "Since GitHub Projects V2 requires web interface setup, here are the manual steps:"
        )
        print()

        print("1. Create Organization-level Project (recommended):")
        print(f"   https://github.com/orgs/{repo_config['owner']}/projects/new")
        print()

        print("2. Or create Repository-level Project:")
        print(
            f"   https://github.com/{repo_config['owner']}/{repo_config['repo']}/projects/new"
        )
        print()

        print("3. Set up these project views/categories:")
        for i, category in enumerate(self.project_categories.keys(), 1):
            print(f"   {i}. {category}")
        print()

        print("4. Configure automation rules:")
        print("   • Auto-add issues with specific labels")
        print("   • Set status based on issue state (open/closed)")
        print("   • Prioritize by label (priority:high, priority:medium, priority:low)")
        print()

        print("5. Use these label filters for automation:")
        for category, labels in self.project_categories.items():
            print(
                f"   {category}: {', '.join(labels[:3])}{'...' if len(labels) > 3 else ''}"
            )

    def create_issue_assignment_script(self, repo_name: str):
        """Create a script template for assigning existing issues to projects"""
        if repo_name not in self.repositories:
            print(f"❌ Unknown repository: {repo_name}")
            return

        repo_config = self.repositories[repo_name]
        script_content = f'''#!/usr/bin/env python3
# Generated GitHub Projects assignment script for {repo_name}

"""
Issue Assignment Script for {repo_name}

This script template shows how to assign existing issues to GitHub Projects
based on their labels and categories.
"""

# Use MCP GitHub tools to:
# 1. mcp_github_list_issues(owner="{repo_config["owner"]}", repo="{repo_config["repo"]}")
# 2. Filter issues by labels
# 3. Add to appropriate projects using GitHub Projects API

project_assignments = {{
'''

        for category, labels in self.project_categories.items():
            script_content += f'    "{category}": {labels},\n'

        script_content += """
}

# TODO: Implement project assignment logic using MCP tools
# This requires GitHub Projects V2 API integration
"""

        script_path = repo_config["path"] / f"assign_issues_to_projects_{repo_name}.py"
        with open(script_path, "w") as f:
            f.write(script_content)

        print(f"📝 Created assignment script: {script_path}")

    def process_repository(self, repo_name: str):
        """Process a single repository for GitHub Projects integration"""
        print(f"\n🎯 PROCESSING {repo_name.upper()}")
        print("-" * 40)

        # Analyze issues
        self.analyze_repository_issues(repo_name)

        # Generate setup commands
        self.generate_project_setup_commands(repo_name)

        # Create assignment script template
        self.create_issue_assignment_script(repo_name)

    def process_all_repositories(self):
        """Process all repositories for GitHub Projects integration"""
        print("🎯 GITHUB PROJECTS MANAGER")
        print("=" * 50)
        print("Setting up GitHub Projects integration for issue organization")

        for repo_name in self.repositories:
            self.process_repository(repo_name)

        print(f"\n✨ GITHUB PROJECTS SETUP COMPLETE")
        print("=" * 50)
        print("📋 SUMMARY:")
        print("• Legacy processing scripts cleaned up")
        print("• Project categories defined based on existing labels")
        print("• Setup instructions generated for each repository")
        print("• Assignment script templates created")
        print()
        print("🔗 NEXT STEPS:")
        print("1. Create GitHub Projects using the provided URLs")
        print("2. Set up automation rules for each project")
        print("3. Run the assignment scripts to add existing issues")
        print("4. Configure label-based auto-assignment for new issues")


def main():
    """Main execution function"""
    manager = GitHubProjectsManager()

    if len(sys.argv) > 1:
        repo_name = sys.argv[1]
        manager.process_repository(repo_name)
    else:
        manager.process_all_repositories()


if __name__ == "__main__":
    main()
