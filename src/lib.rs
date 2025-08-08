// file: src/lib.rs
// version: 1.0.0
// guid: d82472d1-7f0f-4eb4-b0a3-6e1547103eb4

//! # Copilot Agent Utility
//!
//! An extremely safe centralized command execution utility designed to solve VS Code task
//! execution issues and provide comprehensive logging for Copilot/AI agent operations.
//!
//! This Rust implementation emphasizes memory safety, error handling, and robust concurrent execution.

pub mod commands;
pub mod config;
pub mod error;
pub mod executor;
pub mod logger;
pub mod utils;

pub use error::{AgentError, Result};

/// Version information for the utility
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = concat!(
    "Version: ", env!("CARGO_PKG_VERSION"), "\n",
    "Git Hash: ", env!("VERGEN_GIT_SHA_SHORT"), "\n",
    "Build Date: ", env!("VERGEN_BUILD_DATE"), "\n",
    "Rust Version: ", env!("VERGEN_RUSTC_SEMVER")
);
