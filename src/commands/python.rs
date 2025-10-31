// file: src/commands/python.rs
// version: 1.2.0
// guid: 38a24a1a-5d79-4344-acac-f99e390fe1ac

use crate::executor::Executor;
use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Build the python command
pub fn build_command() -> Command {
    Command::new("python")
        .about("Python operations: venv management, pip installs, and running tests safely")
        .long_about(
            "Safe Python workflows with virtual environments.

Key ideas:
- Never install globally by default; operations use a venv (default: .venv)
- Avoid shell 'source' activation; uses direct venv python for reliability
- Works cross-platform (bin/Scripts handling) and honors --dry-run

Common flows:
- Ensure venv:           copilot-agent-util python venv ensure --path .venv
- Install requirements:  copilot-agent-util python pip install -r requirements.txt --path .venv
- Run pytest:            copilot-agent-util python run pytest tests/ --path .venv
- Remove venv:           copilot-agent-util python venv remove --path .venv
",
        )
        .subcommand(
            Command::new("venv")
                .about("Virtual environment operations")
                .subcommand(
                    Command::new("ensure")
                        .about("Ensure a Python virtual environment exists (create if missing)")
                        .arg(
                            Arg::new("path")
                                .long("path")
                                .short('p')
                                .value_name("DIR")
                                .default_value(".venv")
                                .help("Path to the virtual environment directory"),
                        )
                        .arg(
                            Arg::new("python")
                                .long("python")
                                .value_name("PY")
                                .default_value("python3")
                                .help("Python interpreter to use for creating the venv"),
                        )
                        .arg(
                            Arg::new("recreate")
                                .long("recreate")
                                .action(ArgAction::SetTrue)
                                .help("Recreate venv if it already exists (deletes then creates)"),
                        )
                        .arg(
                            Arg::new("prompt")
                                .long("prompt")
                                .value_name("NAME")
                                .help("Set the venv prompt name during creation"),
                        ),
                )
                .subcommand(
                    Command::new("remove")
                        .about("Remove a Python virtual environment directory")
                        .arg(
                            Arg::new("path")
                                .long("path")
                                .short('p')
                                .value_name("DIR")
                                .default_value(".venv")
                                .help("Path to the virtual environment directory"),
                        )
                        .arg(
                            Arg::new("force")
                                .long("force")
                                .action(ArgAction::SetTrue)
                                .help(
                                    "Allow removing a directory not named '.venv' (safety guard)",
                                ),
                        ),
                ),
        )
        .subcommand(
            Command::new("pip")
                .about("Pip operations scoped to a virtual environment (never global by default)")
                .subcommand(
                    Command::new("install")
                        .about("Install packages using pip within the venv")
                        .arg(
                            Arg::new("path")
                                .long("path")
                                .short('p')
                                .value_name("DIR")
                                .default_value(".venv")
                                .help("Path to the virtual environment directory"),
                        )
                        .arg(
                            Arg::new("requirements")
                                .long("requirements")
                                .short('r')
                                .value_name("FILE")
                                .help("Install from the given requirements file"),
                        )
                        .arg(
                            Arg::new("upgrade-pip")
                                .long("upgrade-pip")
                                .action(ArgAction::SetTrue)
                                .help("Upgrade pip before installing packages"),
                        )
                        .arg(
                            Arg::new("package")
                                .value_name("PKG")
                                .num_args(1..)
                                .help("Additional packages to install (e.g., pytest)`"),
                        )
                        .arg(
                            Arg::new("allow-global")
                                .long("allow-global")
                                .action(ArgAction::SetTrue)
                                .hide(true)
                                .help(
                                    "Allow running pip outside of a venv (unsafe; not recommended)",
                                ),
                        ),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Run Python tools within the venv")
                .subcommand(
                    Command::new("pytest")
                        .about("Run pytest via 'python -m pytest' within the venv")
                        .arg(
                            Arg::new("path")
                                .long("path")
                                .short('p')
                                .value_name("DIR")
                                .default_value(".venv")
                                .help("Path to the virtual environment directory"),
                        )
                        .arg(
                            Arg::new("tests")
                                .value_name("TESTS")
                                .num_args(0..)
                                .help("Test paths or patterns to run (default: tests/)"),
                        )
                        .arg(
                            Arg::new("addopt")
                                .long("addopt")
                                .action(ArgAction::Append)
                                .value_name("OPT")
                                .help("Additional pytest option (repeatable)"),
                        ),
                ),
        )
}

/// Execute python commands
pub async fn execute(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    match matches.subcommand() {
        Some(("venv", sub)) => execute_venv(sub, executor).await,
        Some(("pip", sub)) => execute_pip(sub, executor).await,
        Some(("run", sub)) => execute_run(sub, executor).await,
        _ => {
            println!("Use one of the subcommands: venv | pip | run. See --help for details.");
            Ok(())
        }
    }
}

async fn execute_venv(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    match matches.subcommand() {
        Some(("ensure", m)) => {
            let venv_path = m.get_one::<String>("path").unwrap().to_string();
            let python = m.get_one::<String>("python").unwrap().to_string();
            let recreate = m.get_flag("recreate");
            let prompt = m.get_one::<String>("prompt").cloned();

            let venv_dir = PathBuf::from(&venv_path);
            if venv_dir.exists() {
                if recreate {
                    guard_remove_venv(&venv_dir, true)?;
                } else {
                    info!("Venv already exists: {}", venv_dir.display());
                    return Ok(());
                }
            }

            let mut args: Vec<String> = vec!["-m".into(), "venv".into()];
            if let Some(p) = prompt.as_deref() {
                args.push(format!("--prompt={}", p));
            }
            args.push(venv_path.clone());

            executor.execute_secure(&python, &args).await?;
            info!("Created venv at {}", venv_dir.display());
            Ok(())
        }
        Some(("remove", m)) => {
            let venv_path = m.get_one::<String>("path").unwrap().to_string();
            let force = m.get_flag("force");
            let venv_dir = PathBuf::from(&venv_path);
            guard_remove_venv(&venv_dir, force)?;
            Ok(())
        }
        _ => {
            println!("Use: venv ensure|remove --help");
            Ok(())
        }
    }
}

async fn execute_pip(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    match matches.subcommand() {
        Some(("install", m)) => {
            let venv_path = m.get_one::<String>("path").unwrap().to_string();
            let allow_global = m.get_flag("allow-global");
            let venv_python = resolve_venv_python(&venv_path);
            let using_venv = venv_python.exists();

            if !using_venv && !allow_global {
                anyhow::bail!(
                    "No venv found at {}. Run 'python venv ensure --path {}' or use --allow-global (unsafe).",
                    venv_path,
                    venv_path
                );
            }

            // Base python to use (venv or system)
            let py = if using_venv {
                venv_python.to_string_lossy().to_string()
            } else {
                default_python()
            };

            if m.get_flag("upgrade-pip") {
                executor
                    .execute_secure(&py, &["-m", "pip", "install", "--upgrade", "pip"])
                    .await?;
            }

            if let Some(req) = m.get_one::<String>("requirements") {
                if !Path::new(req).exists() {
                    anyhow::bail!("requirements file not found: {}", req);
                }
                let args = vec![
                    "-m".to_string(),
                    "pip".to_string(),
                    "install".to_string(),
                    "-r".to_string(),
                    req.to_string(),
                ];
                executor.execute_secure(&py, &args).await?;
            }

            if let Some(pkgs) = m.get_many::<String>("package") {
                let mut args: Vec<String> = vec!["-m".into(), "pip".into(), "install".into()];
                for p in pkgs {
                    args.push(p.to_string());
                }
                executor.execute_secure(&py, &args).await?;
            }
            Ok(())
        }
        _ => {
            println!("Use: pip install --help");
            Ok(())
        }
    }
}

async fn execute_run(matches: &ArgMatches, executor: &Executor) -> Result<()> {
    match matches.subcommand() {
        Some(("pytest", m)) => {
            let venv_path = m.get_one::<String>("path").unwrap().to_string();
            let py_path = resolve_venv_python(&venv_path);
            if !py_path.exists() {
                anyhow::bail!(
                    "Venv python not found at {}. Run 'python venv ensure --path {}'.",
                    py_path.display(),
                    venv_path
                );
            }

            let mut args: Vec<String> = vec!["-m".into(), "pytest".into()];

            if let Some(opts) = m.get_many::<String>("addopt") {
                for o in opts {
                    args.push(o.to_string());
                }
            }

            if let Some(tests) = m.get_many::<String>("tests") {
                for t in tests {
                    args.push(t.to_string());
                }
            } else {
                args.push("tests/".into());
            }

            let py_cmd = py_path.to_string_lossy().to_string();
            executor.execute_secure(&py_cmd, &args).await?;
            Ok(())
        }
        _ => {
            println!("Use: run pytest --help");
            Ok(())
        }
    }
}

fn default_python() -> String {
    // Prefer python3, fallback to python
    if which::which("python3").is_ok() {
        "python3".to_string()
    } else {
        "python".to_string()
    }
}

fn venv_bin_dir(venv_path: &Path) -> PathBuf {
    let mut p = PathBuf::from(venv_path);
    if cfg!(target_os = "windows") {
        p.push("Scripts");
    } else {
        p.push("bin");
    }
    p
}

fn resolve_venv_python(venv: &str) -> PathBuf {
    let bin = venv_bin_dir(Path::new(venv));
    let mut py = bin.clone();
    if cfg!(target_os = "windows") {
        py.push("python.exe");
    } else {
        // Prefer python, but handle platforms where only python3 exists
        let candidate = bin.join("python");
        if candidate.exists() {
            py = candidate;
        } else {
            py = bin.join("python3");
        }
    }
    py
}

fn guard_remove_venv(venv_dir: &Path, force: bool) -> Result<()> {
    if !venv_dir.exists() {
        info!("Venv not found (nothing to remove): {}", venv_dir.display());
        return Ok(());
    }

    // Safety: require the directory to be named .venv unless --force
    if let Some(name) = venv_dir.file_name().and_then(|s| s.to_str()) {
        if name != ".venv" && !force {
            anyhow::bail!(
                "Refusing to remove directory not named '.venv': {} (use --force to override)",
                venv_dir.display()
            );
        }
    }

    warn!("Removing venv directory: {}", venv_dir.display());
    fs::remove_dir_all(venv_dir)?;
    Ok(())
}
