#!/usr/bin/env python3
# file: scripts/apply_comprehensive_labeling.py
# version: 1.0.0
# guid: d4e5f6g7-h8i9-0123-def4-456789012345

"""
Comprehensive Issue Labeling Script

Applies systematic labeling to issues across all repositories
using the categorization logic and GitHub MCP tools.
"""

import json
from typing import List, Dict, Any

def get_comprehensive_labels(title: str, existing_labels: List[str] = None) -> List[str]:
    """Generate comprehensive label set for an issue"""
    if existing_labels is None:
        existing_labels = []
    
    title_lower = title.lower()
    labels = set(existing_labels)
    
    # Module-specific labels
    module_keywords = {
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
        "workflow": "module:workflow"
    }
    
    for keyword, label in module_keywords.items():
        if keyword in title_lower:
            labels.add(label)
    
    # Priority labels
    if any(word in title_lower for word in ["critical", "urgent", "blocker", "breaking"]):
        labels.add("priority:high")
    elif any(word in title_lower for word in ["enhancement", "feature", "improve"]):
        labels.add("priority:medium")
    else:
        labels.add("priority:low")
    
    # Type labels
    if any(word in title_lower for word in ["bug", "fix", "error", "fail"]):
        labels.add("type:bug")
    elif any(word in title_lower for word in ["feature", "implement", "add"]):
        labels.add("type:feature") 
    elif any(word in title_lower for word in ["documentation", "docs"]):
        labels.add("type:docs")
    elif any(word in title_lower for word in ["test", "testing"]):
        labels.add("type:test")
    elif any(word in title_lower for word in ["refactor", "clean", "improve"]):
        labels.add("type:refactor")
    else:
        labels.add("type:task")
    
    # Technical labels
    if "protobuf" in title_lower or "proto" in title_lower:
        labels.add("protobuf")
    if "migration" in title_lower:
        labels.add("migration")
    if "1-1-1" in title_lower:
        labels.add("1-1-1-migration")
    if "missing" in title_lower and "file" in title_lower:
        labels.add("missing-files")
    if "compilation" in title_lower or "build" in title_lower:
        labels.add("compilation-blocker")
    if "infrastructure" in title_lower:
        labels.add("infrastructure")
    if "automation" in title_lower or "ci" in title_lower:
        labels.add("automation")
    if "security" in title_lower:
        labels.add("security")
    if "performance" in title_lower:
        labels.add("performance")
    
    # Size labels based on title complexity
    if len(title_lower.split()) > 10:
        labels.add("size:large")
    elif len(title_lower.split()) > 5:
        labels.add("size:medium")
    else:
        labels.add("size:small")
    
    return sorted(list(labels))

def generate_labeling_commands() -> List[str]:
    """Generate MCP commands for comprehensive labeling"""
    
    commands = []
    
    # Key gcommon issues to label
    gcommon_issues = [
        {
            "number": 185,
            "title": "Complete GCommon Protobuf 1-1-1 Migration and Missing File Implementation",
            "current_labels": ["protobuf", "1-1-1-migration", "missing-files", "compilation-blocker"]
        },
        {
            "number": 184, 
            "title": "Logging Module 1-1-1 Migration Implementation",
            "current_labels": ["module:log", "1-1-1-migration"]
        },
        {
            "number": 183,
            "title": "Protobuf: Implement core Auth messages",
            "current_labels": ["protobuf", "module:auth"]
        },
        {
            "number": 162,
            "title": "Organize GitHub Project Board",
            "current_labels": ["enhancement", "project-management"]
        }
    ]
    
    # Generate commands for gcommon
    for issue in gcommon_issues:
        comprehensive_labels = get_comprehensive_labels(issue["title"], issue["current_labels"])
        commands.append(f"# Apply comprehensive labeling to gcommon issue #{issue['number']}")
        commands.append(f"mcp_github_update_issue(")
        commands.append(f'    owner="jdfalk",')
        commands.append(f'    repo="gcommon",')
        commands.append(f'    issue_number={issue["number"]},')
        commands.append(f'    labels={json.dumps(comprehensive_labels)}')
        commands.append(f")")
        commands.append("")
    
    # Key ghcommon issues
    ghcommon_issues = [
        {
            "number": 65,
            "title": "Implement comprehensive GitHub automation workflow",
            "current_labels": ["enhancement", "automation"]
        },
        {
            "number": 60,
            "title": "Setup automated labeling for all repositories", 
            "current_labels": ["automation", "workflow"]
        }
    ]
    
    # Generate commands for ghcommon
    for issue in ghcommon_issues:
        comprehensive_labels = get_comprehensive_labels(issue["title"], issue["current_labels"])
        commands.append(f"# Apply comprehensive labeling to ghcommon issue #{issue['number']}")
        commands.append(f"mcp_github_update_issue(")
        commands.append(f'    owner="jdfalk",')
        commands.append(f'    repo="ghcommon",')
        commands.append(f'    issue_number={issue["number"]},')
        commands.append(f'    labels={json.dumps(comprehensive_labels)}')
        commands.append(f")")
        commands.append("")
    
    # Key copilot-agent-util-rust issues
    rust_issues = [
        {
            "number": 2,
            "title": "Add configuration file support for build options",
            "current_labels": ["enhancement"]
        },
        {
            "number": 1,
            "title": "Implement cross-platform binary distribution", 
            "current_labels": ["feature", "enhancement"]
        }
    ]
    
    # Generate commands for copilot-agent-util-rust
    for issue in rust_issues:
        comprehensive_labels = get_comprehensive_labels(issue["title"], issue["current_labels"])
        commands.append(f"# Apply comprehensive labeling to copilot-agent-util-rust issue #{issue['number']}")
        commands.append(f"mcp_github_update_issue(")
        commands.append(f'    owner="jdfalk",')
        commands.append(f'    repo="copilot-agent-util-rust",')
        commands.append(f'    issue_number={issue["number"]},')
        commands.append(f'    labels={json.dumps(comprehensive_labels)}')
        commands.append(f")")
        commands.append("")
    
    return commands

def main():
    """Main execution function"""
    print("Comprehensive Issue Labeling System")
    print("=" * 35)
    
    # Generate labeling commands
    commands = generate_labeling_commands()
    
    print(f"Generated {len([c for c in commands if c.startswith('mcp_')])} labeling commands")
    
    # Save commands to file
    output_file = "/Users/jdfalk/repos/github.com/jdfalk/copilot-agent-util-rust/labeling_commands.txt"
    with open(output_file, 'w') as f:
        f.write("# Comprehensive Issue Labeling Commands\\n")
        f.write("# Generated by apply_comprehensive_labeling.py\\n\\n")
        for command in commands:
            f.write(command + "\\n")
    
    print(f"Commands saved to: {output_file}")
    
    # Example of how labels are generated
    example_title = "Complete GCommon Protobuf 1-1-1 Migration and Missing File Implementation"
    example_labels = get_comprehensive_labels(example_title, ["protobuf"])
    
    print(f"\\nExample labeling:")
    print(f"Title: {example_title}")
    print(f"Generated labels: {example_labels}")
    
    print("\\nNext: Execute these commands using GitHub MCP tools")

if __name__ == "__main__":
    main()
