<!-- file: README.md -->
<!-- version: 1.2.0 -->
# Copilot Agent Utility (renaming to "safe-ai-util") - Rust Implementation

> Note: We're transitioning the project name from "copilot-agent-util" to "safe-ai-util". For zero interruption, both binary names are built and supported. You can continue using `copilot-agent-util` or start using `safe-ai-util` today.

## Table of Contents

- [Copilot Agent Utility (renaming to "safe-ai-util") - Rust Implementation](#copilot-agent-utility-renaming-to-safe-ai-util---rust-implementation)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Installation](#installation)
    - [From Cargo](#from-cargo)
    - [From GitHub](#from-github)
    - [From Binary Releases](#from-binary-releases)
  - [Usage](#usage)
  - [Command Categories](#command-categories)
    - [File Operations](#file-operations)
    - [Git Operations](#git-operations)
    - [Protocol Buffers](#protocol-buffers)
    - [Development Tools](#development-tools)
    - [System Operations](#system-operations)
  - [Safety Features](#safety-features)
    - [Command Validation](#command-validation)
    - [Error Recovery](#error-recovery)
    - [Concurrent Safety](#concurrent-safety)
  - [Configuration](#configuration)
    - [Configuration Example](#configuration-example)
  - [Logging](#logging)
    - [Log Levels](#log-levels)
  - [VS Code Integration](#vs-code-integration)
  - [Performance](#performance)
  - [Development](#development)
    - [Building from Source](#building-from-source)
    - [Contributing](#contributing)
    - [Architecture](#architecture)
  - [License](#license)
  - [Security](#security)

## Overview

This Rust application serves as a reliable, safe, and performant intermediary between VS Code tasks and system commands, ensuring proper working directory handling, comprehensive logging, consistent output formatting, and bulletproof error handling.

## Features

- **Memory Safety**: Rust's ownership system prevents memory leaks and data races
- **Robust Error Handling**: Comprehensive error types and graceful failure modes
- **Async/Concurrent Execution**: Efficient handling of multiple operations
- **Type Safety**: Compile-time guarantees for command validation
- **Comprehensive Logging**: Structured logging with multiple output formats
- **VS Code Integration**: Seamless integration with VS Code tasks and workflows
- **Cross-Platform**: Works reliably on macOS, Linux, and Windows
- **Zero-Cost Abstractions**: High-level features without runtime overhead
- **Safe File Operations**: Protected against race conditions and data corruption

## Installation

### From Cargo

```bash
cargo install copilot-agent-util
# During the transition, both binaries are installed:
# - copilot-agent-util (legacy)
# - safe-ai-util (new)
```

### From GitHub

```bash
git clone https://github.com/jdfalk/copilot-agent-util-rust.git
cd copilot-agent-util-rust
cargo build --release
cargo install --path .
```

### From Binary Releases

Download pre-compiled binaries from the [releases page](https://github.com/jdfalk/copilot-agent-util-rust/releases).
Release artifacts include both names, e.g. `copilot-agent-util-macos-arm64` and `safe-ai-util-macos-arm64`.

## Usage

```bash
# Basic command execution
copilot-agent-util exec "ls -la"
# Or using the new name
safe-ai-util exec "ls -la"

# Git operations
copilot-agent-util git add .
copilot-agent-util git commit -m "feat: add new feature"
copilot-agent-util git push
# New name equivalent
safe-ai-util git add .
safe-ai-util git commit -m "feat: add new feature"
safe-ai-util git push

# Protocol buffer operations
copilot-agent-util buf generate
copilot-agent-util buf generate --module auth
# New name equivalent
safe-ai-util buf generate
safe-ai-util buf generate --module auth

# File operations
copilot-agent-util file cat README.md
copilot-agent-util file ls src/

# Development tools
copilot-agent-util python run script.py
copilot-agent-util npm install
copilot-agent-util uv run main.py
# New name equivalent
safe-ai-util python run script.py
safe-ai-util npm install
safe-ai-util uv run main.py

# Safe operations with dry-run
copilot-agent-util --dry-run git push --force-with-lease
copilot-agent-util --dry-run file rm dangerous-file.txt
# New name equivalent
safe-ai-util --dry-run git push --force-with-lease
safe-ai-util --dry-run file rm dangerous-file.txt

# Verbose logging
copilot-agent-util --verbose buf generate
# New name equivalent
safe-ai-util --verbose buf generate
```

## Command Categories

### File Operations

- `file ls <path>` - List directory contents with safety checks
- `file cat <file>` - Display file contents with encoding detection
- `file cp <src> <dst>` - Copy files/directories with integrity verification
- `file mv <src> <dst>` - Move/rename files/directories safely
- `file mkdir <path>` - Create directories with proper permissions
- `file rm <path>` - Remove files/directories with confirmation prompts
- `file find <pattern>` - Search for files with regex support
- `file grep <pattern> <path>` - Search within files with context

### Git Operations

- `git add <pattern>` - Add files to staging with validation
- `git commit -m <message>` - Commit changes with hooks support
- `git push` - Push to remote with safety checks
- `git push --force-with-lease` - Safe force push with lease validation
- `git status` - Show repository status with detailed output
- `git pull` - Pull from remote with conflict detection
- `git branch` - List/create/delete branches safely
- `git checkout <branch>` - Switch branches with state preservation
- `git merge <branch>` - Merge branches with conflict resolution
- `git rebase <branch>` - Interactive rebase with safety guards

### Protocol Buffers

- `buf generate` - Generate all protocol buffers with validation
- `buf generate --module <name>` - Generate specific module safely
- `buf lint` - Lint protocol buffer files with detailed reports
- `buf format` - Format protocol buffer files consistently
- `buf breaking` - Check for breaking changes with impact analysis

### Development Tools

- `python run <script>` - Run Python scripts with environment isolation
- `python build` - Build Python projects with dependency checking
- `python test` - Run Python tests with coverage reporting
- `uv run <command>` - Execute commands with uv environment management
- `npm install` - Install npm dependencies with integrity checks
- `npm run <script>` - Run npm scripts with timeout protection
- `cargo build` - Build Rust projects with optimization
- `cargo test` - Run Rust tests with parallel execution

### System Operations

- `sys ps` - Show running processes with filtering
- `sys env` - Display environment variables securely
- `sys path` - Show PATH variable with validation
- `sys which <command>` - Find command location with alternatives

## Safety Features

### Command Validation

- Input sanitization and validation
- Path traversal protection
- Command injection prevention
- Resource limit enforcement

### Error Recovery

- Graceful degradation on failures
- Automatic retry with exponential backoff
- State preservation during interruptions
- Comprehensive error reporting

### Concurrent Safety

- Thread-safe logging and state management
- Atomic file operations
- Deadlock prevention
- Resource cleanup guarantees

## Configuration

The utility reads configuration from multiple sources in order of precedence:

1. Command-line arguments
2. Environment variables
3. User configuration file: `~/.config/copilot-agent-util/config.toml`
4. Project configuration file: `.copilot-agent-util.toml`
5. Default values

### Configuration Example

```toml
[general]
log_level = "info"
log_format = "json"
working_directory = "."
timeout = 300

[git]
auto_stage = false
require_message = true
push_hooks = true

[safety]
dry_run = false
confirm_destructive = true
backup_before_delete = true

[logging]
file_rotation = true
max_log_size = "10MB"
retention_days = 30
```

## Logging

Comprehensive logging system with multiple output targets:

- **Console Output**: Colored, formatted logs for interactive use
- **File Logging**: Structured logs with rotation and retention
- **JSON Logs**: Machine-readable logs for automation
- **Metrics**: Performance and usage statistics

### Log Levels

- `ERROR`: Critical failures requiring attention
- `WARN`: Non-critical issues and warnings
- `INFO`: General operational information
- `DEBUG`: Detailed debugging information
- `TRACE`: Extremely verbose execution tracing

## VS Code Integration

Update your `.vscode/tasks.json` to use the Rust utility:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Buf Generate",
      "type": "shell",
      "command": "copilot-agent-util",
      "args": ["buf", "generate"],
      "group": "build",
      "options": {
        "cwd": "${workspaceFolder}"
      },
      "problemMatcher": []
    },
    {
      "label": "Git Push Safe",
      "type": "shell",
      "command": "copilot-agent-util",
      "args": ["git", "push", "--force-with-lease"],
      "group": "build",
      "options": {
        "cwd": "${workspaceFolder}"
      }
    }
  ]
}
```

## Performance

Built for high performance with:

- Compiled native binary (no runtime overhead)
- Efficient async I/O operations
- Minimal memory allocation
- Optimized for common workflows
- Parallel execution where safe

## Development

### Building from Source

```bash
# Debug build
cargo build


## Python workflows (venv + pip + pytest)

The `python` command provides safe, repeatable workflows that avoid global installs and shell activation. It uses the venv's interpreter directly for reliability across shells and platforms.

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

### Architecture

- **src/main.rs**: Application entry point and CLI setup
- **src/commands/**: Command implementations and parsing
- **src/executor/**: Safe command execution engine
- **src/logger/**: Structured logging system
- **src/config/**: Configuration management
- **src/error/**: Error types and handling
- **src/utils/**: Utility functions and helpers

## License

See [LICENSE](LICENSE) file.

## Security

This project takes security seriously. See [SECURITY.md](SECURITY.md) for reporting security vulnerabilities.
