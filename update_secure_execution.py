#!/usr/bin/env python3
# file: update_secure_execution.py
# version: 1.0.0
# guid: a1b2c3d4-e5f6-7890-abcd-ef1234567890

"""
Script to update command files to use execute_secure instead of execute_raw
"""

import re
from pathlib import Path


def update_file(file_path):
    """Update a single file to use secure execution"""
    print(f"Processing {file_path}")

    with open(file_path, "r") as f:
        content = f.read()

    # Pattern to match the execute_raw usage
    pattern = r'(\s+)args\.insert\(0, "([^"]+)"\.to_string\(\)\);\s*\n\s+let args_str: Vec<&str> = args\.iter\(\)\.map\(\|s\| s\.as_str\(\)\)\.collect\(\);\s*\n\s+executor\.execute_raw\(&args_str\)\.await'
    
    def replacement(match):
        indent = match.group(1)
        command = match.group(2)
        return f'{indent}executor.execute_secure("{command}", &args).await'
    
    # Apply replacement
    new_content = re.sub(pattern, replacement, content)
    
    # Pattern for direct execute_raw with inline args mapping
    pattern2 = r'(\s+)executor\.execute_raw\(&args\.iter\(\)\.map\(\|s\| s\.as_str\(\)\)\.collect::<Vec<&str>>\(\)\)\.await'
    
    def replacement2(match):
        indent = match.group(1)
        # For these cases, we need to extract the command from the args vector
        # Let's look for the pattern where args starts with the command
        return f'{indent}match args.first() {{\n{indent}    Some(cmd) => executor.execute_secure(cmd, &args[1..]).await,\n{indent}    None => anyhow::bail!("No command specified")\n{indent}}}'
    
    new_content = re.sub(pattern2, replacement2, new_content)
    
    # Pattern for direct execute_raw with &args
    pattern3 = r'(\s+)executor\.execute_raw\(&args\)\.await'
    
    def replacement3(match):
        indent = match.group(1)
        return f'{indent}match args.first() {{\n{indent}    Some(cmd) => executor.execute_secure(cmd, &args[1..]).await,\n{indent}    None => anyhow::bail!("No command specified")\n{indent}}}'
    
    new_content = re.sub(pattern3, replacement3, new_content)
    
    if content != new_content:
        with open(file_path, "w") as f:
            f.write(new_content)
        print(f"Updated {file_path}")
        return True
    else:
        print(f"No changes needed for {file_path}")
        return False


def main():
    """Update all command files"""
    commands_dir = Path("src/commands")

    if not commands_dir.exists():
        print("Commands directory not found!")
        return

    updated_files = []

    for rust_file in commands_dir.glob("*.rs"):
        if update_file(rust_file):
            updated_files.append(rust_file)

    print(f"\nUpdated {len(updated_files)} files:")
    for file in updated_files:
        print(f"  - {file}")


if __name__ == "__main__":
    main()
