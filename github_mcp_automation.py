#!/usr/bin/env python3
# file: github_mcp_automation.py
# version: 1.0.0
# guid: 3b4c5d6e-7f8a-9b0c-1d2e-3f4a5b6c7d8e

"""
GitHub MCP automation script to label all issues and assign to projects
Uses the mcp_github tools available in VS Code
"""

import json
import os
from typing import List, Dict

# Configuration
REPOSITORIES = [
    {'owner': 'jdfalk', 'repo': 'copilot-agent-util-rust'},
    {'owner': 'jdfalk', 'repo': 'subtitle-manager'},
    {'owner': 'jdfalk', 'repo': 'gcommon'},
    {'owner': 'jdfalk', 'repo': 'ghcommon'},
    {'owner': 'jdfalk', 'repo': 'audiobook-organizer'}
]

# Label definitions
STANDARD_LABELS = [
    # Type labels
    'bug', 'enhancement', 'documentation', 'question', 'good first issue',
    'help wanted', 'invalid', 'wontfix', 'duplicate',
    
    # Priority labels
    'priority:high', 'priority:medium', 'priority:low', 'priority:critical',
    
    # Module labels
    'module:auth', 'module:api', 'module:ui', 'module:database', 'module:config',
    'module:metrics', 'module:cache', 'module:queue', 'module:web', 'module:organization',
    
    # Area labels
    'area:backend', 'area:frontend', 'area:infrastructure', 'area:testing',
    'area:documentation', 'area:security', 'area:performance', 'area:accessibility',
    
    # CI/CD labels
    'ci-cd', 'github-actions', 'deployment', 'build', 'release', 'automation',
    
    # Special labels
    'dependencies', 'breaking-change', 'needs-review', 'work-in-progress'
]

# Project categorization rules
PROJECT_CATEGORIES = {
    'Backend Services': {
        'keywords': {
            'module:auth', 'module:api', 'module:database', 'module:metrics', 
            'module:cache', 'module:config', 'module:queue', 'auth', 'grpc',
            'proto', 'protobuf', 'migration', 'module:organization', 'backend'
        },
        'title_keywords': ['grpc', 'auth', 'proto', 'database', 'api', 'server', 'backend']
    },
    'Web & UI': {
        'keywords': {
            'module:web', 'module:ui', 'frontend', 'web', 'ui', 'webui', 'dashboard'
        },
        'title_keywords': ['web', 'ui', 'frontend', 'interface', 'dashboard']
    },
    'Infrastructure': {
        'keywords': {
            'ci-cd', 'github-actions', 'deployment', 'build', 'release', 
            'automation', 'workflow', 'docker', 'cosign', 'infrastructure'
        },
        'title_keywords': ['ci', 'workflow', 'build', 'deploy', 'docker', 'action']
    },
    'Documentation': {
        'keywords': {
            'documentation', 'docs', 'examples', 'guide', 'readme', 'changelog'
        },
        'title_keywords': ['doc', 'readme', 'guide', 'example', 'manual']
    },
    'Testing': {
        'keywords': {
            'test', 'testing', 'integration', 'unit-tests', 'qa', 'validation',
            'area:testing', 'benchmark'
        },
        'title_keywords': ['test', 'testing', 'qa', 'validation', 'spec']
    },
    'SDKs & Tools': {
        'keywords': {
            'sdk', 'tools', 'cli', 'utility', 'script', 'generator', 'dependencies',
            'copilot-agent-util'
        },
        'title_keywords': ['cli', 'tool', 'script', 'utility', 'sdk']
    }
}

def categorize_issue(title: str, labels: List[str], body: str = "") -> str:
    """Categorize an issue based on title, labels, and body"""
    title_lower = title.lower()
    label_set = {label.lower() for label in labels}
    body_lower = body.lower() if body else ""
    
    # Score each category
    category_scores = {}
    for category, config in PROJECT_CATEGORIES.items():
        score = 0
        
        # Label matches (higher weight)
        score += len(label_set.intersection(config['keywords'])) * 3
        
        # Title keyword matches
        score += sum(1 for keyword in config['title_keywords'] if keyword in title_lower) * 2
        
        # Body keyword matches (lower weight)
        if body_lower:
            score += sum(1 for keyword in config['keywords'] if keyword in body_lower)
        
        category_scores[category] = score
    
    # Return highest scoring category or General
    if max(category_scores.values()) > 0:
        return max(category_scores, key=category_scores.get)
    return "General"

def suggest_labels_for_issue(title: str, body: str, existing_labels: List[str]) -> List[str]:
    """Suggest appropriate labels for an issue based on content"""
    suggested_labels = set(existing_labels)  # Keep existing labels
    
    title_lower = title.lower()
    body_lower = body.lower() if body else ""
    content = f"{title_lower} {body_lower}"
    
    # Suggest module labels
    module_suggestions = {
        'module:auth': ['auth', 'authentication', 'login', 'jwt', 'token'],
        'module:api': ['api', 'endpoint', 'rest', 'grpc', 'service'],
        'module:ui': ['ui', 'interface', 'frontend', 'dashboard', 'web'],
        'module:database': ['database', 'sql', 'query', 'migration', 'schema'],
        'module:config': ['config', 'configuration', 'settings', 'environment'],
        'module:metrics': ['metrics', 'monitoring', 'telemetry', 'stats'],
        'module:cache': ['cache', 'caching', 'redis', 'memory'],
        'module:queue': ['queue', 'job', 'task', 'worker', 'background'],
        'module:web': ['web', 'http', 'server', 'handler', 'middleware']
    }
    
    for module, keywords in module_suggestions.items():
        if any(keyword in content for keyword in keywords):
            suggested_labels.add(module)
    
    # Suggest type labels
    if any(word in content for word in ['bug', 'error', 'fix', 'broken', 'issue']):
        suggested_labels.add('bug')
    if any(word in content for word in ['feature', 'add', 'implement', 'new']):
        suggested_labels.add('enhancement')
    if any(word in content for word in ['test', 'testing', 'spec', 'validation']):
        suggested_labels.add('area:testing')
    if any(word in content for word in ['doc', 'documentation', 'readme', 'guide']):
        suggested_labels.add('documentation')
    
    # Suggest priority labels
    if any(word in content for word in ['urgent', 'critical', 'important', 'asap']):
        suggested_labels.add('priority:high')
    elif any(word in content for word in ['minor', 'low', 'nice to have']):
        suggested_labels.add('priority:low')
    else:
        suggested_labels.add('priority:medium')
    
    return list(suggested_labels)

def create_assignment_summary(repo_info: Dict, categorized_issues: Dict):
    """Create assignment summary for a repository"""
    owner = repo_info['owner']
    repo = repo_info['repo']
    
    print(f"\n📋 PROJECT ASSIGNMENTS for {owner}/{repo}")
    print("-" * 50)
    
    assignment_data = {
        'repository': f"{owner}/{repo}",
        'total_issues': sum(len(issues) for issues in categorized_issues.values()),
        'categories': {}
    }
    
    for category, issues in categorized_issues.items():
        if not issues:
            continue
            
        print(f"\n## {category} ({len(issues)} issues)")
        assignment_data['categories'][category] = {
            'count': len(issues),
            'issues': []
        }
        
        for issue in issues[:10]:  # Show first 10
            status_emoji = "🔓" if issue['state'] == 'open' else "🔒"
            print(f"  {status_emoji} #{issue['number']}: {issue['title']}")
            
            assignment_data['categories'][category]['issues'].append({
                'number': issue['number'],
                'title': issue['title'],
                'state': issue['state'],
                'suggested_labels': issue.get('suggested_labels', []),
                'url': issue.get('url', f"https://github.com/{owner}/{repo}/issues/{issue['number']}")
            })
        
        if len(issues) > 10:
            print(f"  ... and {len(issues) - 10} more issues")
    
    # Write assignment file
    assignment_file = f"/Users/jdfalk/repos/github.com/jdfalk/{repo}/project_assignments_{repo}.json"
    with open(assignment_file, 'w') as f:
        json.dump(assignment_data, f, indent=2)
    
    print(f"📝 Assignment file created: {assignment_file}")
    return assignment_data

def main():
    """Main function - creates instructions for using GitHub MCP tools"""
    
    print("🚀 GITHUB MCP AUTOMATION INSTRUCTIONS")
    print("=" * 60)
    print("This script generates instructions for using GitHub MCP tools to:")
    print("1. List all issues in each repository")
    print("2. Analyze and categorize issues")
    print("3. Generate labeling and project assignment commands")
    print("4. Create comprehensive automation instructions")
    print()
    
    # Generate MCP commands for each repository
    commands = []
    
    for repo_info in REPOSITORIES:
        owner = repo_info['owner']
        repo = repo_info['repo']
        
        print(f"\n🔄 REPOSITORY: {owner}/{repo}")
        print("=" * 40)
        
        # Command to list all issues
        list_command = f"mcp_github_list_issues(owner='{owner}', repo='{repo}', state='all', perPage=100)"
        commands.append({
            'repo': f"{owner}/{repo}",
            'command': list_command,
            'purpose': 'List all issues (open and closed)'
        })
        
        print(f"1. List Issues Command:")
        print(f"   {list_command}")
        
        # Command to get issue details (template)
        get_command = f"mcp_github_get_issue(owner='{owner}', repo='{repo}', issue_number=<ISSUE_NUMBER>)"
        commands.append({
            'repo': f"{owner}/{repo}",
            'command': get_command,
            'purpose': 'Get detailed issue information for labeling'
        })
        
        print(f"2. Get Issue Details Command (template):")
        print(f"   {get_command}")
        
        # Command to update issue labels (template)
        update_command = f"mcp_github_update_issue(owner='{owner}', repo='{repo}', issue_number=<ISSUE_NUMBER>, labels=<SUGGESTED_LABELS>)"
        commands.append({
            'repo': f"{owner}/{repo}",
            'command': update_command,
            'purpose': 'Update issue with suggested labels'
        })
        
        print(f"3. Update Issue Labels Command (template):")
        print(f"   {update_command}")
    
    # Write commands to file
    commands_file = "/Users/jdfalk/repos/github.com/jdfalk/copilot-agent-util-rust/github_mcp_commands.json"
    with open(commands_file, 'w') as f:
        json.dump({
            'description': 'GitHub MCP commands for comprehensive issue automation',
            'repositories': REPOSITORIES,
            'standard_labels': STANDARD_LABELS,
            'project_categories': PROJECT_CATEGORIES,
            'commands': commands
        }, f, indent=2)
    
    print(f"\n📝 Commands file created: {commands_file}")
    
    print("\n✅ INSTRUCTIONS GENERATED!")
    print("\nNext steps:")
    print("1. Use the MCP commands to list issues from each repository")
    print("2. For each issue, analyze content and suggest labels")
    print("3. Update issues with appropriate labels using MCP tools")
    print("4. Categorize issues for project assignment")
    print("5. Create GitHub Projects and assign issues")
    print("\nUse the generated commands file for systematic automation.")

if __name__ == '__main__':
    main()
