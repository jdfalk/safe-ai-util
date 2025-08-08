// file: src/commands/mod.rs
// version: 1.0.0
// guid: 91287190-3d1a-4bda-b077-1342df3f7c09

//! Command implementations for the Copilot Agent Utility

pub mod buf;
pub mod file;
pub mod git;
pub mod linter;
pub mod prettier;
pub mod python;
pub mod system;

use crate::{error::Result, executor::Executor};
use clap::ArgMatches;

/// Trait for command execution
pub trait CommandExecutor {
    /// Execute the command with the given arguments and executor
    async fn execute(matches: &ArgMatches, executor: &Executor) -> Result<()>;
}
