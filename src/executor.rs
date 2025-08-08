// file: src/executor.rs
// version: 1.0.0
// guid: bb371682-35cb-4f34-b318-8bf69ec125bd

use crate::config::Config;
use anyhow::{anyhow, Result};
use std::process::Stdio;
use tokio::process::Command;
use tracing::info;

/// Safe command executor with comprehensive error handling
pub struct Executor {
    config: Config,
}

impl Executor {
    /// Create a new executor with the given configuration
    pub async fn new(config: Config) -> Result<Self> {
        Ok(Self { config })
    }

    /// Execute a raw command with arguments
    pub async fn execute_raw(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow!("No command provided"));
        }

        let command = args[0];
        let command_args = &args[1..];

        info!("Executing command: {} with args: {:?}", command, command_args);

        if self.config.safety.dry_run {
            println!("DRY RUN: Would execute: {} {:?}", command, command_args);
            return Ok(());
        }

        // Validate command exists
        if which::which(command).is_err() {
            return Err(anyhow!("Command not found: {}", command));
        }

        // Execute command
        let mut cmd = Command::new(command);
        cmd.args(command_args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        // Set working directory if specified
        if let Some(ref wd) = self.config.general.working_directory {
            cmd.current_dir(wd);
        }

        let status = cmd.status().await
            .map_err(|e| anyhow!("Failed to execute command: {}", e))?;

        if !status.success() {
            return Err(anyhow!(
                "Command failed with exit code: {:?}",
                status.code()
            ));
        }

        info!("Command executed successfully");
        Ok(())
    }
}
