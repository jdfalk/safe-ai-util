// file: src/logger.rs
// version: 1.3.1
// guid: 5a9fbb43-1e0b-4bea-a858-b74b58176503
// last-edited: 2026-10-04

use crate::error::Result;
use chrono;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Setup logging for the application.
///
/// Behavior is governed by environment variables so callers (worktrees,
/// MCP subprocess, headless agents) can keep noise contained:
///
///   * `SAFE_AI_UTIL_QUIET=1` — disable file logging entirely; only emit a
///     minimal stderr layer at warn-level so accidental writes to stdout
///     don't corrupt MCP/JSON pipelines.
///   * `SAFE_AI_UTIL_LOG_DIR=<path>` — directory for the per-invocation log
///     file. Defaults to `./logs/` (legacy behavior) when unset.
///   * `RUST_LOG=<filter>` — standard tracing filter, applied to both layers.
pub fn setup_logging() -> Result<()> {
    if env::var("SAFE_AI_UTIL_QUIET")
        .map(|v| v != "0" && !v.is_empty())
        .unwrap_or(false)
    {
        return setup_quiet_logging();
    }

    let logs_dir = log_dir_from_env();
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir)?;
    }

    let now = chrono::Utc::now();
    let log_filename = logs_dir.join(format!("safe-ai-util-{}.log", now.format("%Y%m%d_%H%M%S")));

    let filter_stdout = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    let filter_file = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_filename)?;

    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_writer(io::stdout)
        .with_filter(filter_stdout);

    let file_layer = fmt::layer()
        .with_target(false)
        .with_ansi(false)
        .with_writer(file)
        .with_filter(filter_file);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    tracing::info!(
        "Logging initialized - writing to stdout and {}",
        log_filename.display()
    );

    Ok(())
}

/// Headless logging: warn+ to stderr, no file, no stdout pollution.
fn setup_quiet_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn"))
        .unwrap();
    let stderr_layer = fmt::layer()
        .with_target(false)
        .with_writer(io::stderr)
        .with_filter(filter);
    tracing_subscriber::registry().with(stderr_layer).init();
    Ok(())
}

/// Resolve the log directory from env vars, falling back to legacy `./logs/`.
fn log_dir_from_env() -> PathBuf {
    if let Ok(dir) = env::var("SAFE_AI_UTIL_LOG_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    PathBuf::from("logs")
}
