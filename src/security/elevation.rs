// file: src/security/elevation.rs
// version: 1.0.0
// guid: 4f0c2b7e-8a61-4d3e-9b5a-2c7d1e6f9a03
// last-edited: 2026-10-04

//! Elevation detection and the elevated-mode CLI gate.
//!
//! safe-ai-util is meant to be usable as the only command in a sudoers rule.
//! Once it runs as root, everything the caller controls (flags, config files
//! in the working directory or `$HOME`, environment variables) must stop
//! influencing policy. This module decides *whether* the process is elevated
//! and *what* an elevated invocation may ask for.
//!
//! The context is a plain value so tests can construct an elevated context
//! without real root. Production code builds it only through
//! [`ElevationContext::detect`]; there is deliberately no flag or environment
//! variable that can make a process count as *less* elevated.

use crate::error::{AgentError, Result};

/// Facts about the process's privilege, captured once at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElevationContext {
    /// Effective uid.
    pub euid: u32,
    /// Real uid.
    pub ruid: u32,
    /// `SUDO_USER`, when set.
    pub sudo_user: Option<String>,
    /// `SUDO_UID`, when set.
    pub sudo_uid: Option<String>,
}

impl ElevationContext {
    /// Capture the real process context.
    pub fn detect() -> Self {
        let (euid, ruid) = current_uids();
        Self {
            euid,
            ruid,
            sudo_user: non_empty_env("SUDO_USER"),
            sudo_uid: non_empty_env("SUDO_UID"),
        }
    }

    /// A context for an ordinary unprivileged user (tests and docs).
    pub fn unprivileged(uid: u32) -> Self {
        Self {
            euid: uid,
            ruid: uid,
            sudo_user: None,
            sudo_uid: None,
        }
    }

    /// A context that looks like `sudo safe-ai-util ...` run by `user`
    /// (tests and docs).
    pub fn sudo_root(user: &str) -> Self {
        Self {
            euid: 0,
            ruid: 0,
            sudo_user: Some(user.to_string()),
            sudo_uid: Some("1000".to_string()),
        }
    }

    /// True when the process must run under the fixed root policy.
    ///
    /// Any one of these is enough: euid 0, euid differing from the real uid
    /// (a setuid install), or `SUDO_USER`/`SUDO_UID` present (running under
    /// sudo, including `sudo -u someone`).
    pub fn is_elevated(&self) -> bool {
        self.euid == 0
            || self.euid != self.ruid
            || self.sudo_user.is_some()
            || self.sudo_uid.is_some()
    }

    /// Short human-readable description for logs and error messages.
    pub fn describe(&self) -> String {
        format!(
            "euid={} ruid={} sudo_user={}",
            self.euid,
            self.ruid,
            self.sudo_user.as_deref().unwrap_or("-")
        )
    }
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

#[cfg(unix)]
fn current_uids() -> (u32, u32) {
    // SAFETY: geteuid/getuid take no arguments, cannot fail, and touch no
    // memory owned by Rust.
    unsafe { (libc::geteuid(), libc::getuid()) }
}

#[cfg(not(unix))]
fn current_uids() -> (u32, u32) {
    // No uid model; elevation is then detected only through the sudo
    // variables, and `exec` refuses to run on non-unix targets.
    (u32::MAX, u32::MAX)
}

/// What the caller asked for on the command line, reduced to the parts the
/// elevated gate cares about.
#[derive(Debug, Clone, Default)]
pub struct CliRequest<'a> {
    pub config: Option<&'a str>,
    pub policy_overlay: Option<&'a str>,
    pub args_file: Option<&'a str>,
    pub subcommand: Option<&'a str>,
}

/// The only subcommand an elevated invocation may use.
pub const ELEVATED_SUBCOMMAND: &str = "exec";

/// Refuse anything an elevated invocation must not be allowed to ask for.
///
/// When not elevated this always succeeds, so unprivileged behaviour is
/// unchanged. When elevated, the checks run in a fixed order so every refusal
/// has a distinct reason and none of them touches the filesystem:
///
/// 1. `--config` (a caller-chosen policy file)
/// 2. `--policy-overlay`
/// 3. `--args-file` (caller-chosen extra argv, read as root)
/// 4. any subcommand other than `exec` (the other subcommands read user
///    config, honour env-var paths, and several run tools via PATH)
pub fn check_cli_request(ctx: &ElevationContext, req: &CliRequest<'_>) -> Result<()> {
    if !ctx.is_elevated() {
        return Ok(());
    }
    if req.config.is_some() {
        return Err(AgentError::security(
            "--config is refused when elevated; the policy is read only from the fixed root-owned path",
        ));
    }
    if req.policy_overlay.is_some() {
        return Err(AgentError::security(
            "--policy-overlay is refused when elevated",
        ));
    }
    if req.args_file.is_some() {
        return Err(AgentError::security("--args-file is refused when elevated"));
    }
    match req.subcommand {
        Some(ELEVATED_SUBCOMMAND) => Ok(()),
        Some(other) => Err(AgentError::security(format!(
            "subcommand '{}' is refused when elevated; only '{}' may run",
            other, ELEVATED_SUBCOMMAND
        ))),
        None => Err(AgentError::security(format!(
            "no subcommand given; only '{}' may run when elevated",
            ELEVATED_SUBCOMMAND
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_signals() {
        assert!(!ElevationContext::unprivileged(1000).is_elevated());
        assert!(ElevationContext::sudo_root("jdfalk").is_elevated());

        let mut c = ElevationContext::unprivileged(1000);
        c.euid = 0;
        assert!(c.is_elevated(), "euid 0 alone is elevated");

        let mut c = ElevationContext::unprivileged(1000);
        c.euid = 1001;
        assert!(c.is_elevated(), "setuid (euid != ruid) is elevated");

        let mut c = ElevationContext::unprivileged(1000);
        c.sudo_user = Some("x".into());
        assert!(c.is_elevated(), "SUDO_USER alone is elevated");

        let mut c = ElevationContext::unprivileged(1000);
        c.sudo_uid = Some("1000".into());
        assert!(c.is_elevated(), "SUDO_UID alone is elevated");
    }

    #[test]
    fn unprivileged_requests_pass_untouched() {
        let ctx = ElevationContext::unprivileged(1000);
        let req = CliRequest {
            config: Some("/tmp/x.toml"),
            policy_overlay: Some("/tmp/o.toml"),
            args_file: Some("/tmp/a"),
            subcommand: Some("git"),
        };
        assert!(check_cli_request(&ctx, &req).is_ok());
    }

    #[test]
    fn elevated_refuses_config() {
        let ctx = ElevationContext::sudo_root("u");
        let req = CliRequest {
            config: Some("/tmp/permissive.toml"),
            subcommand: Some("exec"),
            ..Default::default()
        };
        let err = check_cli_request(&ctx, &req).unwrap_err().to_string();
        assert!(err.contains("--config"), "{err}");
    }

    #[test]
    fn elevated_refuses_overlay() {
        let ctx = ElevationContext::sudo_root("u");
        let req = CliRequest {
            policy_overlay: Some("/tmp/o.toml"),
            subcommand: Some("exec"),
            ..Default::default()
        };
        let err = check_cli_request(&ctx, &req).unwrap_err().to_string();
        assert!(err.contains("--policy-overlay"), "{err}");
    }

    #[test]
    fn elevated_refuses_args_file() {
        let ctx = ElevationContext::sudo_root("u");
        let req = CliRequest {
            args_file: Some("/tmp/a"),
            subcommand: Some("exec"),
            ..Default::default()
        };
        let err = check_cli_request(&ctx, &req).unwrap_err().to_string();
        assert!(err.contains("--args-file"), "{err}");
    }

    #[test]
    fn elevated_refuses_other_subcommands() {
        let ctx = ElevationContext::sudo_root("u");
        for sub in ["git", "file", "uutils", "python", "editor", "sed", "awk", "system"] {
            let req = CliRequest {
                subcommand: Some(sub),
                ..Default::default()
            };
            let err = check_cli_request(&ctx, &req).unwrap_err().to_string();
            assert!(err.contains("refused when elevated"), "{sub}: {err}");
        }
        assert!(check_cli_request(&ctx, &CliRequest::default()).is_err());
        let ok = CliRequest {
            subcommand: Some("exec"),
            ..Default::default()
        };
        assert!(check_cli_request(&ctx, &ok).is_ok());
    }
}
