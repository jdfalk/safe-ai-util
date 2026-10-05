<!-- file: README.md -->
<!-- version: 1.5.1 -->
<!-- guid: 73ce8c1c-699b-46cc-bf5c-6185e5337fd9 -->
<!-- last-edited: 2026-10-04 -->
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
  - [Using as a sudo gate](#using-as-a-sudo-gate)
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
git clone https://github.com/falkcorp/safe-ai-util.git
cd safe-ai-util
cargo build --release
cargo install --path .
```

### From Binary Releases

Download pre-compiled binaries from the [releases page](https://github.com/falkcorp/safe-ai-util/releases).
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

## Using as a sudo gate

`safe-ai-util exec` can be the only command a user may run as root through
sudo. It runs one binary from a fixed, root-owned policy with an argv that
must match the policy exactly.

```sudoers
jdfalk ALL=(root) NOPASSWD: /usr/local/bin/safe-ai-util exec *
```

```bash
sudo safe-ai-util exec zfs destroy bigdata/rehearsal-sandbox-data
sudo safe-ai-util --dry-run exec zfs destroy bigdata/rehearsal-sandbox-data  # check only
```

The policy lives at `/etc/safe-ai-util/root-policy.toml` and nowhere else.
[`examples/aorg-sandbox-root-policy.toml`](examples/aorg-sandbox-root-policy.toml)
is a complete example that allows exactly seven `zfs` commands and one
built-in `chown-tree`. A rule
looks like this:

```toml
[commands.zfs]
binary = "/usr/sbin/zfs"          # absolute; PATH is never searched
requires_elevation = true         # refused outside the elevated exec path
allowed_argv = [                  # exact argument vectors, element for element
  ["destroy", "bigdata/rehearsal-sandbox-data"],
]
```

### What happens when the process is elevated

safe-ai-util counts as elevated when euid is 0, when euid differs from the
real uid, or when `SUDO_USER`/`SUDO_UID` is set. No flag or environment
variable can make it count as less elevated. When it is elevated:

- `--config`, `--policy-overlay` and `--args-file` are refused (exit 77).
- Every subcommand except `exec` is refused (exit 77). They read user config
  from `$HOME` and the working directory, take log and audit paths from
  environment variables, and several of them look tools up via PATH.
- Logging, config discovery and the general executor are never initialised,
  so `SAFE_AI_UTIL_LOG_DIR`, `SAFE_AI_UTIL_AUDIT_PATH`, `COPILOT_AUDIT_DIR`,
  `./.safe-ai-util.toml` and `~/.config/safe-ai-util/config.toml` are not read.
- The policy file and every directory above it must be owned by root and not
  writable by group or other. The file is opened without following a final
  symlink, and the checks run on the opened file. Otherwise exit 78.
- The rule's binary is resolved through symlinks. The result must be a
  root-owned executable that is not writable by group or other, in a
  root-owned directory chain. Shells, interpreters and program launchers
  (`sh`, `bash`, `python*`, `perl`, `env`, `xargs`, `busybox`, `find` and
  others) are refused both when the policy loads and when the binary is
  resolved.
- argv goes straight to `execve`: no shell, and no splitting, quoting or
  rewriting. `"x; rm -rf /"` is one argument, and it matches nothing.
- The child gets an empty environment plus the policy's `path`
  (default `/usr/sbin:/usr/bin:/sbin:/bin`), `/` as its working directory and
  `/dev/null` as stdin.
- The child runs with umask `022`, whatever umask the caller had.
- Every allowed and refused `exec` is written as a JSON line to syslog
  (authpriv) and to the policy's `log_file`
  (default `/var/log/safe-ai-util/root-exec.log`; its directory must be
  root-owned). Its directory is resolved without following symlinks in any
  path component. If the log file cannot be written, nothing runs (exit 74).
- Exit status: the child's own status when it runs, 77 when refused, 78 for a
  missing or unsafe policy, 74 for an audit-log failure, 71 if the child could
  not start.

A leading `--` after `exec` is accepted and dropped. A `--` anywhere later
counts as an argument, so it has to appear in the rule.

### Path preconditions and `chown-tree`

argv names paths, and a path is only as safe as the directories above it.
Two policy fields make a rule check paths immediately before it runs:

```toml
[commands.zfs-clone-sandbox-data]
binary = "/usr/sbin/zfs"
allowed_argv = [["clone", "-o", "mountpoint=/mnt/aorg-sandbox/data", "..."]]
# Every ancestor (/, /mnt, /mnt/aorg-sandbox) must be a real directory,
# root-owned, not group/other-writable. The path itself may not exist yet.
require_root_owned_ancestors = ["/mnt/aorg-sandbox/data"]
# Optional: the path must be the mount point of this ZFS dataset.
# require_mount = { path = "/mnt/aorg-sandbox/data", dataset = "pool/fs" }
```

Several rules may share a binary, so one argv can carry a precondition while
another does not. Every rule for that binary is consulted, whether you invoke
it by rule name or by binary path. The preconditions of **every** rule that
accepts the argv apply, and so does the strictest `requires_elevation`. A
broad rule therefore cannot shadow a guarded one. Rules for the same binary
may not list the same exact argv, and two rules that accept one argv but
require different mounts are refused.

`chown-tree` replaces `chown -R`. GNU chown and uutils chown (which some
distributions ship as `/usr/bin/chown`) do not handle symlinks under `-R` the
same way, and a gate should not depend on which one is installed.

```toml
[chown_trees.sandbox-data]
path = "/mnt/aorg-sandbox/data"
owner = 1000
group = 1000
require_mount_of = "bigdata/rehearsal-sandbox-data"   # mandatory
```

`sudo safe-ai-util exec chown-tree /mnt/aorg-sandbox/data` takes exactly one
argument, which must equal a configured `path`. Before changing anything it
checks:

- every ancestor of `path` is root-owned and not group/other-writable;
- `path` is the topmost mount at that location, is `zfs`, has source
  `require_mount_of` and mount root `/` (not a bind mount of a
  subdirectory), and its `major:minor` in `/proc/self/mountinfo` equals the
  `st_dev` of `path`.

All of this happens on a single descriptor:

1. The target is opened one component at a time from `/` with
   `openat(O_NOFOLLOW|O_DIRECTORY)`, and each ancestor is checked through its
   descriptor.
2. That descriptor's `fstat` supplies the device, its `fstatfs` must report
   ZFS (`0x2fc12fc1`), and the mountinfo match uses that device.
3. The walk starts from that same descriptor.

The walk is post-order: a directory's contents change before the directory,
and the top changes last.

- **Directories** are opened `O_NOFOLLOW|O_DIRECTORY` and changed with
  `fchown` on the descriptor.
- **Every other entry** is opened `O_PATH|O_NOFOLLOW` and checked with
  `fstat`, then changed with `fchownat(fd, "", AT_EMPTY_PATH)`. The inode
  checked is therefore the inode changed, and a symlink's own ownership
  changes while its target is never touched.
- **Skipped:** entries whose `st_dev` differs from the mount, and a
  top-level `.zfs`. Hard links cannot span filesystems, so nothing outside
  the dataset is reachable, whatever the sandbox user puts in the tree.
- **Limits:** depth is capped at 256 levels (one descriptor per level), and
  `RLIMIT_NOFILE` is raised to its hard limit first.
- **Errors:** a per-entry error is counted and the walk continues. Any
  failure makes the run exit 1, with the counts in the audit log.

`chown-tree` is Linux-only.

**Install requirement for the example policy:** `/mnt` and
`/mnt/aorg-sandbox` must exist as `root:root 0755`
(`install -d -o root -g root -m 0755 /mnt/aorg-sandbox`). Without them both
clones (media at `/mnt/aorg-sandbox/media`, data at `/mnt/aorg-sandbox/data`)
and `chown-tree` are refused. Every clone rule sets an explicit `mountpoint=`:
an inherited mount point can sit under a directory that a non-root user can
write.

Without sudo, `exec` accepts only `--dry-run`. It reads the same fixed policy
and prints whether the command would be allowed.

### Allow rules outside the sudo gate

`allowed_patterns` and the new `allowed_argv` are enforced everywhere a
`CommandRestrictions` applies, including the `[allowlist]` section of a normal
config file:

- `allowed_argv`: the arguments must equal one listed vector exactly.
- `allowed_patterns`: every argument must fully match at least one pattern.
  Patterns are anchored as `^(?:pattern)$`.
- Both empty: unprivileged runs keep the old behaviour (only the deny rules
  apply). Elevated runs treat it as deny.
- A rule with `requires_elevation = true` is refused on the unprivileged
  path. It can only run through `exec` under the root policy.

### Threat model

**Protected against**, for a caller who can run `sudo safe-ai-util ...` with
any arguments and any environment sudo passes through:

- choosing or widening the policy (`--config`, overlays, cwd or `$HOME` config,
  env-var overrides, a group- or world-writable policy file or directory);
- running anything other than a listed binary with a listed argv, including
  extra or reordered arguments, flags such as `-r`, and shell metacharacters;
- reaching a shell or interpreter through a policy rule or a symlinked binary;
- leaking environment (`LD_PRELOAD`, `PATH`, and so on) into the child;
- making root create or append files at caller-chosen paths through the log
  and audit environment variables.

**Not protected against:**

- **Filesystem state outside guarded paths.** The gate pins argv, not what a
  path points at. For paths a rule guards with
  `require_root_owned_ancestors`, `require_mount` or `chown-tree`, the
  ancestors are checked immediately before the action. For any other path
  named in an argv, a caller who can write to one of its parents can swap in
  a symlink or a directory. Guard every such path, or keep its directories
  root-owned.
- **The safe-ai-util binary's own location.** `/usr/local/bin/safe-ai-util`
  and every directory above it must be root-owned and not writable by
  others. If the caller can replace the binary, for example because
  `/usr/local/bin` is user-writable as in some Homebrew-style setups, the
  sudoers line hands them a root shell. safe-ai-util cannot check this for
  itself in time.
- **Programs that interpret their own arguments.** The deny list covers
  shells, interpreters and launchers by name. It does not know every
  program that can run code from an argument: the dynamic loader
  (`ld-linux*.so`) and `zfs program`, which runs Lua, are two examples.
  Exact `allowed_argv` rules, as in the shipped example, make this moot.
  Broad `allowed_patterns` rules do not.
- **What an allowed command does.** Each allowed argv is a capability the
  caller holds. `zfs destroy` of a listed dataset destroys it. Only list
  commands you are willing to have run at any time, in any order.
- **A compromised root.** Root can edit the policy, the binary, or sudoers.
- **sudoers itself.** A rule that allows a different binary, or `SETENV`,
  is outside this tool's control. Use `env_reset` (the default) and never put
  a writable path in `secure_path`. Do not set `closefrom_override`: sudo
  then lets the caller keep open file descriptors (`sudo -C`), which root and
  the child would inherit. safe-ai-util does not close inherited descriptors
  itself.
- **Non-unix platforms.** `exec` refuses to run there.

**Behaviour change for root callers.** Anything that ran safe-ai-util as root
before this change, such as a container with no `USER` line, now gets exit 77
for every subcommand except `exec`. That is deliberate: as root, user config
and environment-chosen paths cannot be trusted. Run those callers as an
unprivileged user.

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
