// file: src/commands/file.rs
// version: 1.0.1
// guid: fbdd6298-852d-4041-a846-83781ff68a50
// last-edited: 2026-10-04

//! File operations: read, write, glob, list, exists.
//!
//! All operations are gated by the same path policy that protects sensitive
//! system locations (/etc, /bin, /sys, ...). When the env var
//! `SAFE_AI_UTIL_REPO_ROOT` is set, every path must canonicalize to a location
//! inside that root — this is how the burndown driver sandboxes agents to a
//! single worktree.
//!
//! Read and write have hard byte ceilings to defend against accidental
//! resource exhaustion. They are configurable via env vars but conservatively
//! defaulted.

use crate::executor::Executor;
use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgMatches, Command};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

const DEFAULT_MAX_READ_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB
const DEFAULT_MAX_WRITE_BYTES: u64 = 5 * 1024 * 1024; // 5 MiB
const DEFAULT_MAX_GLOB_RESULTS: usize = 5_000;

/// Build the file command tree.
pub fn build_command() -> Command {
    Command::new("file")
        .about("File operations (read/write/glob/list/exists), audited and policy-gated")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("read")
                .about("Read a file to stdout")
                .arg(
                    Arg::new("path")
                        .long("path")
                        .help("File to read")
                        .required(true),
                )
                .arg(
                    Arg::new("max-bytes")
                        .long("max-bytes")
                        .help("Maximum bytes to read (default 10 MiB)")
                        .value_parser(clap::value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("write")
                .about("Write content to a file (creates or overwrites)")
                .arg(
                    Arg::new("path")
                        .long("path")
                        .help("File to write")
                        .required(true),
                )
                .arg(
                    Arg::new("content")
                        .long("content")
                        .help("Inline content (mutually exclusive with --content-stdin)")
                        .conflicts_with("content-stdin"),
                )
                .arg(
                    Arg::new("content-stdin")
                        .long("content-stdin")
                        .help("Read content from stdin (use for any non-trivial size)")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("create-dirs")
                        .long("create-dirs")
                        .help("Create parent directories if missing")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("max-bytes")
                        .long("max-bytes")
                        .help("Maximum bytes to write (default 5 MiB)")
                        .value_parser(clap::value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("glob")
                .about("Match files against a glob pattern, one per line")
                .arg(
                    Arg::new("pattern")
                        .long("pattern")
                        .help("Glob pattern, e.g. 'src/**/*.rs'")
                        .required(true),
                )
                .arg(
                    Arg::new("max-results")
                        .long("max-results")
                        .help("Maximum results to return (default 5000)")
                        .value_parser(clap::value_parser!(usize)),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List directory entries (non-recursive), one per line")
                .arg(
                    Arg::new("path")
                        .long("path")
                        .help("Directory to list")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("exists")
                .about("Exit 0 if path exists, 1 otherwise")
                .arg(
                    Arg::new("path")
                        .long("path")
                        .help("Path to test")
                        .required(true),
                ),
        )
}

/// Dispatch a `file` invocation.
pub async fn execute(matches: &ArgMatches, _executor: &Executor) -> Result<()> {
    match matches.subcommand() {
        Some(("read", m)) => exec_read(m),
        Some(("write", m)) => exec_write(m),
        Some(("glob", m)) => exec_glob(m),
        Some(("list", m)) => exec_list(m),
        Some(("exists", m)) => exec_exists(m),
        _ => Err(anyhow!("file: missing or unknown subcommand")),
    }
}

// ---------------------------------------------------------------------------
// subcommand impls
// ---------------------------------------------------------------------------

fn exec_read(m: &ArgMatches) -> Result<()> {
    let path_str = m.get_one::<String>("path").expect("required");
    let max = m
        .get_one::<u64>("max-bytes")
        .copied()
        .unwrap_or_else(read_max_bytes);

    let path = validate_path(path_str, PathIntent::Read)?;
    let meta = fs::metadata(&path)
        .with_context(|| format!("file read: cannot stat '{}'", path.display()))?;
    if !meta.is_file() {
        return Err(anyhow!("file read: not a regular file: {}", path.display()));
    }
    if meta.len() > max {
        return Err(anyhow!(
            "file read: '{}' is {} bytes, exceeds limit of {} bytes",
            path.display(),
            meta.len(),
            max
        ));
    }

    let bytes = fs::read(&path)
        .with_context(|| format!("file read: failed to read '{}'", path.display()))?;
    io::stdout().write_all(&bytes)?;
    info!("file read: {} ({} bytes)", path.display(), bytes.len());
    Ok(())
}

fn exec_write(m: &ArgMatches) -> Result<()> {
    let path_str = m.get_one::<String>("path").expect("required");
    let create_dirs = m.get_flag("create-dirs");
    let max = m
        .get_one::<u64>("max-bytes")
        .copied()
        .unwrap_or_else(write_max_bytes);

    let path = validate_path(path_str, PathIntent::Write)?;

    let inline = m.get_one::<String>("content").cloned();
    let from_stdin = m.get_flag("content-stdin");

    let content: Vec<u8> = match (inline, from_stdin) {
        (Some(s), false) => s.into_bytes(),
        (None, true) => {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;
            buf
        }
        (None, false) => {
            return Err(anyhow!(
                "file write: must supply --content or --content-stdin"
            ));
        }
        (Some(_), true) => unreachable!("clap conflicts_with prevents this"),
    };

    if content.len() as u64 > max {
        return Err(anyhow!(
            "file write: payload {} bytes exceeds limit {} bytes",
            content.len(),
            max
        ));
    }

    if create_dirs {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("file write: cannot create parent of '{}'", path.display())
                })?;
            }
        }
    }

    fs::write(&path, &content)
        .with_context(|| format!("file write: failed to write '{}'", path.display()))?;
    info!("file write: {} ({} bytes)", path.display(), content.len());
    Ok(())
}

fn exec_glob(m: &ArgMatches) -> Result<()> {
    let pattern = m.get_one::<String>("pattern").expect("required");
    let max = m
        .get_one::<usize>("max-results")
        .copied()
        .unwrap_or(DEFAULT_MAX_GLOB_RESULTS);

    let resolved_pattern = resolve_glob_pattern(pattern)?;
    debug!("file glob: resolved pattern = {}", resolved_pattern);

    let mut count = 0usize;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for entry in glob::glob(&resolved_pattern)
        .with_context(|| format!("file glob: bad pattern '{}'", resolved_pattern))?
    {
        let p = entry.with_context(|| "file glob: entry error")?;
        if validate_path(&p.to_string_lossy(), PathIntent::Read).is_err() {
            warn!("file glob: skipping out-of-policy path {}", p.display());
            continue;
        }
        writeln!(out, "{}", p.display())?;
        count += 1;
        if count >= max {
            warn!("file glob: hit max-results cap of {}", max);
            break;
        }
    }
    info!("file glob: pattern='{}' results={}", pattern, count);
    Ok(())
}

fn exec_list(m: &ArgMatches) -> Result<()> {
    let path_str = m.get_one::<String>("path").expect("required");
    let path = validate_path(path_str, PathIntent::Read)?;
    let meta = fs::metadata(&path)
        .with_context(|| format!("file list: cannot stat '{}'", path.display()))?;
    if !meta.is_dir() {
        return Err(anyhow!("file list: not a directory: {}", path.display()));
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    for entry in fs::read_dir(&path)
        .with_context(|| format!("file list: failed to read '{}'", path.display()))?
    {
        let entry = entry?;
        writeln!(out, "{}", entry.file_name().to_string_lossy())?;
    }
    Ok(())
}

fn exec_exists(m: &ArgMatches) -> Result<()> {
    let path_str = m.get_one::<String>("path").expect("required");
    let path = PathBuf::from(path_str);
    if path.exists() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// path policy
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PathIntent {
    Read,
    Write,
}

fn validate_path(path_str: &str, intent: PathIntent) -> Result<PathBuf> {
    if path_str.is_empty() {
        return Err(anyhow!("file: empty path"));
    }

    let raw = Path::new(path_str);

    if raw.is_absolute() {
        let s = raw.to_string_lossy();
        const SENSITIVE: &[&str] = &[
            "/etc",
            "/bin",
            "/sbin",
            "/usr/bin",
            "/usr/sbin",
            "/boot",
            "/root",
            "/sys",
            "/proc",
            "/dev",
        ];
        for prefix in SENSITIVE {
            if s.starts_with(prefix) {
                return Err(anyhow!(
                    "file: access to sensitive path '{}' denied by policy",
                    s
                ));
            }
        }
    }

    let resolved = if let Ok(root_str) = env::var("SAFE_AI_UTIL_REPO_ROOT") {
        let root = PathBuf::from(&root_str);
        let root_canon = fs::canonicalize(&root).with_context(|| {
            format!(
                "file: SAFE_AI_UTIL_REPO_ROOT '{}' is not accessible",
                root_str
            )
        })?;

        let candidate = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            root_canon.join(raw)
        };

        let canon = match intent {
            PathIntent::Read => fs::canonicalize(&candidate).with_context(|| {
                format!(
                    "file: cannot resolve '{}' under repo root",
                    candidate.display()
                )
            })?,
            PathIntent::Write => canonicalize_for_write(&candidate)?,
        };

        if !canon.starts_with(&root_canon) {
            return Err(anyhow!(
                "file: path '{}' escapes repo root '{}'",
                canon.display(),
                root_canon.display()
            ));
        }
        canon
    } else {
        raw.to_path_buf()
    };

    Ok(resolved)
}

fn canonicalize_for_write(p: &Path) -> Result<PathBuf> {
    if p.exists() {
        return Ok(fs::canonicalize(p)?);
    }
    let mut probe = p
        .parent()
        .ok_or_else(|| {
            anyhow!(
                "file write: target '{}' has no parent component",
                p.display()
            )
        })?
        .to_path_buf();
    let mut suffix: Vec<std::ffi::OsString> = Vec::new();
    loop {
        if probe.as_os_str().is_empty() {
            probe = env::current_dir()?;
        }
        if probe.exists() {
            let mut canon = fs::canonicalize(&probe)?;
            for part in suffix.iter().rev() {
                canon.push(part);
            }
            canon.push(p.file_name().expect("checked parent above"));
            return Ok(canon);
        }
        let comp = probe
            .file_name()
            .ok_or_else(|| {
                anyhow!(
                    "file write: unable to resolve any ancestor of '{}'",
                    p.display()
                )
            })?
            .to_owned();
        suffix.push(comp);
        if !probe.pop() {
            return Err(anyhow!(
                "file write: unable to resolve any ancestor of '{}'",
                p.display()
            ));
        }
    }
}

fn resolve_glob_pattern(pattern: &str) -> Result<String> {
    if Path::new(pattern).is_absolute() {
        return Ok(pattern.to_string());
    }
    if let Ok(root) = env::var("SAFE_AI_UTIL_REPO_ROOT") {
        let root_path = PathBuf::from(&root);
        return Ok(root_path.join(pattern).to_string_lossy().into_owned());
    }
    Ok(pattern.to_string())
}

// ---------------------------------------------------------------------------
// env-var-driven config
// ---------------------------------------------------------------------------

fn read_max_bytes() -> u64 {
    env::var("SAFE_AI_UTIL_MAX_READ_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_READ_BYTES)
}

fn write_max_bytes() -> u64 {
    env::var("SAFE_AI_UTIL_MAX_WRITE_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_WRITE_BYTES)
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_repo_root<F: FnOnce(&Path)>(td: &TempDir, f: F) {
        let _g = ENV_LOCK.lock().unwrap();
        let prev = env::var("SAFE_AI_UTIL_REPO_ROOT").ok();
        env::set_var("SAFE_AI_UTIL_REPO_ROOT", td.path());
        f(td.path());
        match prev {
            Some(v) => env::set_var("SAFE_AI_UTIL_REPO_ROOT", v),
            None => env::remove_var("SAFE_AI_UTIL_REPO_ROOT"),
        }
    }

    #[test]
    fn rejects_sensitive_absolute_paths() {
        let _g = ENV_LOCK.lock().unwrap();
        env::remove_var("SAFE_AI_UTIL_REPO_ROOT");
        for p in &["/etc/passwd", "/bin/sh", "/sys/kernel", "/proc/1/mem"] {
            let err = validate_path(p, PathIntent::Read).unwrap_err();
            assert!(
                err.to_string().contains("sensitive path"),
                "expected sensitive-path rejection for {p}, got: {err}"
            );
        }
    }

    #[test]
    fn allows_paths_inside_repo_root() {
        let td = TempDir::new().unwrap();
        let inner = td.path().join("inner.txt");
        fs::write(&inner, "hi").unwrap();
        with_repo_root(&td, |_| {
            let resolved = validate_path("inner.txt", PathIntent::Read).unwrap();
            assert!(resolved.ends_with("inner.txt"));
        });
    }

    #[test]
    fn rejects_paths_escaping_repo_root() {
        let td = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let outside_file = outside.path().join("evil.txt");
        fs::write(&outside_file, "x").unwrap();
        with_repo_root(&td, |_| {
            let err = validate_path(outside_file.to_str().unwrap(), PathIntent::Read).unwrap_err();
            assert!(err.to_string().contains("escapes repo root"));
        });
    }

    #[test]
    fn write_to_new_file_under_root_resolves() {
        let td = TempDir::new().unwrap();
        with_repo_root(&td, |_| {
            let resolved = validate_path("new.txt", PathIntent::Write).unwrap();
            assert!(resolved.ends_with("new.txt"));
        });
    }

    #[test]
    fn write_to_new_file_in_new_subdir_resolves() {
        let td = TempDir::new().unwrap();
        with_repo_root(&td, |_| {
            let resolved = validate_path("sub/dir/file.txt", PathIntent::Write).unwrap();
            assert!(resolved.to_string_lossy().contains("file.txt"));
        });
    }

    #[test]
    fn glob_pattern_resolved_relative_to_root() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join("a.txt"), "a").unwrap();
        with_repo_root(&td, |root| {
            let resolved = resolve_glob_pattern("*.txt").unwrap();
            assert!(resolved.starts_with(root.to_str().unwrap()));
        });
    }
}
