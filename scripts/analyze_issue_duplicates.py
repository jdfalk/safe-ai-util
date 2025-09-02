#!/usr/bin/env python3
# file: scripts/analyze_issue_duplicates.py
# version: 1.0.0
# guid: a1b2c3d4-e5f6-7890-abcd-ef1234567890

"""
GitHub Issue Duplicate Analysis and Bulk Operations Script

This script analyzes all collected GitHub issues across repositories to:
1. Identify and catalog duplicate issues
2. Apply comprehensive labeling based on sophisticated categorization
3. Assign issues to appropriate GitHub Projects
4. Generate bulk operation commands for MCP tools

Features:
- Duplicate detection by title similarity and content analysis
- Advanced categorization with module-specific labeling
- Project assignment based on issue characteristics
- Bulk operation command generation for MCP tools
"""

import re
from typing import Dict, List
from collections import defaultdict
import difflib

class IssueAnalyzer:
    def __init__(self):
        # Module-based categorization patterns
        self.module_patterns = {
            'module:auth': ['auth', 'authentication', 'authorization', 'jwt', 'oauth', 'login', 'token', 'session'],
            'module:config': ['config', 'configuration', 'settings', 'environment', 'env'],
            'module:database': ['database', 'db', 'sql', 'sqlite', 'cockroach', 'transaction', 'migration'],
            'module:cache': ['cache', 'redis', 'memcached', 'caching'],
            'module:queue': ['queue', 'message', 'task', 'job', 'worker', 'async'],
            'module:metrics': ['metrics', 'monitoring', 'prometheus', 'stats', 'telemetry'],
            'module:notification': ['notification', 'alert', 'email', 'sms', 'webhook'],
            'module:organization': ['organization', 'tenant', 'team', 'hierarchy', 'multi-tenant'],
            'module:ui': ['ui', 'interface', 'frontend', 'web', 'html', 'css', 'javascript'],
            'module:log': ['log', 'logging', 'logger', 'audit'],
            'module:grpc': ['grpc', 'protobuf', 'proto', 'rpc'],
            'module:web': ['web', 'http', 'rest', 'api', 'server'],
            'module:security': ['security', 'crypto', 'encryption', 'tls', 'ssl'],
            'module:deployment': ['deploy', 'deployment', 'docker', 'kubernetes', 'k8s'],
            'module:monitoring': ['monitor', 'health', 'status', 'uptime']
        }
        
        # Priority classification patterns
        self.priority_patterns = {
            'priority:high': ['critical', 'urgent', 'blocker', 'broken', 'crash', 'security', 'data loss'],
            'priority:medium': ['important', 'enhancement', 'feature', 'improvement'],
            'priority:low': ['minor', 'nice to have', 'cleanup', 'refactor', 'documentation']
        }
        
        # Type classification patterns
        self.type_patterns = {
            'type:bug': ['bug', 'error', 'issue', 'problem', 'broken', 'fail', 'crash'],
            'type:feature': ['feature', 'add', 'implement', 'create', 'new'],
            'type:enhancement': ['enhance', 'improve', 'better', 'optimize', 'upgrade'],
            'type:docs': ['documentation', 'docs', 'readme', 'guide', 'tutorial'],
            'type:testing': ['test', 'testing', 'spec', 'coverage'],
            'type:maintenance': ['cleanup', 'refactor', 'reorganize', 'update'],
            'type:protobuf': ['protobuf', 'proto', 'grpc', 'buf']
        }
        
        # Technical classification patterns
        self.tech_patterns = {
            'tech:protobuf': ['protobuf', 'proto', '.proto', 'buf', 'grpc'],
            'tech:grpc': ['grpc', 'rpc', 'service'],
            'tech:go': ['go', 'golang', '.go'],
            'tech:rust': ['rust', 'cargo', '.rs'],
            'tech:python': ['python', '.py', 'pip'],
            'tech:javascript': ['javascript', 'js', 'npm', 'node'],
            'tech:docker': ['docker', 'dockerfile', 'container'],
            'tech:kubernetes': ['kubernetes', 'k8s', 'kubectl']
        }
        
        # Special labels
        self.special_labels = {
            'automation': ['github-actions', 'ci', 'workflow', 'automated'],
            'codex': ['ai', 'generated', 'copilot', 'automated'],
            'security': ['security', 'vulnerability', 'cve'],
            'performance': ['performance', 'slow', 'optimize', 'speed'],
            'dependencies': ['dependency', 'upgrade', 'update', 'version']
        }

    def normalize_title(self, title: str) -> str:
        """Normalize issue title for comparison"""
        # Remove common prefixes and suffixes
        title = re.sub(r'^(feat|fix|docs|style|refactor|test|chore)(\([^)]+\))?\s*:\s*', '', title, flags=re.IGNORECASE)
        
        # Remove version numbers, dates, and IDs
        title = re.sub(r'\b\d+\.\d+\.\d+\b', '', title)  # Version numbers
        title = re.sub(r'\b\d{4}-\d{2}-\d{2}\b', '', title)  # Dates
        title = re.sub(r'#\d+', '', title)  # Issue numbers
        
        # Remove common variations
        title = re.sub(r'\b(implement|add|create|fix|resolve|update)\b', '', title, flags=re.IGNORECASE)
        
        # Normalize whitespace
        title = ' '.join(title.split())
        
        return title.lower().strip()

    def calculate_similarity(self, title1: str, title2: str) -> float:
        """Calculate similarity between two issue titles"""
        norm1 = self.normalize_title(title1)
        norm2 = self.normalize_title(title2)
        
        # Use SequenceMatcher for similarity calculation
        similarity = difflib.SequenceMatcher(None, norm1, norm2).ratio()
        
        # Boost similarity for exact substring matches
        if norm1 in norm2 or norm2 in norm1:
            similarity = max(similarity, 0.8)
            
        return similarity

    def find_duplicates(self, issues: List[Dict]) -> Dict[str, List[Dict]]:
        """Find duplicate issues based on title similarity"""
        duplicates = defaultdict(list)
        processed = set()
        
        for i, issue1 in enumerate(issues):
            if i in processed:
                continue
                
            group = [issue1]
            processed.add(i)
            
            for j, issue2 in enumerate(issues[i+1:], i+1):
                if j in processed:
                    continue
                    
                similarity = self.calculate_similarity(issue1['title'], issue2['title'])
                
                if similarity >= 0.7:  # 70% similarity threshold
                    group.append(issue2)
                    processed.add(j)
            
            if len(group) > 1:
                duplicates[f"duplicate_group_{len(duplicates)+1}"] = group
        
        return dict(duplicates)

    def categorize_issue(self, issue: Dict) -> Dict[str, List[str]]:
        """Categorize an issue and suggest labels"""
        title = issue['title'].lower()
        body = issue.get('body', '').lower()
        
        combined_text = f"{title} {body}"
        
        suggested_labels = {
            'module': [],
            'priority': [],
            'type': [],
            'technical': [],
            'special': []
        }
        
        # Module classification
        for module, patterns in self.module_patterns.items():
            if any(pattern in combined_text for pattern in patterns):
                suggested_labels['module'].append(module)
        
        # Priority classification
        for priority, patterns in self.priority_patterns.items():
            if any(pattern in combined_text for pattern in patterns):
                suggested_labels['priority'].append(priority)
        
        # Type classification
        for type_label, patterns in self.type_patterns.items():
            if any(pattern in combined_text for pattern in patterns):
                suggested_labels['type'].append(type_label)
        
        # Technical classification
        for tech, patterns in self.tech_patterns.items():
            if any(pattern in combined_text for pattern in patterns):
                suggested_labels['technical'].append(tech)
        
        # Special classification
        for special, patterns in self.special_labels.items():
            if any(pattern in combined_text for pattern in patterns):
                suggested_labels['special'].append(special)
        
        return suggested_labels

    def suggest_project_assignment(self, issue: Dict, suggested_labels: Dict) -> str:
        """Suggest GitHub Project assignment based on issue characteristics"""
        
        # High priority project mapping
        if any('priority:high' in label for label in suggested_labels.get('priority', [])):
            return "Critical Issues"
        
        # Module-based project mapping
        if suggested_labels.get('module'):
            primary_module = suggested_labels['module'][0]
            module_name = primary_module.replace('module:', '').title()
            return f"{module_name} Development"
        
        # Type-based project mapping
        if 'type:protobuf' in suggested_labels.get('type', []):
            return "Protobuf Infrastructure"
        
        if 'type:docs' in suggested_labels.get('type', []):
            return "Documentation"
        
        # Default assignment
        return "General Development"

    def generate_mcp_commands(self, issues: List[Dict], repository: str) -> List[str]:
        """Generate MCP command sequences for bulk operations"""
        commands = []
        
        for issue in issues:
            issue_number = issue['number']
            suggested_labels = self.categorize_issue(issue)
            
            # Flatten suggested labels
            all_labels = []
            for category, labels in suggested_labels.items():
                all_labels.extend(labels)
            
            # Add existing labels that should be preserved
            existing_labels = [label['name'] for label in issue.get('labels', [])]
            preserved_labels = [label for label in existing_labels if label in [
                'bug', 'enhancement', 'documentation', 'security', 'testing', 'performance'
            ]]
            
            all_labels.extend(preserved_labels)
            all_labels = list(set(all_labels))  # Remove duplicates
            
            if all_labels:
                # Generate label update command
                labels_str = ', '.join(f'"{label}"' for label in all_labels)
                commands.append(
                    f'mcp_github_update_issue(owner="jdfalk", repo="{repository}", '
                    f'issue_number={issue_number}, labels=[{labels_str}])'
                )
        
        return commands

def main():
    """Main execution function"""
    analyzer = IssueAnalyzer()
    
    # Sample issues for analysis (would be loaded from actual data)
    print("GitHub Issue Duplicate Analysis and Bulk Operations")
    print("=" * 60)
    
    print("\nAnalysis Features:")
    print("- Duplicate detection with 70% similarity threshold")
    print("- Advanced categorization with module-specific labeling")
    print("- Project assignment based on issue characteristics")
    print("- Bulk operation command generation for MCP tools")
    
    print("\nClassification Categories:")
    print(f"- Module labels: {len(analyzer.module_patterns)} categories")
    print(f"- Priority labels: {len(analyzer.priority_patterns)} levels")
    print(f"- Type labels: {len(analyzer.type_patterns)} types")
    print(f"- Technical labels: {len(analyzer.tech_patterns)} technologies")
    print(f"- Special labels: {len(analyzer.special_labels)} categories")
    
    print("\nReady for issue analysis and bulk operations!")
    print("Load issue data and call analyzer methods to process.")

if __name__ == "__main__":
    main()
