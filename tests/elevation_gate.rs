// file: tests/elevation_gate.rs
// version: 1.0.0
// guid: 6e2b9d47-1f8a-4c3e-a5d0-8b7c2e4f1a96
// last-edited: 2026-10-04

//! End-to-end checks of the elevated-mode gate through the real binary.
//!
//! Elevation is simulated with `SUDO_USER`/`SUDO_UID`: the gate treats either
//! as elevated, and elevation can only tighten behaviour, so these tests need
//! no root. A test that needs a non-elevated process skips itself when the
//! suite is run as root.

#![cfg(unix)]

use assert_cmd::Command;
use predicates::prelude::*;

const EXIT_REFUSED: i32 = 77;

fn as_root() -> bool {
    // SAFETY: no arguments, cannot fail.
    unsafe { libc::geteuid() == 0 }
}

fn sudo_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("safe-ai-util").unwrap();
    cmd.current_dir(dir)
        .env("SUDO_USER", "tester")
        .env("SUDO_UID", "1000")
        // If any of these were honoured, the elevated run would write here.
        .env("SAFE_AI_UTIL_LOG_DIR", dir.join("evil-logs"))
        .env("SAFE_AI_UTIL_AUDIT_PATH", dir.join("evil-audit"))
        .env("COPILOT_AUDIT_DIR", dir.join("evil-audit2"));
    cmd
}

fn user_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("safe-ai-util").unwrap();
    cmd.current_dir(dir)
        .env_remove("SUDO_USER")
        .env_remove("SUDO_UID")
        .env("SAFE_AI_UTIL_QUIET", "1");
    cmd
}

fn assert_nothing_written(dir: &std::path::Path) {
    for name in [
        "logs",
        "evil-logs",
        "evil-audit",
        "evil-audit2",
        ".copilot-audit",
    ] {
        assert!(
            !dir.join(name).exists(),
            "elevated run created {} in a caller-chosen location",
            name
        );
    }
}

#[test]
fn elevated_refuses_config_flag() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(
        d.path().join("permissive.toml"),
        "[allowlist]\npermissive_mode = true\n",
    )
    .unwrap();
    sudo_cmd(d.path())
        .args([
            "--config",
            "permissive.toml",
            "exec",
            "zfs",
            "destroy",
            "bigdata/rehearsal-sandbox-data",
        ])
        .assert()
        .code(EXIT_REFUSED)
        .stderr(predicate::str::contains(
            "--config is refused when elevated",
        ));
    assert_nothing_written(d.path());
}

#[test]
fn elevated_refuses_policy_overlay() {
    let d = tempfile::tempdir().unwrap();
    sudo_cmd(d.path())
        .args(["--policy-overlay", "o.toml", "exec", "zfs", "list"])
        .assert()
        .code(EXIT_REFUSED)
        .stderr(predicate::str::contains("--policy-overlay is refused"));
    assert_nothing_written(d.path());
}

#[test]
fn elevated_refuses_args_file() {
    let d = tempfile::tempdir().unwrap();
    sudo_cmd(d.path())
        .args(["--args-file", "a.txt", "exec", "zfs", "list"])
        .assert()
        .code(EXIT_REFUSED)
        .stderr(predicate::str::contains("--args-file is refused"));
    assert_nothing_written(d.path());
}

#[test]
fn elevated_refuses_every_other_subcommand() {
    let d = tempfile::tempdir().unwrap();
    // A permissive project config in the caller's cwd must not matter.
    std::fs::write(
        d.path().join(".safe-ai-util.toml"),
        "[allowlist]\npermissive_mode = true\n",
    )
    .unwrap();
    for args in [
        &["git", "status"][..],
        &["uutils", "ls"],
        &["file", "read", "--path", "/etc/shadow"],
        &["system"],
    ] {
        sudo_cmd(d.path())
            .args(args)
            .assert()
            .code(EXIT_REFUSED)
            .stderr(predicate::str::contains("refused when elevated"));
    }
    sudo_cmd(d.path())
        .assert()
        .code(EXIT_REFUSED)
        .stderr(predicate::str::contains("no subcommand"));
    assert_nothing_written(d.path());
}

#[test]
fn elevated_exec_never_uses_cwd_config() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(
        d.path().join(".safe-ai-util.toml"),
        "[allowlist]\npermissive_mode = true\n",
    )
    .unwrap();
    // Without a valid root-owned /etc/safe-ai-util/root-policy.toml this is
    // EX_CONFIG (78); with one, an argv outside it is EX_NOPERM (77). It must
    // never succeed and must name the fixed policy path, not the cwd file.
    let out = sudo_cmd(d.path())
        .args([
            "exec",
            "zfs",
            "destroy",
            "bigdata/rehearsal-sandbox-data",
            "-r",
        ])
        .output()
        .unwrap();
    let code = out.status.code();
    assert!(
        code == Some(77) || code == Some(78),
        "unexpected exit {code:?}"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("REFUSED"), "{err}");
    assert!(!err.contains(".safe-ai-util.toml"), "{err}");
    assert_nothing_written(d.path());
}

#[test]
fn exec_captures_hyphen_args_verbatim() {
    if as_root() {
        return;
    }
    let d = tempfile::tempdir().unwrap();
    // clap must hand `-r`, `-R` and `--` through to the policy check rather
    // than rejecting them as unknown flags (clap usage errors exit 2).
    for args in [
        &[
            "--dry-run",
            "exec",
            "zfs",
            "destroy",
            "bigdata/rehearsal-sandbox-data",
            "-r",
        ][..],
        &["--dry-run", "exec", "zfs", "destroy", "-R", "x"],
        &["--dry-run", "exec", "--", "zfs", "destroy", "x"],
    ] {
        let out = user_cmd(d.path()).args(args).output().unwrap();
        assert_ne!(
            out.status.code(),
            Some(2),
            "clap rejected {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_ne!(out.status.code(), Some(0), "{args:?} must not be allowed");
    }
}

#[test]
fn unprivileged_exec_requires_dry_run() {
    if as_root() {
        return;
    }
    let d = tempfile::tempdir().unwrap();
    user_cmd(d.path())
        .args(["exec", "zfs", "destroy", "bigdata/rehearsal-sandbox-data"])
        .assert()
        .code(EXIT_REFUSED)
        .stderr(predicate::str::contains("only under sudo"));
}

#[test]
fn unprivileged_exec_refuses_config_flag() {
    if as_root() {
        return;
    }
    let d = tempfile::tempdir().unwrap();
    user_cmd(d.path())
        .args(["--config", "x.toml", "--dry-run", "exec", "zfs", "list"])
        .assert()
        .code(EXIT_REFUSED)
        .stderr(predicate::str::contains("--config is not accepted by exec"));
}
