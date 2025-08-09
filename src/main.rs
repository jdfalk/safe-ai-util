// file: src/main.rs
// version: 2.1.0
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
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first
    setup_logging()?;

    // Load configuration
    let config = Config::load().await?;

    info!("Starting Copilot Agent Utility (Rust)");

    // Build CLI
    let app = build_cli();
    let matches = app.get_matches();

    // Create executor with config
    let executor = Executor::new(config).await?;

    // Route to appropriate command handler
    match execute_command(&matches, &executor).await {
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
    Command::new("copilot-agent-util")
        .version(env!("CARGO_PKG_VERSION"))
        .author("jdfalk <jdfalk@users.noreply.github.com>")
        .about("Extremely safe centralized utility for Copilot/AI agent command execution")
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
        .subcommand(
            Command::new("exec")
                .about("Execute arbitrary commands safely")
                .arg(
                    Arg::new("command")
                        .required(true)
                        .num_args(1..)
                        .help("Command to execute")
                )
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

async fn execute_command(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    match matches.subcommand() {
        Some(("exec", sub_matches)) => {
            let command_args: Vec<&str> = sub_matches
                .get_many::<String>("command")
                .unwrap()
                .map(|s| s.as_str())
                .collect();
            executor.execute_raw(&command_args).await
        }
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
