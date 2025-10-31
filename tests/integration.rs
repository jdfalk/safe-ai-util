// file: tests/integration.rs
// version: 1.1.0
// guid: 5108e7ff-575f-45b4-aef8-b13db42ab609

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command_safe_ai_util() {
    let mut cmd = Command::cargo_bin("safe-ai-util").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "reliable command execution utility",
        ));
}

#[test]
fn test_version_command_safe_ai_util() {
    let mut cmd = Command::cargo_bin("safe-ai-util").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_help_command_copilot_agent_util() {
    // Test backward compatibility binary name
    let mut cmd = Command::cargo_bin("copilot-agent-util").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "reliable command execution utility",
        ));
}

#[test]
fn test_version_command_copilot_agent_util() {
    // Test backward compatibility binary name
    let mut cmd = Command::cargo_bin("copilot-agent-util").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
