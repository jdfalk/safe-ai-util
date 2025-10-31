<!-- file: MIGRATION_STATUS.md -->
<!-- version: 1.0.0 -->
<!-- guid: a1b2c3d4-e5f6-7890-abcd-ef1234567890 -->

# Migration Status: copilot-agent-util-rust → safe-ai-util + MCP Server

**Date Started:** 2025-10-30
**Status:** 🚀 IN PROGRESS
**Last Updated:** 2025-10-30 (initial)

## Overview

This document tracks the three-objective migration:

1. Create new `safe-ai-util-mcp` repository with proper settings
2. Rename `copilot-agent-util-rust` to `safe-ai-util`
3. Implement MCP server in the new repository

## Objective 1: Create safe-ai-util-mcp Repository ✅

### Repository Creation

* [x] **Create repo via gh CLI**
  * Command: `gh repo create jdfalk/safe-ai-util-mcp --public --description "MCP server for safe-ai-util - exposes AI-safe command execution tools via ModelContext Protocol"`
  * Status: Not started
  * Blocker: None

### Repository Settings Configuration

* [x] **Merge settings**
  * Disable merge commits
  * Disable squash merging
  * Enable rebase merging only
  * Status: Not started

* [x] **Pull request settings**
  * Enable "Always suggest updating pull request branches"
  * Enable auto-merge
  * Status: Not started

* [x] **Branch settings**
  * Enable automatic deletion of head branches
  * Status: Not started

* [x] **Branch protection ruleset**
  * Create ruleset for `main` branch
  * Prevent deletion
  * Require pull request before merging (optional, decide later)
  * Status: Not started

### Initial Repository Setup

* [x] **Clone locally**
  * Location: `/Users/jdfalk/repos/github.com/jdfalk/safe-ai-util-mcp`
  * Status: Not started

* [x] **Initial commit**
  * Basic .gitignore (Python)
  * README stub
  * LICENSE
  * Status: Not started

## Objective 2: Rename copilot-agent-util-rust → safe-ai-util ⏳

### GitHub Repository Rename

* [ ] **Rename via gh CLI or web**
  * Old: `jdfalk/copilot-agent-util-rust`
  * New: `jdfalk/safe-ai-util`
  * Method: `gh repo rename safe-ai-util` (from repo directory)
  * Status: Not started
  * Note: GitHub auto-redirects old URLs

### Codebase Updates

* [ ] **Cargo.toml**
  * Update `name = "safe-ai-util"`
  * Update repository URL
  * Update documentation URL
  * Status: Not started

* [ ] **README.md**
  * Update title and references
  * Update installation instructions
  * Update binary name in examples
  * Update repository links
  * Status: Not started

* [ ] **Workflow files (.github/workflows/\*.yml)**
  * Update all repository references
  * Update binary artifact names
  * Files to check:
    * ci.yml
    * release-rust.yml
    * Any others referencing copilot-agent-util
  * Status: Not started

* [ ] **Documentation files**
  * CHANGELOG.md (already updated with unreleased section)
  * CONTRIBUTING.md
  * Any other docs with old name
  * Status: CHANGELOG updated, others pending

* [ ] **Build/distribution scripts**
  * build-cross-platform.sh
  * distribute-binaries.sh
  * Any scripts that reference binary name
  * Status: Not started

### Testing & Validation

* [ ] **Build test**
  * `cargo build --release`
  * Verify binary name is `safe-ai-util`
  * Status: Not started

* [ ] **URL redirect test**
  * Verify old GitHub URLs redirect to new repo
  * Status: Not started

* [ ] **Release artifact test**
  * Update release workflow
  * Test that artifacts have correct names
  * Status: Not started

## Objective 3: Implement MCP Server in safe-ai-util-mcp ⏳

### Project Scaffolding

* [ ] **Python project structure**
  * pyproject.toml with mcp dependency
  * src/safe_ai_util_mcp/ package
  * src/safe_ai_util_mcp/\_\_init\_\_.py
  * src/safe_ai_util_mcp/server.py
  * README.md
  * .gitignore
  * LICENSE
  * Status: Not started

* [ ] **Development environment**
  * Create requirements-dev.txt (pytest, ruff, mypy)
  * Set up .venv
  * Status: Not started

### MCP Server Implementation

* [ ] **Core server setup**
  * Import mcp SDK
  * Set up stdio transport
  * Create Server instance
  * Implement main() entry point
  * Status: Not started

* [ ] **Binary wrapper utilities**
  * Function to locate safe-ai-util binary (env var → PATH)
  * Function to execute with timeout
  * Function to normalize results
  * Status: Not started

* [ ] **Tool implementations - Git**
  * `git_status` tool
  * `git_add` tool (with pattern support)
  * `git_commit` tool (with message validation)
  * `git_push` tool (with branch option)
  * Status: Not started

* [ ] **Tool implementations - Buf**
  * `buf_lint` tool (with optional module)
  * `buf_generate` tool (with optional module)
  * Status: Not started

* [ ] **Tool implementations - Python**
  * `python_venv_ensure` tool
  * `python_venv_remove` tool
  * `python_pip_install` tool
  * `python_run_pytest` tool
  * Status: Not started

### Security & Safety

* [ ] **Path validation**
  * Workspace boundary enforcement
  * Reject absolute paths outside workspace
  * Status: Not started

* [ ] **Argument sanitization**
  * No shell metacharacters in args
  * Use subprocess argv arrays only
  * Status: Not started

* [ ] **Environment sanitization**
  * Strip dangerous variables (PYTHONPATH, etc.)
  * Pass only safe environment
  * Status: Not started

* [ ] **Timeout enforcement**
  * Per-tool configurable timeouts
  * Default: 300s
  * Status: Not started

### Documentation

* [ ] **README.md**
  * Installation instructions
  * Client configuration examples (Claude, Continue.dev)
  * Tool catalog with schemas
  * Security model explanation
  * Status: Not started

* [ ] **Client config examples**
  * Claude Desktop config snippet
  * Continue.dev config snippet
  * Generic stdio MCP client config
  * Status: Not started

* [ ] **API documentation**
  * Document each tool's schema
  * Input parameters
  * Output format
  * Error handling
  * Status: Not started

### Testing

* [ ] **Unit tests**
  * Test tool handlers with mocked binary
  * Test path validation
  * Test argument sanitization
  * Status: Not started

* [ ] **Integration tests**
  * Test with real safe-ai-util binary
  * Test stdio transport
  * Status: Not started

* [ ] **Security tests**
  * Test path traversal rejection
  * Test shell injection prevention
  * Test timeout enforcement
  * Status: Not started

### CI/CD

* [ ] **GitHub Actions workflow**
  * Lint (ruff)
  * Type check (mypy)
  * Test (pytest)
  * Optional: PyPI publish
  * Status: Not started

## Current Blockers

None at this time.

## Next Steps (Immediate)

1. Execute gh CLI command to create safe-ai-util-mcp repository
2. Configure repository settings via gh CLI or web UI
3. Clone repository locally
4. Begin rename of copilot-agent-util-rust to safe-ai-util

## Notes & Decisions

* **Why separate repo for MCP?**
  * Clean separation of concerns (tooling vs protocol server)
  * Python SDK is mature and fast to iterate
  * Preserves Rust tool's security model
  * Easier adoption by MCP clients

* **Why rename to safe-ai-util?**
  * More descriptive of purpose (AI-safe command execution)
  * Shorter and cleaner
  * "copilot-agent-util" was too GitHub-specific

* **Binary compatibility**
  * Old installations will continue to work (GitHub redirects)
  * Users can rename binary locally if desired
  * Future releases will use new name

## Risk Assessment

* **Low risk:** Repository rename (GitHub auto-redirects)
* **Low risk:** MCP server implementation (separate repo, no impact on existing tool)
* **Medium risk:** Build/distribution script updates (need thorough testing)

## Rollback Plan

If issues arise:

1. Repository rename: Can rename back via gh CLI
2. MCP server: Separate repo, no impact on main tool
3. Binary name: Can be reverted in Cargo.toml

## Success Criteria

* [ ] All three objectives completed
* [ ] All tests passing
* [ ] Documentation updated
* [ ] Zero disruption to existing users
* [ ] MCP server successfully connects to Claude Desktop
* [ ] At least one successful tool call via MCP

## Timeline Estimate

* Objective 1: 15-30 minutes
* Objective 2: 30-60 minutes
* Objective 3: 2-3 hours

**Total:** 3-4.5 hours
