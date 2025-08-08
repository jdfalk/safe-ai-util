// file: src/commands/buf.rs
// version: 1.0.0
// guid: fad7290a-3a19-4f89-a33c-15a68278800c

use crate::{error::{AgentError, Result}, executor::Executor};
use clap::{Arg, ArgMatches, Command};
use tracing::{debug, info};

/// Build the buf command with comprehensive subcommands
pub fn build_command() -> Command {
    Command::new("buf")
        .about("Protocol buffer operations with buf")
        .subcommand(
            Command::new("generate")
                .about("Generate code from protocol buffers")
                .arg(
                    Arg::new("module")
                        .long("module")
                        .short('m')
                        .value_name("MODULE")
                        .help("Generate for specific module")
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .short('p')
                        .value_name("PATH")
                        .help("Path to protocol buffer files")
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_name("OUTPUT_DIR")
                        .help("Output directory for generated code")
                )
        )
        .subcommand(
            Command::new("lint")
                .about("Lint protocol buffer files")
                .arg(
                    Arg::new("path")
                        .value_name("PATH")
                        .help("Path to lint (defaults to current directory)")
                        .default_value(".")
                )
                .arg(
                    Arg::new("config")
                        .long("config")
                        .short('c')
                        .value_name("CONFIG_FILE")
                        .help("Path to buf configuration file")
                )
        )
        .subcommand(
            Command::new("format")
                .about("Format protocol buffer files")
                .arg(
                    Arg::new("path")
                        .value_name("PATH")
                        .help("Path to format (defaults to current directory)")
                        .default_value(".")
                )
                .arg(
                    Arg::new("write")
                        .long("write")
                        .short('w')
                        .action(clap::ArgAction::SetTrue)
                        .help("Write formatted output to files")
                )
        )
        .subcommand(
            Command::new("breaking")
                .about("Check for breaking changes")
                .arg(
                    Arg::new("against")
                        .long("against")
                        .value_name("REF")
                        .help("Git reference to compare against")
                        .default_value("main")
                )
        )
        .subcommand(
            Command::new("build")
                .about("Build protocol buffer modules")
                .arg(
                    Arg::new("path")
                        .value_name("PATH")
                        .help("Path to build (defaults to current directory)")
                        .default_value(".")
                )
        )
        .subcommand(
            Command::new("push")
                .about("Push to Buf Schema Registry")
                .arg(
                    Arg::new("tag")
                        .long("tag")
                        .short('t')
                        .value_name("TAG")
                        .help("Tag for the push")
                )
        )
}

/// Execute buf commands
pub async fn execute(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    match matches.subcommand() {
        Some(("generate", sub_matches)) => execute_generate(sub_matches, executor).await,
        Some(("lint", sub_matches)) => execute_lint(sub_matches, executor).await,
        Some(("format", sub_matches)) => execute_format(sub_matches, executor).await,
        Some(("breaking", sub_matches)) => execute_breaking(sub_matches, executor).await,
        Some(("build", sub_matches)) => execute_build(sub_matches, executor).await,
        Some(("push", sub_matches)) => execute_push(sub_matches, executor).await,
        _ => {
            println!("No buf subcommand specified. Use 'buf --help' for usage information.");
            Ok(())
        }
    }
}

async fn execute_generate(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    let mut args = vec!["buf", "generate"];

    if let Some(module) = matches.get_one::<String>("module") {
        args.extend(&["--path", &format!("pkg/{}/proto", module)]);
    }

    if let Some(path) = matches.get_one::<String>("path") {
        args.extend(&["--path", path]);
    }

    if let Some(output) = matches.get_one::<String>("output") {
        args.extend(&["--output", output]);
    }

    info!("Generating protocol buffers with args: {:?}", args);
    executor.execute_raw(&args).await
}

async fn execute_lint(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    let path = matches.get_one::<String>("path").unwrap();
    let mut args = vec!["buf", "lint", path];

    if let Some(config) = matches.get_one::<String>("config") {
        args.extend(&["--config", config]);
    }

    info!("Linting protocol buffers at path: {}", path);
    executor.execute_raw(&args).await
}

async fn execute_format(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    let path = matches.get_one::<String>("path").unwrap();
    let mut args = vec!["buf", "format"];

    if matches.get_flag("write") {
        args.push("--write");
    }

    args.push(path);

    info!("Formatting protocol buffers at path: {}", path);
    executor.execute_raw(&args).await
}

async fn execute_breaking(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    let against = matches.get_one::<String>("against").unwrap();
    let args = vec!["buf", "breaking", "--against", against];

    info!("Checking for breaking changes against: {}", against);
    executor.execute_raw(&args).await
}

async fn execute_build(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    let path = matches.get_one::<String>("path").unwrap();
    let args = vec!["buf", "build", path];

    info!("Building protocol buffers at path: {}", path);
    executor.execute_raw(&args).await
}

async fn execute_push(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    let mut args = vec!["buf", "push"];

    if let Some(tag) = matches.get_one::<String>("tag") {
        args.extend(&["--tag", tag]);
    }

    info!("Pushing to Buf Schema Registry");
    executor.execute_raw(&args).await
}
