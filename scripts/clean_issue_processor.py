#!/usr/bin/env python3
# file: scripts/clean_issue_processor.py
# version: 1.0.0
# guid: c1d2e3f4-g5h6-7890-cdef-gh1234567890

"""
Clean Issue Processor with GitHub Projects Integration

This script consolidates all issue processing functionality and adds support
for GitHub Projects management to ensure issues are properly organized.
"""

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Set
import requests
from dataclasses import dataclass


@dataclass
class RepositoryConfig:
    """Configuration for a repository"""

    name: str
    owner: str
    repo: str
    path: Path
    project_id: Optional[str] = None


class CleanIssueProcessor:
    """Consolidated issue processor with project management"""

    def __init__(self, github_token: Optional[str] = None):
        self.github_token = github_token or os.getenv("GITHUB_TOKEN")
        self.base_url = "https://api.github.com"
        self.headers = (
            {
                "Authorization": f"token {self.github_token}",
                "Accept": "application/vnd.github.v3+json",
                "Content-Type": "application/json",
            }
            if self.github_token
            else {}
        )

        # Repository configurations
        self.repositories = {
            "copilot-agent-util-rust": RepositoryConfig(
                name="copilot-agent-util-rust",
                owner="jdfalk",
                repo="copilot-agent-util-rust",
                path=Path(
                    "/Users/jdfalk/repos/github.com/jdfalk/copilot-agent-util-rust"
                ),
            ),
            "gcommon": RepositoryConfig(
                name="gcommon",
                owner="jdfalk",
                repo="gcommon",
                path=Path("/Users/jdfalk/repos/github.com/jdfalk/gcommon"),
            ),
            "subtitle-manager": RepositoryConfig(
                name="subtitle-manager",
                owner="jdfalk",
                repo="subtitle-manager",
                path=Path("/Users/jdfalk/repos/github.com/jdfalk/subtitle-manager"),
            ),
            "ghcommon": RepositoryConfig(
                name="ghcommon",
                owner="jdfalk",
                repo="ghcommon",
                path=Path("/Users/jdfalk/repos/github.com/jdfalk/ghcommon"),
            ),
        }

    def get_repository_issues(self, repo_config: RepositoryConfig) -> List[Dict]:
        """Get all issues for a repository"""
        if not self.github_token:
            print("⚠️  No GitHub token available")
            return []

        url = f"{self.base_url}/repos/{repo_config.owner}/{repo_config.repo}/issues"
        params = {
            "state": "all",
            "per_page": 100,
            "sort": "created",
            "direction": "desc",
        }

        try:
            response = requests.get(url, headers=self.headers, params=params)
            response.raise_for_status()
            return response.json()
        except Exception as e:
            print(f"❌ Error fetching issues for {repo_config.name}: {e}")
            return []

    def categorize_issues_by_labels(self, issues: List[Dict]) -> Dict[str, List[Dict]]:
        """Categorize issues by their labels for project assignment"""
        categories = {
            "module:auth": [],
            "module:cache": [],
            "module:config": [],
            "module:database": [],
            "module:metrics": [],
            "module:notification": [],
            "module:organization": [],
            "module:queue": [],
            "module:security": [],
            "module:web": [],
            "type:enhancement": [],
            "type:feature": [],
            "type:bug": [],
            "type:documentation": [],
            "ci-cd": [],
            "testing": [],
            "automation": [],
            "uncategorized": [],
        }

        for issue in issues:
            issue_labels = [label["name"] for label in issue.get("labels", [])]
            categorized = False

            # Check for module-specific labels first
            for label in issue_labels:
                if label in categories:
                    categories[label].append(issue)
                    categorized = True
                    break

            # Check for type-specific labels
            if not categorized:
                for label in issue_labels:
                    if label.startswith("type:") and label in categories:
                        categories[label].append(issue)
                        categorized = True
                        break

            # Check for other category labels
            if not categorized:
                for category in ["ci-cd", "testing", "automation"]:
                    if category in issue_labels or any(
                        category in label for label in issue_labels
                    ):
                        categories[category].append(issue)
                        categorized = True
                        break

            # Default to uncategorized
            if not categorized:
                categories["uncategorized"].append(issue)

        return categories

    def clean_legacy_scripts(self, repo_config: RepositoryConfig):
        """Clean up legacy processing scripts"""
        scripts_to_remove = [
            "scripts/analyze_issue_updates.py",
            "scripts/github_issue_processor.py",
            "scripts/issue_processing_summary.py",
            "scripts/process_all_repositories.py",
            "automated_issue_processor.py",
            "batch_process_gcommon.py",
            "process_gcommon_issues.py",
            "universal_issue_processor.py",
        ]

        removed_count = 0
        for script in scripts_to_remove:
            script_path = repo_config.path / script
            if script_path.exists():
                try:
                    script_path.unlink()
                    print(f"🗑️  Removed: {script}")
                    removed_count += 1
                except Exception as e:
                    print(f"⚠️  Could not remove {script}: {e}")

        if removed_count > 0:
            print(
                f"✅ Cleaned up {removed_count} legacy scripts from {repo_config.name}"
            )
        else:
            print(f"ℹ️  No legacy scripts found in {repo_config.name}")

    def generate_project_assignments_report(self, repo_config: RepositoryConfig):
        """Generate a report of how issues should be assigned to projects"""
        print(f"\n📊 PROJECT ASSIGNMENTS REPORT - {repo_config.name.upper()}")
        print("=" * 60)

        issues = self.get_repository_issues(repo_config)
        if not issues:
            print("No issues found or unable to fetch issues")
            return

        categorized = self.categorize_issues_by_labels(issues)

        for category, issue_list in categorized.items():
            if issue_list:
                print(f"\n🏷️  {category.upper()} ({len(issue_list)} issues):")
                for issue in issue_list[:5]:  # Show first 5 issues
                    state_icon = "✅" if issue["state"] == "closed" else "🔵"
                    print(
                        f"   {state_icon} #{issue['number']}: {issue['title'][:50]}..."
                    )

                if len(issue_list) > 5:
                    print(f"   ... and {len(issue_list) - 5} more issues")

    def process_repository(self, repo_name: str):
        """Process a single repository"""
        if repo_name not in self.repositories:
            print(f"❌ Unknown repository: {repo_name}")
            return

        repo_config = self.repositories[repo_name]
        print(f"\n🚀 Processing {repo_config.name}")
        print("-" * 40)

        # Clean legacy scripts
        self.clean_legacy_scripts(repo_config)

        # Generate project assignments report
        self.generate_project_assignments_report(repo_config)

    def process_all_repositories(self):
        """Process all repositories"""
        print("🔧 CLEAN ISSUE PROCESSOR")
        print("=" * 50)
        print("Cleaning up legacy scripts and generating project assignment reports")

        for repo_name in self.repositories:
            self.process_repository(repo_name)

        print(f"\n✨ CLEANUP AND ANALYSIS COMPLETE")
        print("=" * 50)
        print("📋 NEXT STEPS:")
        print("1. Review the project assignment reports above")
        print("2. Create GitHub Projects if not already existing")
        print("3. Use GitHub Projects API to assign issues to appropriate projects")
        print("4. Set up automation rules for future issue assignments")


def main():
    """Main execution function"""
    if len(sys.argv) > 1:
        repo_name = sys.argv[1]
        processor = CleanIssueProcessor()
        processor.process_repository(repo_name)
    else:
        processor = CleanIssueProcessor()
        processor.process_all_repositories()


if __name__ == "__main__":
    main()
