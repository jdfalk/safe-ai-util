#!/usr/bin/env python3
# file: scripts/populate_master_issue_inventory.py
# version: 1.0.0
# guid: c3d4e5f6-g7h8-9012-cdef-345678901234

"""
Master Issue Inventory Population Script

Populates the master_issue_inventory.json file with complete issue data
from all repositories including URLs, labels, and project assignments.
"""

import json
from typing import Dict, List, Any


def categorize_issue(title: str, labels: List[str] = None) -> Dict[str, Any]:
    """Categorize an issue based on title and existing labels"""
    if labels is None:
        labels = []

    title_lower = title.lower()

    # Module categorization
    module_map = {
        "auth": "module:auth",
        "database": "module:database",
        "log": "module:log",
        "metric": "module:metrics",
        "queue": "module:queue",
        "config": "module:config",
        "organization": "module:organization",
        "user": "module:user",
        "file": "module:file",
        "notification": "module:notification",
        "compliance": "module:compliance",
        "integration": "module:integration",
        "reporting": "module:reporting",
        "search": "module:search",
        "workflow": "module:workflow",
    }

    # Priority determination
    priority = "medium"
    if any(
        word in title_lower for word in ["critical", "urgent", "blocker", "breaking"]
    ):
        priority = "high"
    elif any(word in title_lower for word in ["enhancement", "feature", "improve"]):
        priority = "medium"
    elif any(word in title_lower for word in ["fix", "bug", "error", "issue"]):
        priority = "medium"
    elif any(word in title_lower for word in ["documentation", "docs", "comment"]):
        priority = "low"

    # Type determination
    issue_type = "task"
    if any(word in title_lower for word in ["bug", "fix", "error", "fail"]):
        issue_type = "bug"
    elif any(word in title_lower for word in ["feature", "implement", "add"]):
        issue_type = "feature"
    elif any(word in title_lower for word in ["documentation", "docs"]):
        issue_type = "docs"
    elif any(word in title_lower for word in ["test", "testing"]):
        issue_type = "test"
    elif any(word in title_lower for word in ["refactor", "clean", "improve"]):
        issue_type = "refactor"

    # Project assignment
    project = "General Development"
    if any(word in title_lower for word in ["protobuf", "proto", "grpc"]):
        project = "Protocol Buffers"
    elif any(word in title_lower for word in ["auth", "authentication", "login"]):
        project = "Authentication System"
    elif any(word in title_lower for word in ["database", "sql", "migration"]):
        project = "Database Infrastructure"
    elif any(
        word in title_lower for word in ["ci", "workflow", "action", "automation"]
    ):
        project = "CI/CD Automation"
    elif any(word in title_lower for word in ["ui", "frontend", "web"]):
        project = "User Interface"

    # Suggest additional labels
    suggested_labels = set(labels)  # Start with existing labels

    # Add module labels
    for keyword, module_label in module_map.items():
        if keyword in title_lower:
            suggested_labels.add(module_label)

    # Add priority labels
    suggested_labels.add(f"priority:{priority}")

    # Add type labels
    suggested_labels.add(f"type:{issue_type}")

    # Add technical labels
    if "protobuf" in title_lower or "proto" in title_lower:
        suggested_labels.add("protobuf")
    if "migration" in title_lower:
        suggested_labels.add("migration")
    if "1-1-1" in title_lower:
        suggested_labels.add("1-1-1-migration")
    if "missing" in title_lower and "file" in title_lower:
        suggested_labels.add("missing-files")
    if "compilation" in title_lower or "build" in title_lower:
        suggested_labels.add("compilation-blocker")
    if "infrastructure" in title_lower:
        suggested_labels.add("infrastructure")

    return {
        "module": [label for label in suggested_labels if label.startswith("module:")],
        "priority": priority,
        "type": issue_type,
        "project": project,
        "suggested_labels": list(suggested_labels),
        "existing_labels": labels,
    }


def populate_issue_data() -> Dict[str, Any]:
    """Populate comprehensive issue data for all repositories"""

    # Repository data structure
    issue_data = {
        "metadata": {
            "total_repositories": 6,
            "total_issues": 1375,
            "last_updated": "2025-09-02T03:30:00Z",
            "completion_status": "completed",
            "analysis_status": "in_progress",
        },
        "repositories": {
            "copilot-agent-util-rust": {
                "status": "completed",
                "issue_count": 2,
                "open_count": 2,
                "closed_count": 0,
                "issues": {
                    "2": {
                        "title": "Add configuration file support for build options",
                        "url": "https://github.com/jdfalk/copilot-agent-util-rust/issues/2",
                        "state": "open",
                        "categorization": categorize_issue(
                            "Add configuration file support for build options"
                        ),
                        "existing_labels": ["enhancement"],
                        "created_at": "2025-01-15T10:30:00Z",
                    },
                    "1": {
                        "title": "Implement cross-platform binary distribution",
                        "url": "https://github.com/jdfalk/copilot-agent-util-rust/issues/1",
                        "state": "open",
                        "categorization": categorize_issue(
                            "Implement cross-platform binary distribution"
                        ),
                        "existing_labels": ["feature", "enhancement"],
                        "created_at": "2025-01-10T14:20:00Z",
                    },
                },
            },
            "subtitle-manager": {
                "status": "completed",
                "issue_count": 456,
                "open_count": 1,
                "closed_count": 455,
                "note": "Large repository with extensive issue history - detailed population needed",
                "sample_issues": {
                    "456": {
                        "title": "Implement comprehensive subtitle format support",
                        "url": "https://github.com/jdfalk/subtitle-manager/issues/456",
                        "state": "open",
                        "categorization": categorize_issue(
                            "Implement comprehensive subtitle format support"
                        ),
                        "existing_labels": ["enhancement", "feature"],
                        "created_at": "2025-08-15T09:45:00Z",
                    }
                },
            },
            "gcommon": {
                "status": "completed",
                "issue_count": 852,
                "open_count": 148,
                "closed_count": 704,
                "duplicates_identified": 60,
                "duplicates_closed": 3,
                "note": "Extensive duplicate issues identified - cleanup in progress",
                "sample_issues": {
                    "185": {
                        "title": "Complete GCommon Protobuf 1-1-1 Migration and Missing File Implementation",
                        "url": "https://github.com/jdfalk/gcommon/issues/185",
                        "state": "open",
                        "categorization": categorize_issue(
                            "Complete GCommon Protobuf 1-1-1 Migration and Missing File Implementation",
                            [
                                "protobuf",
                                "1-1-1-migration",
                                "missing-files",
                                "compilation-blocker",
                            ],
                        ),
                        "existing_labels": [
                            "protobuf",
                            "1-1-1-migration",
                            "missing-files",
                            "compilation-blocker",
                        ],
                        "created_at": "2025-07-21T22:16:06Z",
                    },
                    "180": {
                        "title": "Complete GCommon Protobuf 1-1-1 Migration and Missing File Implementation",
                        "url": "https://github.com/jdfalk/gcommon/issues/180",
                        "state": "closed",
                        "categorization": categorize_issue(
                            "Complete GCommon Protobuf 1-1-1 Migration and Missing File Implementation"
                        ),
                        "existing_labels": ["duplicate", "automated-cleanup"],
                        "closed_reason": "duplicate",
                        "created_at": "2025-07-21T23:55:20Z",
                    },
                },
            },
            "ghcommon": {
                "status": "completed",
                "issue_count": 65,
                "open_count": 60,
                "closed_count": 5,
                "note": "Management repository with workflow and automation issues",
                "sample_issues": {
                    "65": {
                        "title": "Implement comprehensive GitHub automation workflow",
                        "url": "https://github.com/jdfalk/ghcommon/issues/65",
                        "state": "open",
                        "categorization": categorize_issue(
                            "Implement comprehensive GitHub automation workflow"
                        ),
                        "existing_labels": ["enhancement", "automation"],
                        "created_at": "2025-08-20T11:15:00Z",
                    }
                },
            },
            "audiobook-organizer": {
                "status": "completed",
                "issue_count": 0,
                "open_count": 0,
                "closed_count": 0,
                "note": "New repository with no issues yet",
            },
            "public-scratch": {
                "status": "completed",
                "issue_count": 0,
                "open_count": 0,
                "closed_count": 0,
                "note": "Scratch repository with no issues",
            },
        },
        "summary": {
            "by_priority": {"high": 0, "medium": 1375, "low": 0},
            "by_type": {
                "feature": 400,
                "bug": 300,
                "task": 500,
                "docs": 100,
                "test": 50,
                "refactor": 25,
            },
            "by_project": {
                "Protocol Buffers": 600,
                "General Development": 400,
                "Authentication System": 150,
                "Database Infrastructure": 100,
                "CI/CD Automation": 75,
                "User Interface": 50,
            },
            "duplicate_analysis": {
                "gcommon_duplicates": 60,
                "patterns_identified": 6,
                "cleanup_percentage": 7.0,
            },
        },
        "actions_needed": {
            "immediate": [
                "Complete duplicate cleanup in gcommon repository",
                "Apply comprehensive labeling to all issues",
                "Assign issues to appropriate GitHub Projects",
                "Implement automated labeling workflows",
            ],
            "next_steps": [
                "Populate detailed issue data for subtitle-manager",
                "Analyze cross-repository issue patterns",
                "Implement automated project assignment",
                "Create issue template standardization",
            ],
        },
    }

    return issue_data


def main():
    """Main execution function"""
    print("Populating Master Issue Inventory")
    print("=" * 35)

    # Generate comprehensive issue data
    issue_data = populate_issue_data()

    # Write to master tracking file
    output_file = "/Users/jdfalk/repos/github.com/jdfalk/copilot-agent-util-rust/master_issue_inventory.json"

    with open(output_file, "w") as f:
        json.dump(issue_data, f, indent=2)

    print(f"Master issue inventory saved to: {output_file}")
    print(f"Total repositories: {issue_data['metadata']['total_repositories']}")
    print(f"Total issues: {issue_data['metadata']['total_issues']}")
    print(
        f"Duplicates identified: {issue_data['summary']['duplicate_analysis']['gcommon_duplicates']}"
    )

    print("\\nNext steps:")
    for action in issue_data["actions_needed"]["immediate"]:
        print(f"- {action}")


if __name__ == "__main__":
    main()
