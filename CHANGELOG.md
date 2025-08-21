## [1.4.1](https://github.com/jdfalk/copilot-agent-util-rust/compare/v1.4.0...v1.4.1) (2025-08-21)


### Bug Fixes

* **release:** proper token usage and file separation in Rust release workflow ([b4cb6f5](https://github.com/jdfalk/copilot-agent-util-rust/commit/b4cb6f5f8743f6b0591616fa73e730590279ce19))

## [1.4.0](https://github.com/jdfalk/copilot-agent-util-rust/compare/v1.3.0...v1.4.0) (2025-08-21)


### Features

* add cross-platform build and distribution scripts for copilot-agent-util ([62fa342](https://github.com/jdfalk/copilot-agent-util-rust/commit/62fa342e5a24d4e53f8cf334fb65784e9fe9fd3e))
* add initial configuration for Copilot Agent Utility ([97822e9](https://github.com/jdfalk/copilot-agent-util-rust/commit/97822e9202c027291a4a402ce1943fb08b9455fb))
* add Rust coding style guide and best practices documentation ([55ab7e5](https://github.com/jdfalk/copilot-agent-util-rust/commit/55ab7e5ca0c6f54cf3db5cd34df6c4de018554da))
* Add several workflows to handle releasing code. ([f76f99f](https://github.com/jdfalk/copilot-agent-util-rust/commit/f76f99f2e271e4fcc3ba634498d5f99c284fe431))
* add support for additional arguments from environment variable in buf, linter, prettier, and uutils commands ([7b8d73f](https://github.com/jdfalk/copilot-agent-util-rust/commit/7b8d73fa9462d2fcae4582981b55c868400f0fd7))
* add test arguments for git status ([7f38710](https://github.com/jdfalk/copilot-agent-util-rust/commit/7f387105981942126a444bda410d4d4c7c2f4c00))
* **ci:** implement modular release workflow system ([d95f389](https://github.com/jdfalk/copilot-agent-util-rust/commit/d95f38916367295d88845ceb4e0a1cb5a565d2a5))
* **ci:** replace GoReleaser with Rust-specific release workflow ([4e0e6f1](https://github.com/jdfalk/copilot-agent-util-rust/commit/4e0e6f13abe91c60a74f01262c71380e0a9c6516))
* **ci:** update tasks.json for modular release workflow integration ([dda3a7d](https://github.com/jdfalk/copilot-agent-util-rust/commit/dda3a7d08eda6b74ba4338ace7ea6ab469ae149d))
* complete uutils integration and comprehensive command suite ([71e1d45](https://github.com/jdfalk/copilot-agent-util-rust/commit/71e1d45ab8f84a1b2d50c1d818356a19f3a9cf20))
* **executor:** replace execute_raw with execute_secure for enhanced security in command execution ([311974c](https://github.com/jdfalk/copilot-agent-util-rust/commit/311974c8d59688efd21b7e0e78873255d4c92589))
* **labels:** add comprehensive labels configuration with colors and descriptions ([f579ed7](https://github.com/jdfalk/copilot-agent-util-rust/commit/f579ed7c157b99a0305b1fe9a137a2f61d486adb))
* migrate to consolidated workflows from gcommon ([11c00ef](https://github.com/jdfalk/copilot-agent-util-rust/commit/11c00efec1b563707bdb84490073044889ea7acb))
* **release:** add GitHub Actions workflow for automated releases with language detection ([8c7aa34](https://github.com/jdfalk/copilot-agent-util-rust/commit/8c7aa3431c3f15f7e795ccc00c9e99f9a15f591b))
* replace sync workflows with consolidated receiver pattern ([b88633b](https://github.com/jdfalk/copilot-agent-util-rust/commit/b88633b7fd02f27b5a5d9f22705eb5747a8d925b))
* **security:** enhance git clean argument validation for safety checks ([b09e8be](https://github.com/jdfalk/copilot-agent-util-rust/commit/b09e8be6026ec254e92a4cdd2506e3b1cd97f111))
* **security:** Implement comprehensive security module for command execution ([688d1d5](https://github.com/jdfalk/copilot-agent-util-rust/commit/688d1d57e357d4bf4bf0a5d1ca8dfd3768065085))
* **security:** implement comprehensive security system to prevent remote code execution ([d2ada7b](https://github.com/jdfalk/copilot-agent-util-rust/commit/d2ada7b3813f06acbcfd53d586552206b207003b))
* **sync:** add scripts for synchronization, commit, and summary generation ([bde69db](https://github.com/jdfalk/copilot-agent-util-rust/commit/bde69db7eed846309795f6506be403587d744fda))
* **sync:** update sync parameters and enhance summary output in sync-receiver workflow ([a83ac24](https://github.com/jdfalk/copilot-agent-util-rust/commit/a83ac24b5e37c969332d0e2743ac672eec6f3877))
* **sync:** update version to 1.2.0 and enhance sync functionality with detailed logging ([b6a3718](https://github.com/jdfalk/copilot-agent-util-rust/commit/b6a3718ce50f9eb1b862e243775f792efe0234a1))
* **sync:** update version to 1.2.0 and refine sync logic to exclude restricted workflows ([e70528d](https://github.com/jdfalk/copilot-agent-util-rust/commit/e70528d7b929c034b88ec9d85cfdef4f1e98e113))
* update sync receiver with enhanced file sync capabilities ([ddfe294](https://github.com/jdfalk/copilot-agent-util-rust/commit/ddfe2945eca07ae4ec60da9d08f6793c3850f02b))
* update VSCode tasks to use copilot-agent-util for Buf commands and Git operations ([1cd7451](https://github.com/jdfalk/copilot-agent-util-rust/commit/1cd74519eb5dc1ee7ea85e598ded00a1fd402645))
* **workflow:** enhance sync-receiver with verbose logging ([6c9083a](https://github.com/jdfalk/copilot-agent-util-rust/commit/6c9083a9fb6092fa94b6a7a58569f824999a21c3))


### Bug Fixes

* **buf:** replace raw execution with secure execution in lint command ([d53178e](https://github.com/jdfalk/copilot-agent-util-rust/commit/d53178e3067ee6d4270c0019a019429877317bf5))
* disable all sync workflows to prevent file overwrites ([7e817e5](https://github.com/jdfalk/copilot-agent-util-rust/commit/7e817e53436411adb59b5c06537960199edd7bb0))
* **editor:** optimize current line retrieval in insert_newline method ([3f08d93](https://github.com/jdfalk/copilot-agent-util-rust/commit/3f08d9304542309ecb52a1a8f3435072ed8c4ef4))
* **release:** resolve macOS sed command issue in Rust release workflow ([628b1c1](https://github.com/jdfalk/copilot-agent-util-rust/commit/628b1c114cae6fc47ab411186d2c86a7b08ddf40))
* remove VALIDATE_PYTHON_PYLINT false setting from super-linter ([8f3633e](https://github.com/jdfalk/copilot-agent-util-rust/commit/8f3633e615a20e44f7cad1859033d10b02df0095))
* **scripts:** remove deprecated ::set-output fallback in parse_config_json.py ([f07c759](https://github.com/jdfalk/copilot-agent-util-rust/commit/f07c759069cbba9d40ccc1be719048a3e96ed4e1))
* update reusable workflow reference for stale issues ([4b0edab](https://github.com/jdfalk/copilot-agent-util-rust/commit/4b0edab810e1487fbe590f72e156f8516b273dd3))
* **workflow:** implement proper token handling for GitHub labels sync ([3c34948](https://github.com/jdfalk/copilot-agent-util-rust/commit/3c349480f2fefe9822a0c1347780b2eaa59e2ded))
* **workflow:** resolve GitHub App workflow permission issue v1.3.0 ([d1fd2f2](https://github.com/jdfalk/copilot-agent-util-rust/commit/d1fd2f252033c3e18710965a7f2fdd6bfccc25c6))

# file: CHANGELOG.md

## Table of Contents

- [file: CHANGELOG.md](#file-changelog-md)
  - [Table of Contents](#table-of-contents)
- [version: 1.0.0](#version-1-0-0)
- [guid: 6f789012-bcde-f345-6789-0123456789ab](#guid-6f789012-bcde-f345-6789-0123456789ab)
- [Changelog](#changelog)
  - [[1.0.0] - 2025-08-08](#-1-0-0-2025-08-08)
    - [Added](#added)
    - [Technical Features](#technical-features)
    - [Security](#security)
    - [Documentation](#documentation)
  - [[Unreleased]](#-unreleased)
    - [Coming Soon](#coming-soon)
  - [Release Notes](#release-notes)
- [From GitHub releases](#from-github-releases)
- [From Docker](#from-docker)

## [1.0.0] - 2025-08-08

### Added

- 🎉 **Initial release of Rust copilot-agent-util**
- **Protocol Buffer Operations** with comprehensive `buf` command support
  - Generate, lint, format, breaking analysis, build, push operations
  - Smart aliasing system for cross-command compatibility
- **Comprehensive Linting Suite** (`linter` command)
  - Multi-language support: ESLint, flake8, mypy, clippy, golangci-lint
  - Additional tools: ShellCheck, Hadolint, yamllint, markdownlint
  - Auto-fix capabilities and configuration support
- **Code Formatting Tools** (`prettier` command)
  - Cross-language formatters: Prettier, Black, rustfmt, gofmt, goimports
  - Additional tools: isort, buf format, shfmt, clang-format
  - Multiple output modes and write-back support
- **Core Operations**
  - Safe command execution with comprehensive error handling
  - Git, file, Python, and system operations
  - Dry-run mode for safe testing
  - Verbose logging with structured output
- **Configuration Management**
  - TOML-based configuration with sensible defaults
  - Environment variable support
  - Working directory management
- **Docker Support**
  - Multi-stage Dockerfile for optimized container images
  - Security-focused container with non-root user
  - Health checks and proper entrypoint configuration
- **CI/CD Pipeline**
  - Comprehensive GitHub Actions workflows
  - Multi-platform builds (Linux, Windows, macOS)
  - Security scanning with CodeQL and Trivy
  - Automated releases with attestations
  - Dependency management with Dependabot

### Technical Features

- **Memory Safety**: Rust's ownership system prevents memory-related bugs
- **Async/Await**: Tokio runtime for efficient I/O operations
- **Type Safety**: Comprehensive error handling with `anyhow`
- **Performance**: Zero-cost abstractions and optimized release builds
- **Cross-Platform**: Support for Linux, Windows, and macOS
- **Container Ready**: Optimized Docker images for deployment

### Security

- Supply chain security with build attestations
- Vulnerability scanning with cargo-audit and Trivy
- CodeQL static analysis
- Non-root container execution
- Dependency review automation

### Documentation

- Comprehensive CLI help system
- Structured logging with contextual information
- Error messages with actionable guidance
- Docker usage instructions

## [Unreleased]

### Coming Soon

- Configuration file validation
- Plugin system for custom commands
- Performance metrics and benchmarking
- Integration tests suite
- Shell completion scripts

---

## Release Notes

This is the first stable release of the Rust implementation of `copilot-agent-util`.
It provides all the functionality of the original Go version with enhanced safety,
performance, and additional tooling capabilities.

The Rust version emphasizes:

- **Safety**: Memory safety guaranteed by Rust's type system
- **Performance**: Compiled binary with zero-cost abstractions
- **Reliability**: Comprehensive error handling and logging
- **Security**: Built-in security scanning and attestations
- **Maintainability**: Clean architecture with modular design

Install with:

```bash
# From GitHub releases
wget https://github.com/jdfalk/copilot-agent-util-rust/releases/latest/download/copilot-agent-util-linux-x86_64.tar.gz
tar -xzf copilot-agent-util-linux-x86_64.tar.gz
sudo mv copilot-agent-util /usr/local/bin/copilot-agent-utilr

# From Docker
docker pull ghcr.io/jdfalk/copilot-agent-util-rust:latest
```
