// file: src/main.rs
// version: 2.3.0
// guid: 9dc55dfd-921c-4db5-84e1-fbccd6b03a6b

use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use copilot_agent_util::{
    commands::{awk, buf, editor, file, git, linter, prettier, python, sed, system, uutils},
    config::Config,
    executor::Executor,
    logger::setup_logging,
};
use std::env;
use std::fs;
use tracing::{error, info};

/// Helper function to append additional arguments from environment variable
#[allow(dead_code)]
fn append_additional_args(mut args: Vec<String>) -> Vec<String> {
    if let Ok(additional_args_str) = env::var("COPILOT_AGENT_ADDITIONAL_ARGS") {
        let additional_args: Vec<&str> = additional_args_str.lines().collect();
        for arg in additional_args {
            if !arg.trim().is_empty() {
                args.push(arg.to_string());
            }
        }
    }
    args
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first
    setup_logging()?;

    // Load configuration
    let config = Config::load().await?;

    info!("Starting Safe AI Utility");

    // Build CLI
    let app = build_cli();
    let matches = app.get_matches();

    // Create executor with config
    let executor = Executor::new(config).await?;

    // Read additional arguments from file if specified
    let mut additional_args = Vec::new();
    if let Some(args_file) = matches.get_one::<String>("args-file") {
        info!("Reading additional arguments from file: {}", args_file);
        match fs::read_to_string(args_file) {
            Ok(content) => {
                additional_args = content
                    .lines()
                    .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                    .map(|line| line.trim().to_string())
                    .collect();
                info!(
                    "Loaded {} additional arguments from file",
                    additional_args.len()
                );
            }
            Err(e) => {
                error!("Failed to read args file {}: {}", args_file, e);
                std::process::exit(1);
            }
        }
    }

    // Route to appropriate command handler
    match execute_command(&matches, &executor, &additional_args).await {
        Ok(_) => {
            info!("Command executed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Command execution failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn build_cli() -> Command {
    Command::new("safe-ai-util")
        .version(env!("CARGO_PKG_VERSION"))
        .author("jdfalk <jdfalk@users.noreply.github.com>")
        .about("An extremely safe and reliable command execution utility")
        .long_about("A reliable command execution utility designed to solve VS Code task execution issues and provide consistent logging for Copilot/AI agent operations. This Rust implementation emphasizes memory safety, error handling, and robust concurrent execution.")
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose logging")
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .action(clap::ArgAction::SetTrue)
                .help("Show what would be done without executing")
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("Specify custom configuration file")
        )
        .arg(
            Arg::new("args-file")
                .long("args-file")
                .value_name("FILE")
                .help("Read additional arguments from file, one per line")
        )
        .subcommand(git::build_command())
        .subcommand(file::build_command())
        .subcommand(buf::build_command())
        .subcommand(python::build_command())
        .subcommand(system::build_command())
        .subcommand(linter::build_command())
        .subcommand(prettier::build_command())
        .subcommand(sed::build_command())
        .subcommand(awk::build_command())
        .subcommand(editor::build_command())
        .subcommand(uutils::build_command())
}

async fn execute_command(
    matches: &ArgMatches,
    executor: &Executor,
    additional_args: &[String],
) -> Result<()> {
    // Set environment variable for additional args if present
    if !additional_args.is_empty() {
        env::set_var("COPILOT_AGENT_ADDITIONAL_ARGS", additional_args.join("\n"));
    }

    match matches.subcommand() {
        Some(("git", sub_matches)) => git::execute(sub_matches, executor).await,
        Some(("file", sub_matches)) => file::execute(sub_matches, executor).await,
        Some(("buf", sub_matches)) => buf::execute(sub_matches, executor).await,
        Some(("python", sub_matches)) => python::execute(sub_matches, executor).await,
        Some(("system", sub_matches)) => system::execute(sub_matches, executor).await,
        Some(("linter", sub_matches)) => linter::execute(sub_matches, executor).await,
        Some(("prettier", sub_matches)) => prettier::execute(sub_matches, executor).await,
        Some(("sed", sub_matches)) => sed::execute(sub_matches, executor).await,
        Some(("awk", sub_matches)) => awk::execute(sub_matches, executor).await,
        Some(("editor", sub_matches)) => editor::execute(sub_matches, executor).await,
        Some(("uutils", sub_matches)) => uutils::execute(sub_matches, executor).await,
        _ => {
            println!("No command specified. Use --help for usage information.");
            Ok(())
        }
    }
}
