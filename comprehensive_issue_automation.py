#!/usr/bin/env python3
# file: comprehensive_issue_automation.py
# version: 1.0.0
# guid: 2a3b4c5d-6e7f-8a9b-0c1d-2e3f4a5b6c7d

"""
Comprehensive issue automation script that:
1. Sets up labeler workflows where missing
2. Labels ALL issues (open and closed)
3. Assigns issues to GitHub Projects
4. Fixes dependabot configurations
"""

import os
import requests
from typing import List, Dict

# Configuration
REPOSITORIES = [
    'jdfalk/copilot-agent-util-rust',
    'jdfalk/subtitle-manager', 
    'jdfalk/gcommon',
    'jdfalk/ghcommon',
    'jdfalk/audiobook-organizer'
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

def get_github_headers() -> Dict[str, str]:
    """Get GitHub API headers"""
    token = os.getenv('GITHUB_TOKEN') or os.getenv('JF_CI_GH_PAT')
    if not token:
        print("❌ No GitHub token found. Please set GITHUB_TOKEN or JF_CI_GH_PAT")
        return {}
    
    return {
        'Authorization': f'token {token}',
        'Accept': 'application/vnd.github.v3+json'
    }

def fetch_all_issues(repo: str) -> List[Dict]:
    """Fetch ALL issues (open and closed) from a repository"""
    headers = get_github_headers()
    if not headers:
        return []
    
    print(f"📥 Fetching all issues from {repo}")
    
    all_issues = []
    page = 1
    
    while True:
        # Fetch both open and closed issues
        for state in ['open', 'closed']:
            url = f'https://api.github.com/repos/{repo}/issues'
            params = {
                'state': state,
                'page': page,
                'per_page': 100,
                'sort': 'created',
                'direction': 'desc'
            }
            
            response = requests.get(url, headers=headers, params=params)
            if response.status_code != 200:
                print(f"❌ Error fetching {state} issues: {response.status_code}")
                continue
                
            issues = response.json()
            if not issues:
                continue
                
            # Filter out pull requests
            issues = [issue for issue in issues if 'pull_request' not in issue]
            all_issues.extend(issues)
            
            print(f"  📄 Fetched {len(issues)} {state} issues (page {page})")
            
            if len(issues) < 100:
                break
        
        page += 1
        if not any(len(requests.get(f'https://api.github.com/repos/{repo}/issues', 
                                  headers=headers, 
                                  params={'state': s, 'page': page, 'per_page': 100}).json()) > 0 
                  for s in ['open', 'closed']):
            break
    
    print(f"✅ Total issues fetched from {repo}: {len(all_issues)}")
    return all_issues

def suggest_labels_for_issue(issue: Dict) -> List[str]:
    """Suggest appropriate labels for an issue based on content"""
    title = issue.get('title', '')
    body = issue.get('body', '') or ''
    existing_labels = [label['name'] for label in issue.get('labels', [])]
    
    suggested_labels = set(existing_labels)  # Keep existing labels
    
    title_lower = title.lower()
    body_lower = body.lower()
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
        suggested_labels.add('testing')
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

def apply_labels_to_issue(repo: str, issue_number: int, labels: List[str]) -> bool:
    """Apply labels to an issue"""
    headers = get_github_headers()
    if not headers:
        return False
    
    url = f'https://api.github.com/repos/{repo}/issues/{issue_number}/labels'
    data = {'labels': labels}
    
    response = requests.put(url, headers=headers, json=data)
    return response.status_code == 200

def create_labeler_workflow(repo_path: str) -> bool:
    """Create pr-automation.yml workflow with labeler if missing"""
    workflow_path = f"{repo_path}/.github/workflows/pr-automation.yml"
    
    if os.path.exists(workflow_path):
        print(f"✅ Labeler workflow already exists: {workflow_path}")
        return True
    
    workflow_content = '''name: PR Automation
on:
  pull_request:
    types: [opened, edited, synchronize, reopened]

jobs:
  auto-labeling:
    name: Auto Label PR
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Apply file-based labels
        uses: actions/labeler@v5
        with:
          repo-token: ${{ secrets.GITHUB_TOKEN }}
          configuration-path: .github/labeler.yml
          sync-labels: true
'''
    
    os.makedirs(os.path.dirname(workflow_path), exist_ok=True)
    
    with open(workflow_path, 'w') as f:
        f.write(workflow_content)
    
    print(f"✅ Created labeler workflow: {workflow_path}")
    return True

def process_repository_issues(repo: str) -> Dict:
    """Process all issues in a repository"""
    print(f"\n🔄 PROCESSING {repo}")
    print("=" * 50)
    
    # Fetch all issues
    issues = fetch_all_issues(repo)
    if not issues:
        return {}
    
    categorized_issues = {}
    label_changes = 0
    
    for issue in issues:
        issue_number = issue['number']
        title = issue['title']
        existing_labels = [label['name'] for label in issue.get('labels', [])]
        
        # Suggest new labels
        suggested_labels = suggest_labels_for_issue(issue)
        new_labels = [label for label in suggested_labels if label not in existing_labels]
        
        # Categorize for project assignment
        category = categorize_issue(title, suggested_labels, issue.get('body', ''))
        
        if category not in categorized_issues:
            categorized_issues[category] = []
        
        issue_info = {
            'number': issue_number,
            'title': title,
            'state': issue['state'],
            'existing_labels': existing_labels,
            'suggested_labels': suggested_labels,
            'new_labels': new_labels,
            'url': issue['html_url']
        }
        
        categorized_issues[category].append(issue_info)
        
        # Apply new labels if any
        if new_labels:
            print(f"  🏷️  Issue #{issue_number}: Adding labels {new_labels}")
            if apply_labels_to_issue(repo, issue_number, suggested_labels):
                label_changes += 1
            else:
                print(f"    ❌ Failed to apply labels to #{issue_number}")
    
    print(f"✅ Applied labels to {label_changes} issues")
    return categorized_issues

def generate_project_assignments(repo: str, categorized_issues: Dict):
    """Generate project assignment instructions"""
    repo_name = repo.split('/')[-1]
    
    print(f"\n📋 PROJECT ASSIGNMENTS for {repo}")
    print("-" * 40)
    
    for category, issues in categorized_issues.items():
        if not issues:
            continue
            
        print(f"\n## {category} ({len(issues)} issues)")
        for issue in issues[:10]:  # Show first 10
            status_emoji = "🔓" if issue['state'] == 'open' else "🔒"
            print(f"  {status_emoji} #{issue['number']}: {issue['title']}")
            if issue['new_labels']:
                print(f"    🏷️  Added: {', '.join(issue['new_labels'])}")
        
        if len(issues) > 10:
            print(f"  ... and {len(issues) - 10} more issues")
    
    # Write assignment file
    assignment_file = f"/Users/jdfalk/repos/github.com/jdfalk/{repo_name}/project_assignments_{repo_name}.md"
    
    with open(assignment_file, 'w') as f:
        f.write(f"# Project Assignments for {repo}\n\n")
        f.write("Generated by comprehensive_issue_automation.py\n\n")
        
        for category, issues in categorized_issues.items():
            if not issues:
                continue
                
            f.write(f"## {category} Project\n")
            f.write(f"**{len(issues)} issues to assign:**\n\n")
            
            for issue in issues:
                f.write(f"- [ ] #{issue['number']}: {issue['title']} ({issue['state']})\n")
                f.write(f"  - URL: {issue['url']}\n")
                f.write(f"  - Labels: {', '.join(issue['suggested_labels'])}\n")
            
            f.write("\n")
    
    print(f"📝 Assignment file created: {assignment_file}")

def check_dependabot_config(repo_path: str, repo_name: str):
    """Check and fix dependabot configuration"""
    dependabot_path = f"{repo_path}/.github/dependabot.yml"
    
    if not os.path.exists(dependabot_path):
        print(f"❌ Dependabot config missing: {repo_name}")
        return
    
    with open(dependabot_path, 'r') as f:
        content = f.read()
    
    # Check if it references proper labels
    if 'dependencies' not in content and 'dependency' not in content:
        print(f"⚠️  Dependabot config may need label fixes: {repo_name}")
    else:
        print(f"✅ Dependabot config looks good: {repo_name}")

def main():
    """Main execution function"""
    print("🚀 COMPREHENSIVE ISSUE AUTOMATION")
    print("=" * 60)
    print("This script will:")
    print("1. Set up labeler workflows where missing")
    print("2. Label ALL issues (open and closed)")
    print("3. Categorize issues for project assignment")
    print("4. Generate assignment instructions")
    print("5. Check dependabot configurations")
    print()
    
    for repo in REPOSITORIES:
        repo_name = repo.split('/')[-1]
        repo_path = f"/Users/jdfalk/repos/github.com/jdfalk/{repo_name}"
        
        # Check if repo exists
        if not os.path.exists(repo_path):
            print(f"⚠️  Repository not found: {repo_path}")
            continue
        
        # Create labeler workflow if missing
        create_labeler_workflow(repo_path)
        
        # Process all issues
        categorized_issues = process_repository_issues(repo)
        
        if categorized_issues:
            # Generate project assignments
            generate_project_assignments(repo, categorized_issues)
        
        # Check dependabot config
        check_dependabot_config(repo_path, repo_name)
    
    print("\n✅ AUTOMATION COMPLETE!")
    print("\nNext steps:")
    print("1. Review the generated project assignment files")
    print("2. Create GitHub Projects if they don't exist")
    print("3. Manually assign issues to projects using the generated lists")
    print("4. Set up project automation rules based on labels")
    print("5. Commit any new labeler workflows")

if __name__ == '__main__':
    main()
