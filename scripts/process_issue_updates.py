#!/usr/bin/env python3
# file: scripts/process_issue_updates.py
# version: 1.0.0
# guid: a1b2c3d4-e5f6-7890-abcd-ef1234567890

"""
Comprehensive GitHub Issue Updates Processor

This script processes all issue update JSON files to ensure GitHub issues
are properly created, updated, and managed in the project.
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional, Set
import requests
from dataclasses import dataclass


@dataclass
class IssueUpdate:
    """Represents an issue update request"""

    action: str
    title: str
    body: str
    labels: List[str]
    guid: str
    legacy_guid: Optional[str] = None
    number: Optional[int] = None
    state: Optional[str] = None
    assignees: Optional[List[str]] = None


class GitHubIssueProcessor:
    """Handles GitHub issue operations"""

    def __init__(self, owner: str, repo: str, token: str):
        self.owner = owner
        self.repo = repo
        self.token = token
        self.base_url = "https://api.github.com"
        self.headers = {
            "Authorization": f"token {token}",
            "Accept": "application/vnd.github.v3+json",
            "Content-Type": "application/json",
        }

    def get_all_issues(self) -> List[Dict]:
        """Get all issues from the repository"""
        issues = []
        page = 1
        per_page = 100

        while True:
            url = f"{self.base_url}/repos/{self.owner}/{self.repo}/issues"
            params = {"state": "all", "page": page, "per_page": per_page}

            response = requests.get(url, headers=self.headers, params=params)
            response.raise_for_status()

            page_issues = response.json()
            if not page_issues:
                break

            issues.extend(page_issues)
            page += 1

        return issues

    def create_issue(self, issue_update: IssueUpdate) -> Dict:
        """Create a new issue"""
        url = f"{self.base_url}/repos/{self.owner}/{self.repo}/issues"

        # Add GUID to body if not present
        body = issue_update.body
        if issue_update.guid and f"guid:{issue_update.guid}" not in body:
            body += f"\n\n<!-- guid:{issue_update.guid} -->"

        data = {
            "title": issue_update.title,
            "body": body,
            "labels": issue_update.labels,
        }

        if issue_update.assignees:
            data["assignees"] = issue_update.assignees

        response = requests.post(url, headers=self.headers, json=data)
        response.raise_for_status()

        return response.json()

    def update_issue(self, number: int, issue_update: IssueUpdate) -> Dict:
        """Update an existing issue"""
        url = f"{self.base_url}/repos/{self.owner}/{self.repo}/issues/{number}"

        data = {}
        if issue_update.title:
            data["title"] = issue_update.title
        if issue_update.body:
            data["body"] = issue_update.body
        if issue_update.labels:
            data["labels"] = issue_update.labels
        if issue_update.state:
            data["state"] = issue_update.state
        if issue_update.assignees:
            data["assignees"] = issue_update.assignees

        response = requests.patch(url, headers=self.headers, json=data)
        response.raise_for_status()

        return response.json()

    def find_issue_by_guid(self, issues: List[Dict], guid: str) -> Optional[Dict]:
        """Find an issue by GUID in the body"""
        for issue in issues:
            if issue.get("body") and f"guid:{guid}" in issue["body"]:
                return issue
        return None


def load_issue_update(file_path: Path) -> IssueUpdate:
    """Load an issue update from JSON file"""
    with open(file_path, "r") as f:
        data = json.load(f)

    return IssueUpdate(
        action=data.get("action", "create"),
        title=data.get("title", ""),
        body=data.get("body", ""),
        labels=data.get("labels", []),
        guid=data.get("guid", ""),
        legacy_guid=data.get("legacy_guid"),
        number=data.get("number"),
        state=data.get("state"),
        assignees=data.get("assignees"),
    )


def main():
    """Main processing function"""
    # Configuration
    OWNER = "jdfalk"
    REPO = "copilot-agent-util-rust"
    TOKEN = os.environ.get("GITHUB_TOKEN")

    if not TOKEN:
        print("Error: GITHUB_TOKEN environment variable not set")
        sys.exit(1)

    # Initialize processor
    processor = GitHubIssueProcessor(OWNER, REPO, TOKEN)

    # Get all existing issues
    print("Fetching all existing issues...")
    existing_issues = processor.get_all_issues()
    print(f"Found {len(existing_issues)} existing issues")

    # Process issue update directories
    base_dir = Path(
        "/Users/jdfalk/repos/github.com/jdfalk/copilot-agent-util-rust/.github/issue-updates"
    )

    # Track processed GUIDs
    processed_guids: Set[str] = set()
    created_issues: List[Dict] = []
    updated_issues: List[Dict] = []
    errors: List[str] = []

    # Process unprocessed files first
    unprocessed_files = [
        f for f in base_dir.glob("*.json") if f.name != "workflow-trigger-test.json"
    ]

    print(f"\nProcessing {len(unprocessed_files)} unprocessed issue updates...")

    for file_path in unprocessed_files:
        try:
            print(f"\nProcessing: {file_path.name}")
            issue_update = load_issue_update(file_path)

            if not issue_update.guid:
                print("  ⚠️  No GUID found, skipping")
                continue

            if issue_update.guid in processed_guids:
                print("  ⚠️  GUID already processed, skipping")
                continue

            # Check if issue already exists
            existing_issue = processor.find_issue_by_guid(
                existing_issues, issue_update.guid
            )

            if existing_issue:
                print(
                    f"  ✅ Issue already exists: #{existing_issue['number']} - {existing_issue['title']}"
                )
                processed_guids.add(issue_update.guid)

                # Move to processed folder
                processed_path = base_dir / "processed" / file_path.name
                file_path.rename(processed_path)
                print("  📁 Moved to processed folder")
                continue

            # Create new issue
            if issue_update.action == "create":
                print(f"  🔄 Creating issue: {issue_update.title}")
                created_issue = processor.create_issue(issue_update)
                created_issues.append(created_issue)
                processed_guids.add(issue_update.guid)

                print(
                    f"  ✅ Created issue #{created_issue['number']}: {created_issue['title']}"
                )

                # Move to processed folder
                processed_path = base_dir / "processed" / file_path.name
                file_path.rename(processed_path)
                print("  📁 Moved to processed folder")
            else:
                print(f"  ⚠️  Unknown action: {issue_update.action}")

        except Exception as e:
            error_msg = f"Error processing {file_path.name}: {str(e)}"
            print(f"  ❌ {error_msg}")
            errors.append(error_msg)

    # Check processed folder for validation
    processed_dir = base_dir / "processed"
    processed_files = list(processed_dir.glob("*.json"))

    print(f"\nValidating {len(processed_files)} processed files...")

    missing_issues = []
    for file_path in processed_files:
        try:
            if file_path.name.startswith("workflow-trigger-test"):
                continue

            issue_update = load_issue_update(file_path)

            if not issue_update.guid:
                continue

            existing_issue = processor.find_issue_by_guid(
                existing_issues, issue_update.guid
            )

            if not existing_issue:
                print(f"  ⚠️  Missing issue for processed file: {file_path.name}")
                missing_issues.append(file_path.name)
            else:
                print(
                    f"  ✅ Validated: #{existing_issue['number']} - {existing_issue['title']}"
                )

        except Exception as e:
            error_msg = f"Error validating {file_path.name}: {str(e)}"
            print(f"  ❌ {error_msg}")
            errors.append(error_msg)

    # Summary
    print(f"\n{'=' * 60}")
    print("PROCESSING SUMMARY")
    print(f"{'=' * 60}")
    print(f"✅ Issues created: {len(created_issues)}")
    print(f"🔄 Issues updated: {len(updated_issues)}")
    print(f"⚠️  Missing issues: {len(missing_issues)}")
    print(f"❌ Errors: {len(errors)}")

    if created_issues:
        print("\n📋 Created Issues:")
        for issue in created_issues:
            print(f"  #{issue['number']}: {issue['title']}")

    if missing_issues:
        print("\n⚠️  Missing Issues for Processed Files:")
        for filename in missing_issues:
            print(f"  {filename}")

    if errors:
        print("\n❌ Errors:")
        for error in errors:
            print(f"  {error}")

    # Recommendations
    print("\n💡 RECOMMENDATIONS:")
    if missing_issues:
        print("- Review missing issues and consider recreating them")
    print("- Check project board assignments for new issues")
    print("- Verify issue labels are correctly applied")
    print("- Consider adding priority labels to new issues")


if __name__ == "__main__":
    main()
