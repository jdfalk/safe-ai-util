// file: src/security/allowlist.rs
// version: 1.1.0
// guid: e5f6a7b8-c9d0-1234-ef56-567890123456
// last-edited: 2026-10-04

//! Command allowlist management module
//!
//! This module manages the allowlist of commands that can be executed by the utility.
//! It provides a centralized location for defining safe commands and their restrictions.

use crate::error::{AgentError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

/// Command allowlist configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowlistConfig {
    /// Commands that are always allowed
    pub always_allowed: HashSet<String>,
    /// Commands that are conditionally allowed based on arguments
    pub conditionally_allowed: HashMap<String, CommandRestrictions>,
    /// Commands that are explicitly blocked
    pub blocked: HashSet<String>,
    /// Whether to allow unknown commands in permissive mode
    pub permissive_mode: bool,
}

/// Restrictions for a specific command.
///
/// Every field has a serde default so existing config files that predate a
/// field keep parsing.
///
/// # Argument allowlisting
///
/// Two positive (allow) rules exist, and they are checked in addition to the
/// negative (deny) rules:
///
/// * `allowed_argv` — the argument vector (everything after the command name)
///   must be *exactly equal* to one of these lists, element for element. This
///   is the strongest form and the only one the shipped root-policy example
///   uses.
/// * `allowed_patterns` — every argument must *fully* match at least one of
///   these regexes. Patterns are anchored automatically (`^(?:p)$`), so `foo`
///   does not admit `foo; rm -rf /`.
///
/// When both are empty the behaviour depends on [`EnforcementMode`]: the
/// legacy (unprivileged) mode treats that as "no positive restriction" so
/// existing configs keep working, while [`EnforcementMode::Elevated`] treats
/// it as deny.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRestrictions {
    /// Maximum number of arguments allowed
    #[serde(default)]
    pub max_args: Option<usize>,
    /// Required arguments that must be present
    #[serde(default)]
    pub required_args: Vec<String>,
    /// Forbidden arguments that must not be present
    #[serde(default)]
    pub forbidden_args: Vec<String>,
    /// Allowed argument patterns (regex, anchored automatically). When
    /// non-empty, every argument must fully match at least one pattern.
    #[serde(default)]
    pub allowed_patterns: Vec<String>,
    /// Exact argument vectors. When non-empty, the arguments must equal one
    /// of these lists exactly (same length, same elements, same order).
    #[serde(default)]
    pub allowed_argv: Vec<Vec<String>>,
    /// Forbidden argument patterns (regex, unanchored)
    #[serde(default)]
    pub forbidden_patterns: Vec<String>,
    /// When true, the command may only run in [`EnforcementMode::Elevated`],
    /// i.e. through the `exec` subcommand under the fixed root-owned policy.
    /// The unprivileged (legacy) validation path refuses it.
    #[serde(default)]
    pub requires_elevation: bool,
    /// Custom validation function name.
    ///
    /// No validator registry exists, so this is not dispatched anywhere. The
    /// elevated mode therefore refuses any rule that sets it rather than
    /// pretend a check ran.
    #[serde(default)]
    pub custom_validator: Option<String>,
}

/// How strictly `AllowlistConfig::validate_command_with_mode` interprets a
/// policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementMode {
    /// Unprivileged use. Empty `allowed_patterns`/`allowed_argv` mean "no
    /// positive restriction" (pre-1.1 behaviour), `always_allowed` and
    /// `permissive_mode` are honoured, and `requires_elevation` rules are
    /// refused.
    Legacy,
    /// Running as root (euid 0) or under sudo. Fail closed: only
    /// `conditionally_allowed` rules with a non-empty `allowed_argv` or
    /// `allowed_patterns` can pass; `always_allowed`, `permissive_mode` and
    /// `custom_validator` are refused because none of them constrain argv.
    Elevated,
}

/// Anchor a user-supplied regex so it must match the whole argument.
fn anchored(pattern: &str) -> String {
    format!("^(?:{})$", pattern)
}

impl Default for AllowlistConfig {
    fn default() -> Self {
        Self::secure_default()
    }
}

impl AllowlistConfig {
    /// Create a secure default allowlist configuration
    pub fn secure_default() -> Self {
        let mut config = Self {
            always_allowed: HashSet::new(),
            conditionally_allowed: HashMap::new(),
            blocked: Self::default_blocked_commands(),
            permissive_mode: false,
        };

        // Add safe commands
        config.add_safe_development_commands();
        config.add_safe_file_commands();
        config.add_conditional_commands();

        config
    }

    /// Get the default set of blocked commands
    pub fn default_blocked_commands() -> HashSet<String> {
        let mut blocked = HashSet::new();

        // Shell and interpreters
        blocked.insert("bash".to_string());
        blocked.insert("sh".to_string());
        blocked.insert("zsh".to_string());
        blocked.insert("fish".to_string());
        blocked.insert("csh".to_string());
        blocked.insert("tcsh".to_string());
        blocked.insert("ksh".to_string());
        blocked.insert("dash".to_string());
        blocked.insert("cmd".to_string());
        blocked.insert("powershell".to_string());
        blocked.insert("pwsh".to_string());

        // Network tools
        blocked.insert("curl".to_string());
        blocked.insert("wget".to_string());
        blocked.insert("nc".to_string());
        blocked.insert("netcat".to_string());
        blocked.insert("ncat".to_string());
        blocked.insert("socat".to_string());
        blocked.insert("telnet".to_string());
        blocked.insert("ftp".to_string());
        blocked.insert("sftp".to_string());
        blocked.insert("scp".to_string());
        blocked.insert("rsync".to_string());

        // System administration
        blocked.insert("sudo".to_string());
        blocked.insert("su".to_string());
        blocked.insert("doas".to_string());
        blocked.insert("pkexec".to_string());
        blocked.insert("systemctl".to_string());
        blocked.insert("service".to_string());
        blocked.insert("chroot".to_string());
        blocked.insert("mount".to_string());
        blocked.insert("umount".to_string());
        blocked.insert("fdisk".to_string());
        blocked.insert("mkfs".to_string());
        blocked.insert("fsck".to_string());

        // Process and system control
        blocked.insert("kill".to_string());
        blocked.insert("killall".to_string());
        blocked.insert("pkill".to_string());
        blocked.insert("reboot".to_string());
        blocked.insert("shutdown".to_string());
        blocked.insert("halt".to_string());
        blocked.insert("init".to_string());

        // Package managers (system-wide)
        blocked.insert("apt".to_string());
        blocked.insert("apt-get".to_string());
        blocked.insert("yum".to_string());
        blocked.insert("dnf".to_string());
        blocked.insert("pacman".to_string());
        blocked.insert("zypper".to_string());
        blocked.insert("emerge".to_string());
        blocked.insert("portage".to_string());
        blocked.insert("brew".to_string()); // Can be dangerous if system-wide

        // Database tools
        blocked.insert("mysql".to_string());
        blocked.insert("psql".to_string());
        blocked.insert("mongo".to_string());
        blocked.insert("redis-cli".to_string());
        blocked.insert("sqlite3".to_string());

        // Compilers and interpreters that can execute arbitrary code
        blocked.insert("gcc".to_string());
        blocked.insert("clang".to_string());
        blocked.insert("cc".to_string());
        blocked.insert("ld".to_string());
        blocked.insert("as".to_string());
        blocked.insert("nasm".to_string());
        blocked.insert("perl".to_string());
        blocked.insert("ruby".to_string());
        blocked.insert("php".to_string());
        blocked.insert("lua".to_string());
        blocked.insert("tcl".to_string());

        // Text editors that can execute commands
        blocked.insert("vim".to_string());
        blocked.insert("vi".to_string());
        blocked.insert("emacs".to_string());
        blocked.insert("nano".to_string()); // Usually safe, but being conservative

        blocked
    }

    /// Add safe development commands to the allowlist
    fn add_safe_development_commands(&mut self) {
        let safe_commands = [
            "git",
            "buf",
            "cargo",
            "rustc",
            "rustfmt",
            "clippy",
            "go",
            "gofmt",
            "goimports",
            "golint",
            "node",
            "npm",
            "yarn",
            "pnpm",
            "npx",
            "python",
            "python3",
            "pip",
            "pip3",
            "make",
            "cmake",
            "ninja",
            "mvn",
            "gradle",
            "sbt",
            "dotnet",
            "nuget",
        ];

        for cmd in &safe_commands {
            self.always_allowed.insert(cmd.to_string());
        }
    }

    /// Add safe file operation commands
    fn add_safe_file_commands(&mut self) {
        let file_commands = [
            "ls", "cat", "head", "tail", "wc", "sort", "uniq", "grep", "awk", "sed", "cut", "tr",
            "tee", "find", "locate", "which", "whereis", "type", "file", "stat", "du", "df", "pwd",
            "dirname", "basename",
        ];

        for cmd in &file_commands {
            self.always_allowed.insert(cmd.to_string());
        }
    }

    /// Add commands that are conditionally allowed
    fn add_conditional_commands(&mut self) {
        // Docker with restrictions
        self.conditionally_allowed.insert("docker".to_string(), CommandRestrictions {
            max_args: Some(20),
            required_args: vec![],
            forbidden_args: vec!["--privileged".to_string()],
            allowed_patterns: vec![],
            allowed_argv: vec![],
            forbidden_patterns: vec![
                r"--user.*root".to_string(),
                r"--volume.*:/".to_string(),
                r"--mount.*source=/".to_string(),
            ],
            requires_elevation: false,
            custom_validator: Some("validate_docker".to_string()),
        });

        // Python with restrictions (no -c flag)
        self.conditionally_allowed.insert("python".to_string(), CommandRestrictions {
            max_args: Some(10),
            required_args: vec![],
            forbidden_args: vec!["-c".to_string(), "--command".to_string()],
            allowed_patterns: vec![],
            allowed_argv: vec![],
            forbidden_patterns: vec![r"-c\s+".to_string()],
            requires_elevation: false,
            custom_validator: Some("validate_python".to_string()),
        });

        // File operations with restrictions
        for cmd in &["cp", "mv", "rm", "mkdir", "rmdir"] {
            self.conditionally_allowed.insert(cmd.to_string(), CommandRestrictions {
                max_args: Some(100),
                required_args: vec![],
                forbidden_args: vec![],
                allowed_patterns: vec![],
                allowed_argv: vec![],
                forbidden_patterns: vec![
                    r"^/etc/".to_string(),
                    r"^/bin/".to_string(),
                    r"^/sbin/".to_string(),
                    r"^/usr/bin/".to_string(),
                    r"^/usr/sbin/".to_string(),
                    r"^/boot/".to_string(),
                    r"^/root/".to_string(),
                    r"^/sys/".to_string(),
                    r"^/proc/".to_string(),
                    r"^/dev/".to_string(),
                ],
                requires_elevation: false,
                custom_validator: Some("validate_file_ops".to_string()),
            });
        }
    }

    /// Check if a command is allowed
    pub fn is_command_allowed(&self, command: &str) -> bool {
        // Check if explicitly blocked
        if self.blocked.contains(command) {
            debug!("Command '{}' is explicitly blocked", command);
            return false;
        }

        // Check if always allowed
        if self.always_allowed.contains(command) {
            debug!("Command '{}' is always allowed", command);
            return true;
        }

        // Check if conditionally allowed
        if self.conditionally_allowed.contains_key(command) {
            debug!("Command '{}' is conditionally allowed", command);
            return true;
        }

        // Check permissive mode
        if self.permissive_mode {
            warn!("Command '{}' allowed due to permissive mode", command);
            return true;
        }

        debug!("Command '{}' not found in allowlist", command);
        false
    }

    /// Validate command with arguments against restrictions, in the legacy
    /// (unprivileged) enforcement mode.
    pub fn validate_command(&self, command: &str, args: &[String]) -> Result<()> {
        self.validate_command_with_mode(command, args, EnforcementMode::Legacy)
    }

    /// Validate command with arguments against restrictions under the given
    /// enforcement mode. See [`EnforcementMode`] for the differences.
    pub fn validate_command_with_mode(
        &self,
        command: &str,
        args: &[String],
        mode: EnforcementMode,
    ) -> Result<()> {
        if self.blocked.contains(command) {
            return Err(AgentError::security(format!(
                "Command '{}' is not allowed",
                command
            )));
        }

        match mode {
            EnforcementMode::Legacy => {
                if !self.is_command_allowed(command) {
                    return Err(AgentError::security(format!(
                        "Command '{}' is not allowed",
                        command
                    )));
                }
                if let Some(restrictions) = self.conditionally_allowed.get(command) {
                    if restrictions.requires_elevation {
                        return Err(AgentError::security(format!(
                            "Command '{}' requires elevation and may only run through \
                             `exec` under the fixed root policy",
                            command
                        )));
                    }
                    self.apply_restrictions(command, args, restrictions, mode)?;
                }
                Ok(())
            }
            EnforcementMode::Elevated => {
                let Some(restrictions) = self.conditionally_allowed.get(command) else {
                    return Err(AgentError::security(format!(
                        "Command '{}' has no argument-restricted rule; only \
                         conditionally_allowed rules with allowed_argv or \
                         allowed_patterns can run elevated",
                        command
                    )));
                };
                if restrictions.custom_validator.is_some() {
                    return Err(AgentError::security(format!(
                        "Command '{}' names a custom_validator, which is not \
                         implemented; refusing to run it elevated",
                        command
                    )));
                }
                self.apply_restrictions(command, args, restrictions, mode)
            }
        }
    }

    /// Apply restrictions to a command
    fn apply_restrictions(
        &self,
        command: &str,
        args: &[String],
        restrictions: &CommandRestrictions,
        mode: EnforcementMode,
    ) -> Result<()> {
        // Elevated mode: a rule with no positive restriction is a deny.
        if mode == EnforcementMode::Elevated
            && restrictions.allowed_argv.is_empty()
            && restrictions.allowed_patterns.is_empty()
        {
            return Err(AgentError::security(format!(
                "Command '{}' has no allowed_argv or allowed_patterns; refusing \
                 to run it elevated",
                command
            )));
        }

        // Exact argv templates.
        if !restrictions.allowed_argv.is_empty()
            && !restrictions
                .allowed_argv
                .iter()
                .any(|template| template.as_slice() == args)
        {
            return Err(AgentError::security(format!(
                "Command '{}' arguments {:?} do not exactly match any allowed argv",
                command, args
            )));
        }

        // Anchored allow patterns: every argument must fully match one.
        if !restrictions.allowed_patterns.is_empty() {
            let regexes = restrictions
                .allowed_patterns
                .iter()
                .map(|p| {
                    regex::Regex::new(&anchored(p)).map_err(|e| {
                        AgentError::validation(format!("Invalid regex pattern '{}': {}", p, e))
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            for arg in args {
                if !regexes.iter().any(|r| r.is_match(arg)) {
                    return Err(AgentError::security(format!(
                        "Command '{}' argument '{}' does not match any allowed pattern",
                        command, arg
                    )));
                }
            }
        }

        // Check maximum arguments
        if let Some(max_args) = restrictions.max_args {
            if args.len() > max_args {
                return Err(AgentError::validation(format!(
                    "Command '{}' has too many arguments ({} > {})",
                    command,
                    args.len(),
                    max_args
                )));
            }
        }

        // Check required arguments
        for required in &restrictions.required_args {
            if !args.contains(required) {
                return Err(AgentError::validation(format!(
                    "Command '{}' missing required argument: {}",
                    command, required
                )));
            }
        }

        // Check forbidden arguments
        for forbidden in &restrictions.forbidden_args {
            if args.contains(forbidden) {
                return Err(AgentError::security(format!(
                    "Command '{}' contains forbidden argument: {}",
                    command, forbidden
                )));
            }
        }

        // Check forbidden patterns
        for pattern in &restrictions.forbidden_patterns {
            let regex = regex::Regex::new(pattern)
                .map_err(|e| AgentError::validation(format!("Invalid regex pattern: {}", e)))?;

            for arg in args {
                if regex.is_match(arg) {
                    return Err(AgentError::security(format!(
                        "Command '{}' argument '{}' matches forbidden pattern: {}",
                        command, arg, pattern
                    )));
                }
            }
        }

        Ok(())
    }

    /// Add a command to the allowlist
    pub fn add_allowed_command(&mut self, command: String) {
        self.always_allowed.insert(command.clone());
        info!("Added command '{}' to allowlist", command);
    }

    /// Remove a command from the allowlist
    pub fn remove_allowed_command(&mut self, command: &str) {
        self.always_allowed.remove(command);
        self.conditionally_allowed.remove(command);
        info!("Removed command '{}' from allowlist", command);
    }

    /// Block a command explicitly
    pub fn block_command(&mut self, command: String) {
        self.blocked.insert(command.clone());
        self.always_allowed.remove(&command);
        self.conditionally_allowed.remove(&command);
        warn!("Blocked command '{}'", command);
    }

    /// Enable or disable permissive mode
    pub fn set_permissive_mode(&mut self, enabled: bool) {
        self.permissive_mode = enabled;
        if enabled {
            warn!("Permissive mode ENABLED - security reduced!");
        } else {
            info!("Permissive mode disabled - security restored");
        }
    }

    /// Get statistics about the allowlist
    pub fn get_stats(&self) -> AllowlistStats {
        AllowlistStats {
            always_allowed_count: self.always_allowed.len(),
            conditionally_allowed_count: self.conditionally_allowed.len(),
            blocked_count: self.blocked.len(),
            permissive_mode: self.permissive_mode,
        }
    }
}

/// Statistics about the allowlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowlistStats {
    pub always_allowed_count: usize,
    pub conditionally_allowed_count: usize,
    pub blocked_count: usize,
    pub permissive_mode: bool,
}

/// A narrowing overlay applied on top of an existing `AllowlistConfig`.
///
/// Every field is optional. When a field is present, its contents must
/// represent a *strictly equal or tighter* policy than the base — the
/// `apply_overlay` function rejects anything that would widen access.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllowlistOverlay {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub always_allowed: Option<HashSet<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditionally_allowed: Option<HashMap<String, CommandRestrictions>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked: Option<HashSet<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissive_mode: Option<bool>,
}

impl AllowlistConfig {
    /// Apply a narrowing overlay, returning the merged config or an error if
    /// the overlay would widen access.
    pub fn apply_overlay(&self, overlay: &AllowlistOverlay) -> Result<AllowlistConfig> {
        let mut merged = self.clone();

        // 1. permissive_mode — overlay can flip it OFF, but never ON if base is OFF.
        if let Some(p) = overlay.permissive_mode {
            if p && !self.permissive_mode {
                return Err(AgentError::security(
                    "policy overlay attempted to enable permissive_mode (would widen access)",
                ));
            }
            merged.permissive_mode = p;
        }

        // 2. always_allowed — every overlay entry must already be reachable in
        //    the base. Result narrows by enumeration.
        if let Some(overlay_allowed) = &overlay.always_allowed {
            let base_reachable: HashSet<String> = self
                .always_allowed
                .iter()
                .cloned()
                .chain(self.conditionally_allowed.keys().cloned())
                .collect();
            for cmd in overlay_allowed {
                if !base_reachable.contains(cmd) {
                    return Err(AgentError::security(format!(
                        "policy overlay tried to add '{}' to always_allowed (not reachable in base)",
                        cmd
                    )));
                }
            }
            // A command that is conditional in the base keeps its argument
            // restrictions: listing it here must not promote it to
            // always_allowed, which would drop those restrictions and widen
            // access.
            merged.always_allowed = overlay_allowed
                .iter()
                .filter(|cmd| self.always_allowed.contains(*cmd))
                .cloned()
                .collect();
        }

        // 3. blocked — union (overlay can only add bans).
        if let Some(overlay_blocked) = &overlay.blocked {
            for cmd in overlay_blocked {
                merged.blocked.insert(cmd.clone());
                merged.always_allowed.remove(cmd);
                merged.conditionally_allowed.remove(cmd);
            }
        }

        // 4. conditionally_allowed — only existing keys, with tightened restrictions.
        if let Some(overlay_cond) = &overlay.conditionally_allowed {
            for (cmd, overlay_restr) in overlay_cond {
                let base_reachable = self.always_allowed.contains(cmd)
                    || self.conditionally_allowed.contains_key(cmd);
                if !base_reachable {
                    return Err(AgentError::security(format!(
                        "policy overlay tried to introduce conditional rules for '{}' (not reachable in base)",
                        cmd
                    )));
                }

                let base_restr = self
                    .conditionally_allowed
                    .get(cmd)
                    .cloned()
                    .unwrap_or_default();

                merged.always_allowed.remove(cmd);
                match tighten_restrictions(&base_restr, overlay_restr) {
                    Some(tightened) => {
                        merged.conditionally_allowed.insert(cmd.clone(), tightened);
                    }
                    None => {
                        // Base and overlay allow-lists share nothing, so no
                        // argument vector can satisfy both. Leaving an empty
                        // list would read as "no positive restriction" in
                        // legacy mode, which widens access. Block instead.
                        warn!(
                            "policy overlay allow-list for '{}' does not intersect the base; blocking it",
                            cmd
                        );
                        merged.conditionally_allowed.remove(cmd);
                        merged.blocked.insert(cmd.clone());
                    }
                }
            }
        }

        Ok(merged)
    }
}

/// Intersect two positive allow-lists.
///
/// An empty list means "no positive restriction", so the populated side wins.
/// When both are populated the result is their intersection, and `None`
/// signals that the intersection is empty (nothing can pass both).
fn intersect_allow<T: Clone + PartialEq>(base: &[T], overlay: &[T]) -> Option<Vec<T>> {
    match (base.is_empty(), overlay.is_empty()) {
        (true, _) => Some(overlay.to_vec()),
        (_, true) => Some(base.to_vec()),
        (false, false) => {
            let both: Vec<T> = base
                .iter()
                .filter(|p| overlay.contains(p))
                .cloned()
                .collect();
            if both.is_empty() {
                None
            } else {
                Some(both)
            }
        }
    }
}

/// Combine two `CommandRestrictions` such that the result is no looser than
/// either operand. Forbidden lists are unioned; required lists are unioned;
/// max_args is the minimum of the two; `allowed_patterns` and `allowed_argv`
/// are intersected when both sides populate them (otherwise the populated
/// side wins).
///
/// Returns `None` when both sides populate an allow-list and the lists share
/// nothing: an empty result would otherwise read as "no positive
/// restriction" and widen access, so the caller must block the command.
fn tighten_restrictions(
    base: &CommandRestrictions,
    overlay: &CommandRestrictions,
) -> Option<CommandRestrictions> {
    let mut required = base.required_args.clone();
    for r in &overlay.required_args {
        if !required.contains(r) {
            required.push(r.clone());
        }
    }

    let mut forbidden = base.forbidden_args.clone();
    for f in &overlay.forbidden_args {
        if !forbidden.contains(f) {
            forbidden.push(f.clone());
        }
    }

    let mut forbidden_pat = base.forbidden_patterns.clone();
    for f in &overlay.forbidden_patterns {
        if !forbidden_pat.contains(f) {
            forbidden_pat.push(f.clone());
        }
    }

    let allowed_pat = intersect_allow(&base.allowed_patterns, &overlay.allowed_patterns)?;
    let allowed_argv = intersect_allow(&base.allowed_argv, &overlay.allowed_argv)?;

    let max_args = match (base.max_args, overlay.max_args) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    Some(CommandRestrictions {
        max_args,
        required_args: required,
        forbidden_args: forbidden,
        allowed_patterns: allowed_pat,
        allowed_argv,
        forbidden_patterns: forbidden_pat,
        requires_elevation: base.requires_elevation || overlay.requires_elevation,
        custom_validator: overlay
            .custom_validator
            .clone()
            .or_else(|| base.custom_validator.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allowlist() {
        let config = AllowlistConfig::secure_default();

        // Should allow safe commands
        assert!(config.is_command_allowed("git"));
        assert!(config.is_command_allowed("cargo"));
        assert!(config.is_command_allowed("ls"));

        // Should block dangerous commands
        assert!(!config.is_command_allowed("bash"));
        assert!(!config.is_command_allowed("curl"));
        assert!(!config.is_command_allowed("sudo"));
    }

    #[test]
    fn test_command_validation() {
        let config = AllowlistConfig::secure_default();

        // Should allow safe git commands
        assert!(config
            .validate_command("git", &["status".to_string()])
            .is_ok());

        // Should block dangerous python usage
        assert!(config
            .validate_command("python", &["-c".to_string(), "print('hi')".to_string()])
            .is_err());
    }

    #[test]
    fn test_permissive_mode() {
        let mut config = AllowlistConfig::secure_default();

        // Should block unknown command by default
        assert!(!config.is_command_allowed("unknown_command"));

        // Should allow unknown command in permissive mode
        config.set_permissive_mode(true);
        assert!(config.is_command_allowed("unknown_command"));

        // Should still block explicitly blocked commands
        assert!(!config.is_command_allowed("bash"));
    }

    // ---------------- Overlay narrow-only enforcement ----------------

    fn make_base() -> AllowlistConfig {
        let mut cfg = AllowlistConfig {
            always_allowed: HashSet::new(),
            conditionally_allowed: HashMap::new(),
            blocked: HashSet::new(),
            permissive_mode: false,
        };
        cfg.always_allowed.insert("git".to_string());
        cfg.always_allowed.insert("make".to_string());
        cfg.blocked.insert("bash".to_string());
        cfg.conditionally_allowed.insert(
            "docker".to_string(),
            CommandRestrictions {
                max_args: Some(20),
                required_args: vec![],
                forbidden_args: vec!["--privileged".to_string()],
                allowed_patterns: vec![],
                allowed_argv: vec![],
                forbidden_patterns: vec![],
                requires_elevation: false,
                custom_validator: None,
            },
        );
        cfg
    }

    #[test]
    fn overlay_can_narrow_always_allowed() {
        let base = make_base();
        let mut overlay_set = HashSet::new();
        overlay_set.insert("git".to_string()); // drops `make`
        let overlay = AllowlistOverlay {
            always_allowed: Some(overlay_set),
            ..AllowlistOverlay::default()
        };
        let merged = base
            .apply_overlay(&overlay)
            .expect("narrowing should succeed");
        assert!(merged.always_allowed.contains("git"));
        assert!(!merged.always_allowed.contains("make"));
    }

    #[test]
    fn overlay_rejects_widening_always_allowed() {
        let base = make_base();
        let mut overlay_set = HashSet::new();
        overlay_set.insert("curl".to_string()); // not in base
        let overlay = AllowlistOverlay {
            always_allowed: Some(overlay_set),
            ..AllowlistOverlay::default()
        };
        let err = base.apply_overlay(&overlay).unwrap_err();
        assert!(
            err.to_string().contains("not reachable in base"),
            "got: {err}"
        );
    }

    #[test]
    fn overlay_can_add_to_blocked() {
        let base = make_base();
        let mut bset = HashSet::new();
        bset.insert("zsh".to_string());
        let overlay = AllowlistOverlay {
            blocked: Some(bset),
            ..AllowlistOverlay::default()
        };
        let merged = base.apply_overlay(&overlay).unwrap();
        assert!(merged.blocked.contains("bash"));
        assert!(merged.blocked.contains("zsh"));
    }

    #[test]
    fn overlay_blocked_overrides_allowed() {
        let base = make_base();
        let mut bset = HashSet::new();
        bset.insert("git".to_string()); // ban a previously-allowed command
        let overlay = AllowlistOverlay {
            blocked: Some(bset),
            ..AllowlistOverlay::default()
        };
        let merged = base.apply_overlay(&overlay).unwrap();
        assert!(merged.blocked.contains("git"));
        assert!(!merged.always_allowed.contains("git"));
        assert!(!merged.is_command_allowed("git"));
    }

    #[test]
    fn overlay_can_tighten_conditional_restrictions() {
        let base = make_base();
        let mut cond = HashMap::new();
        cond.insert(
            "docker".to_string(),
            CommandRestrictions {
                max_args: Some(5), // tighter than base's 20
                required_args: vec![],
                forbidden_args: vec!["--rm".to_string()], // adds new ban
                allowed_patterns: vec![],
                allowed_argv: vec![],
                forbidden_patterns: vec![],
                requires_elevation: false,
                custom_validator: None,
            },
        );
        let overlay = AllowlistOverlay {
            conditionally_allowed: Some(cond),
            ..AllowlistOverlay::default()
        };
        let merged = base.apply_overlay(&overlay).unwrap();
        let r = merged.conditionally_allowed.get("docker").unwrap();
        assert_eq!(r.max_args, Some(5));
        assert!(r.forbidden_args.contains(&"--privileged".to_string())); // base preserved
        assert!(r.forbidden_args.contains(&"--rm".to_string())); // overlay added
    }

    #[test]
    fn overlay_rejects_introducing_unknown_conditional() {
        let base = make_base();
        let mut cond = HashMap::new();
        cond.insert(
            "wget".to_string(),
            CommandRestrictions {
                max_args: Some(1),
                required_args: vec![],
                forbidden_args: vec![],
                allowed_patterns: vec![],
                allowed_argv: vec![],
                forbidden_patterns: vec![],
                requires_elevation: false,
                custom_validator: None,
            },
        );
        let overlay = AllowlistOverlay {
            conditionally_allowed: Some(cond),
            ..AllowlistOverlay::default()
        };
        let err = base.apply_overlay(&overlay).unwrap_err();
        assert!(err.to_string().contains("not reachable in base"));
    }

    #[test]
    fn overlay_rejects_enabling_permissive_mode() {
        let base = make_base();
        let overlay = AllowlistOverlay {
            permissive_mode: Some(true),
            ..AllowlistOverlay::default()
        };
        let err = base.apply_overlay(&overlay).unwrap_err();
        assert!(err.to_string().contains("permissive_mode"));
    }

    #[test]
    fn overlay_can_disable_permissive_mode() {
        let mut base = make_base();
        base.permissive_mode = true;
        let overlay = AllowlistOverlay {
            permissive_mode: Some(false),
            ..AllowlistOverlay::default()
        };
        let merged = base.apply_overlay(&overlay).unwrap();
        assert!(!merged.permissive_mode);
    }

    #[test]
    fn test_restrictions() {
        let mut config = AllowlistConfig::secure_default();

        // Add a command with restrictions
        config.conditionally_allowed.insert("test_cmd".to_string(), CommandRestrictions {
            max_args: Some(2),
            required_args: vec!["--required".to_string()],
            forbidden_args: vec!["--forbidden".to_string()],
            allowed_patterns: vec![],
            allowed_argv: vec![],
            forbidden_patterns: vec![],
            requires_elevation: false,
            custom_validator: None,
        });

        // Should reject too many args
        assert!(config
            .validate_command(
                "test_cmd",
                &["arg1".to_string(), "arg2".to_string(), "arg3".to_string()]
            )
            .is_err());

        // Should reject missing required arg
        assert!(config
            .validate_command("test_cmd", &["arg1".to_string()])
            .is_err());

        // Should reject forbidden arg
        assert!(config
            .validate_command(
                "test_cmd",
                &["--required".to_string(), "--forbidden".to_string()]
            )
            .is_err());

        // Should accept valid args
        assert!(config
            .validate_command(
                "test_cmd",
                &["--required".to_string(), "valid_arg".to_string()]
            )
            .is_ok());
    }
}
