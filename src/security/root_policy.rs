// file: src/security/root_policy.rs
// version: 1.0.0
// guid: 9b3e6d21-5c4f-4a8e-b1d7-0e2f8c6a4b59
// last-edited: 2026-10-04

//! The fixed, root-owned policy used when safe-ai-util runs elevated.
//!
//! Under sudo the caller controls argv, the environment and the working
//! directory, so none of those may choose the policy. The policy is read only
//! from [`ROOT_POLICY_PATH`], and only after checking that the file and every
//! directory above it are owned by root and not writable by group or other.
//!
//! Each rule names an absolute binary and the argument vectors it may receive.
//! Argument checking reuses [`AllowlistConfig::validate_command_with_mode`] in
//! [`EnforcementMode::Elevated`], so there is one enforcement path, not two.

use crate::error::{AgentError, Result};
use crate::security::allowlist::{AllowlistConfig, CommandRestrictions, EnforcementMode};
use crate::security::elevation::ElevationContext;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

/// The only place an elevated run reads its policy from.
pub const ROOT_POLICY_PATH: &str = "/etc/safe-ai-util/root-policy.toml";

/// PATH given to children when the policy does not set one.
pub const DEFAULT_CHILD_PATH: &str = "/usr/sbin:/usr/bin:/sbin:/bin";

/// Audit log used when the policy does not set one.
pub const DEFAULT_LOG_FILE: &str = "/var/log/safe-ai-util/root-exec.log";

/// Binaries that may never be the target of an `exec` rule, whatever the
/// policy says: shells, interpreters and anything that runs another program
/// from its arguments. Checked against the rule name, the configured binary
/// path's basename and the canonical (symlink-resolved) basename.
pub fn forbidden_exec_basenames() -> HashSet<String> {
    let mut set = AllowlistConfig::default_blocked_commands();
    for name in [
        "python",
        "python2",
        "python3",
        "node",
        "nodejs",
        "deno",
        "bun",
        "env",
        "xargs",
        "busybox",
        "toybox",
        "awk",
        "gawk",
        "mawk",
        "nawk",
        "expect",
        "osascript",
        "nohup",
        "nice",
        "timeout",
        "stdbuf",
        "setsid",
        "runuser",
        "chpst",
        "doas",
        "sudoedit",
        "find",
        "ssh",
        "script",
        "watch",
        "strace",
        "gdb",
        "lldb",
    ] {
        set.insert(name.to_string());
    }
    set
}

fn is_forbidden_basename(name: &str, forbidden: &HashSet<String>) -> bool {
    if forbidden.contains(name) {
        return true;
    }
    // python3.12, perl5.36, ruby3.2 and friends.
    let stem: String = name
        .trim_end_matches(|c: char| c.is_ascii_digit() || c == '.')
        .to_string();
    forbidden.contains(&stem)
}

/// One `exec` rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootCommandRule {
    /// Absolute path of the binary to run. Never resolved through PATH.
    pub binary: String,
    /// See [`CommandRestrictions::requires_elevation`].
    #[serde(default)]
    pub requires_elevation: bool,
    /// Exact argument vectors (everything after the binary).
    #[serde(default)]
    pub allowed_argv: Vec<Vec<String>>,
    /// Anchored per-argument patterns.
    #[serde(default)]
    pub allowed_patterns: Vec<String>,
    #[serde(default)]
    pub max_args: Option<usize>,
    #[serde(default)]
    pub forbidden_args: Vec<String>,
    #[serde(default)]
    pub forbidden_patterns: Vec<String>,
}

impl RootCommandRule {
    fn restrictions(&self) -> CommandRestrictions {
        CommandRestrictions {
            max_args: self.max_args,
            required_args: vec![],
            forbidden_args: self.forbidden_args.clone(),
            allowed_patterns: self.allowed_patterns.clone(),
            allowed_argv: self.allowed_argv.clone(),
            forbidden_patterns: self.forbidden_patterns.clone(),
            requires_elevation: self.requires_elevation,
            custom_validator: None,
        }
    }
}

/// The parsed root policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootPolicy {
    /// PATH handed to children (the environment is otherwise cleared).
    #[serde(default)]
    pub path: Option<String>,
    /// Append-only JSON-lines audit log.
    #[serde(default)]
    pub log_file: Option<String>,
    /// Rules keyed by name (`exec <name> args...`).
    #[serde(default)]
    pub commands: BTreeMap<String, RootCommandRule>,
}

/// An invocation that passed policy. Holds exactly the argv that will be
/// passed to execve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedExec {
    pub rule: String,
    pub binary: PathBuf,
    pub args: Vec<String>,
    pub child_path: String,
}

impl RootPolicy {
    /// Parse and validate policy text. Rejects relative binaries, forbidden
    /// binaries, unparsable regexes, rules with no positive restriction and
    /// a malformed PATH, so a bad policy fails at load rather than at use.
    pub fn parse(text: &str) -> Result<Self> {
        let policy: RootPolicy = toml::from_str(text)
            .map_err(|e| AgentError::config(format!("Failed to parse root policy: {}", e)))?;
        policy.validate()?;
        Ok(policy)
    }

    fn validate(&self) -> Result<()> {
        let forbidden = forbidden_exec_basenames();
        if self.commands.is_empty() {
            return Err(AgentError::config("root policy defines no commands"));
        }
        validate_child_path(self.child_path())?;
        if !Path::new(self.log_path()).is_absolute() {
            return Err(AgentError::config("root policy log_file must be absolute"));
        }
        for (name, rule) in &self.commands {
            let bin = Path::new(&rule.binary);
            if !is_clean_absolute(bin) {
                return Err(AgentError::config(format!(
                    "rule '{}': binary '{}' must be an absolute path with no '.' or '..' components",
                    name, rule.binary
                )));
            }
            let base = bin.file_name().and_then(|b| b.to_str()).unwrap_or("");
            if is_forbidden_basename(name, &forbidden) || is_forbidden_basename(base, &forbidden) {
                return Err(AgentError::config(format!(
                    "rule '{}': shells, interpreters and program launchers may not be exec targets",
                    name
                )));
            }
            if rule.allowed_argv.is_empty() && rule.allowed_patterns.is_empty() {
                return Err(AgentError::config(format!(
                    "rule '{}': needs allowed_argv or allowed_patterns",
                    name
                )));
            }
            for p in rule.allowed_patterns.iter().chain(&rule.forbidden_patterns) {
                regex::Regex::new(p).map_err(|e| {
                    AgentError::config(format!("rule '{}': bad regex '{}': {}", name, p, e))
                })?;
            }
        }
        Ok(())
    }

    pub fn child_path(&self) -> &str {
        self.path.as_deref().unwrap_or(DEFAULT_CHILD_PATH)
    }

    pub fn log_path(&self) -> &str {
        self.log_file.as_deref().unwrap_or(DEFAULT_LOG_FILE)
    }

    /// The equivalent allowlist, for the shared enforcement path.
    fn as_allowlist(&self) -> AllowlistConfig {
        let mut conditionally_allowed = HashMap::new();
        for (name, rule) in &self.commands {
            conditionally_allowed.insert(name.clone(), rule.restrictions());
        }
        AllowlistConfig {
            always_allowed: HashSet::new(),
            conditionally_allowed,
            blocked: forbidden_exec_basenames(),
            permissive_mode: false,
        }
    }

    /// Decide whether `argv` (`[name-or-binary, args...]`) may run.
    ///
    /// `argv[0]` is either a rule name or that rule's exact `binary` path.
    /// The remaining elements are checked as one exact vector; nothing is
    /// split, joined, unquoted or rewritten, so `"a; rm -rf /"` is one
    /// argument that matches no template.
    pub fn authorize(&self, ctx: &ElevationContext, argv: &[String]) -> Result<AuthorizedExec> {
        let Some((head, args)) = argv.split_first() else {
            return Err(AgentError::security("exec: no command given"));
        };
        let (name, rule) = self
            .commands
            .get_key_value(head.as_str())
            .or_else(|| self.commands.iter().find(|(_, r)| &r.binary == head))
            .ok_or_else(|| {
                AgentError::security(format!("exec: '{}' is not in the root policy", head))
            })?;

        if rule.requires_elevation && !ctx.is_elevated() {
            return Err(AgentError::security(format!(
                "exec: '{}' requires elevation ({})",
                name,
                ctx.describe()
            )));
        }

        self.as_allowlist()
            .validate_command_with_mode(name, args, EnforcementMode::Elevated)?;

        Ok(AuthorizedExec {
            rule: name.clone(),
            binary: PathBuf::from(&rule.binary),
            args: args.to_vec(),
            child_path: self.child_path().to_string(),
        })
    }
}

fn is_clean_absolute(p: &Path) -> bool {
    p.is_absolute()
        && p.components()
            .all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
}

fn validate_child_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(AgentError::config("root policy path must not be empty"));
    }
    for entry in path.split(':') {
        if !is_clean_absolute(Path::new(entry)) {
            return Err(AgentError::config(format!(
                "root policy path entry '{}' must be absolute with no '.' or '..'",
                entry
            )));
        }
    }
    Ok(())
}

/// Where and how to load the root policy.
///
/// Production uses [`RootPolicyLoader::system`] only. The other fields exist
/// so tests can point at a temporary directory owned by the test user; no
/// command-line flag or environment variable reaches them.
#[derive(Debug, Clone)]
pub struct RootPolicyLoader {
    pub path: PathBuf,
    /// uid that must own the file and every checked directory.
    pub trusted_uid: u32,
    /// Stop the ancestor check here (inclusive). `None` checks up to `/`.
    pub trust_anchor: Option<PathBuf>,
}

impl RootPolicyLoader {
    pub fn system() -> Self {
        Self {
            path: PathBuf::from(ROOT_POLICY_PATH),
            trusted_uid: 0,
            trust_anchor: None,
        }
    }

    /// Verify ownership and permissions, then read and parse the policy.
    pub fn load(&self) -> Result<RootPolicy> {
        let text = read_trusted_file(&self.path, self.trusted_uid, self.trust_anchor.as_deref())?;
        RootPolicy::parse(&text)
    }
}

#[cfg(unix)]
mod unix_checks {
    use super::*;
    use std::fs::{File, Metadata, OpenOptions};
    use std::io::Read;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

    fn check_meta(path: &Path, meta: &Metadata, uid: u32, what: &str) -> Result<()> {
        if meta.uid() != uid {
            return Err(AgentError::security(format!(
                "{} {} is owned by uid {}, expected uid {}",
                what,
                path.display(),
                meta.uid(),
                uid
            )));
        }
        if meta.mode() & 0o022 != 0 {
            return Err(AgentError::security(format!(
                "{} {} is writable by group or other (mode {:o})",
                what,
                path.display(),
                meta.mode() & 0o7777
            )));
        }
        Ok(())
    }

    /// Check `dir` and each of its ancestors (up to and including `anchor`,
    /// or `/`). `dir` is canonicalized first so symlinked ancestors such as
    /// macOS's `/etc -> private/etc` are judged by their targets; the
    /// symlinks themselves live in directories this walk also checks.
    pub fn verify_dir_chain(dir: &Path, uid: u32, anchor: Option<&Path>) -> Result<PathBuf> {
        let canon = dir.canonicalize().map_err(|e| {
            AgentError::security(format!("cannot resolve {}: {}", dir.display(), e))
        })?;
        let anchor = match anchor {
            Some(a) => Some(a.canonicalize().map_err(|e| {
                AgentError::security(format!("cannot resolve {}: {}", a.display(), e))
            })?),
            None => None,
        };
        if let Some(a) = &anchor {
            if !canon.starts_with(a) {
                return Err(AgentError::security(format!(
                    "{} is outside the trust anchor {}",
                    canon.display(),
                    a.display()
                )));
            }
        }
        let mut cur: Option<&Path> = Some(canon.as_path());
        while let Some(p) = cur {
            let meta = std::fs::symlink_metadata(p)
                .map_err(|e| AgentError::security(format!("cannot stat {}: {}", p.display(), e)))?;
            if !meta.is_dir() {
                return Err(AgentError::security(format!(
                    "{} is not a directory",
                    p.display()
                )));
            }
            check_meta(p, &meta, uid, "directory")?;
            if anchor.as_deref() == Some(p) {
                break;
            }
            cur = p.parent();
        }
        Ok(canon)
    }

    /// Open `path` without following a final symlink and check the opened
    /// file (fstat, so the checked inode is the read inode).
    pub fn read_trusted_file(path: &Path, uid: u32, anchor: Option<&Path>) -> Result<String> {
        let parent = path
            .parent()
            .ok_or_else(|| AgentError::security("policy path has no parent"))?;
        let name = path
            .file_name()
            .ok_or_else(|| AgentError::security("policy path has no file name"))?;
        let dir = verify_dir_chain(parent, uid, anchor)?;
        let full = dir.join(name);
        let mut file: File = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&full)
            .map_err(|e| {
                AgentError::config(format!("cannot open root policy {}: {}", full.display(), e))
            })?;
        let meta = file.metadata()?;
        if !meta.is_file() {
            return Err(AgentError::security(format!(
                "{} is not a regular file",
                full.display()
            )));
        }
        check_meta(&full, &meta, uid, "policy file")?;
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        Ok(text)
    }

    /// Resolve the binary through symlinks and check that the result is a
    /// root-owned, non-group/other-writable regular executable in a
    /// root-owned directory chain whose basename is not a shell or
    /// interpreter. Returns the canonical path, which is what gets executed.
    pub fn verify_binary(binary: &Path, uid: u32) -> Result<PathBuf> {
        let canon = binary.canonicalize().map_err(|e| {
            AgentError::security(format!("cannot resolve binary {}: {}", binary.display(), e))
        })?;
        let base = canon.file_name().and_then(|b| b.to_str()).unwrap_or("");
        if is_forbidden_basename(base, &forbidden_exec_basenames()) {
            return Err(AgentError::security(format!(
                "binary {} resolves to a shell, interpreter or launcher ({})",
                binary.display(),
                canon.display()
            )));
        }
        let parent = canon
            .parent()
            .ok_or_else(|| AgentError::security("binary has no parent directory"))?;
        verify_dir_chain(parent, uid, None)?;
        let meta = std::fs::metadata(&canon)?;
        if !meta.is_file() || meta.mode() & 0o111 == 0 {
            return Err(AgentError::security(format!(
                "{} is not an executable regular file",
                canon.display()
            )));
        }
        check_meta(&canon, &meta, uid, "binary")?;
        Ok(canon)
    }
}

#[cfg(unix)]
pub use unix_checks::{verify_binary, verify_dir_chain};

#[cfg(unix)]
fn read_trusted_file(path: &Path, uid: u32, anchor: Option<&Path>) -> Result<String> {
    unix_checks::read_trusted_file(path, uid, anchor)
}

#[cfg(not(unix))]
fn read_trusted_file(_path: &Path, _uid: u32, _anchor: Option<&Path>) -> Result<String> {
    Err(AgentError::security(
        "the root policy gate is only supported on unix",
    ))
}

#[cfg(not(unix))]
pub fn verify_binary(_binary: &Path, _uid: u32) -> Result<PathBuf> {
    Err(AgentError::security("exec is only supported on unix"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    const ONE_RULE: &str = r#"
[commands.zfs]
binary = "/usr/sbin/zfs"
requires_elevation = true
allowed_argv = [["destroy", "bigdata/rehearsal-sandbox-data"]]
"#;

    #[test]
    fn exact_destroy_only() {
        let p = RootPolicy::parse(ONE_RULE).unwrap();
        let root = ElevationContext::sudo_root("jdfalk");

        let ok = p
            .authorize(
                &root,
                &s(&["zfs", "destroy", "bigdata/rehearsal-sandbox-data"]),
            )
            .unwrap();
        assert_eq!(ok.binary, PathBuf::from("/usr/sbin/zfs"));
        assert_eq!(ok.args, s(&["destroy", "bigdata/rehearsal-sandbox-data"]));
        assert_eq!(ok.child_path, DEFAULT_CHILD_PATH);

        // The binary path is accepted as the name too.
        assert!(p
            .authorize(
                &root,
                &s(&["/usr/sbin/zfs", "destroy", "bigdata/rehearsal-sandbox-data"])
            )
            .is_ok());

        let refused: &[&[&str]] = &[
            &["zfs", "destroy", "-r", "bigdata/BD/bigdata/books"],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data; rm -rf /"],
            &[
                "zfs",
                "destroy",
                "bigdata/rehearsal-sandbox-data;",
                "rm",
                "-rf",
                "/",
            ],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data", "-r"],
            &["zfs", "destroy", "-r", "bigdata/rehearsal-sandbox-data"],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data", "extra"],
            &["zfs", "destroy", "bigdata/other-dataset"],
            &["zfs", "destroy"],
            &["zfs"],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data\n"],
            &["zfs", "destroy", " bigdata/rehearsal-sandbox-data"],
            &["/sbin/zfs", "destroy", "bigdata/rehearsal-sandbox-data"],
            &["rm", "-rf", "/"],
            &["bash", "-c", "zfs destroy bigdata/rehearsal-sandbox-data"],
        ];
        for argv in refused {
            assert!(
                p.authorize(&root, &s(argv)).is_err(),
                "should refuse {:?}",
                argv
            );
        }
        assert!(p.authorize(&root, &[]).is_err());
    }

    #[test]
    fn requires_elevation_refused_when_unprivileged() {
        let p = RootPolicy::parse(ONE_RULE).unwrap();
        let user = ElevationContext::unprivileged(1000);
        let err = p
            .authorize(
                &user,
                &s(&["zfs", "destroy", "bigdata/rehearsal-sandbox-data"]),
            )
            .unwrap_err()
            .to_string();
        assert!(err.contains("requires elevation"), "{err}");
    }

    #[test]
    fn rules_without_positive_restriction_are_rejected_at_load() {
        let err = RootPolicy::parse(
            r#"
[commands.zfs]
binary = "/usr/sbin/zfs"
forbidden_args = ["-r"]
"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("allowed_argv or allowed_patterns"), "{err}");
    }

    #[test]
    fn shells_and_interpreters_cannot_be_targets() {
        for (name, bin) in [
            ("sh", "/bin/sh"),
            ("ok", "/bin/bash"),
            ("ok", "/usr/bin/python3.12"),
            ("ok", "/usr/bin/env"),
            ("perl", "/usr/bin/perl"),
            ("ok", "/usr/bin/xargs"),
            ("ok", "/bin/busybox"),
        ] {
            let text = format!("[commands.{name}]\nbinary = \"{bin}\"\nallowed_argv = [[\"x\"]]\n");
            assert!(RootPolicy::parse(&text).is_err(), "{name} {bin}");
        }
    }

    #[test]
    fn relative_or_dotted_binary_rejected() {
        for bin in ["zfs", "./zfs", "/usr/sbin/../bin/zfs"] {
            let text = format!("[commands.zfs]\nbinary = \"{bin}\"\nallowed_argv = [[\"x\"]]\n");
            assert!(RootPolicy::parse(&text).is_err(), "{bin}");
        }
    }

    #[test]
    fn unknown_fields_rejected() {
        // A typo must not silently drop a restriction.
        let text = "[commands.zfs]\nbinary = \"/usr/sbin/zfs\"\nallowed_args = [[\"x\"]]\nallowed_argv = [[\"x\"]]\n";
        assert!(RootPolicy::parse(text).is_err());
        let text = "[commands.zfs]\nbinary = \"/usr/sbin/zfs\"\nallowed_argv = [[\"x\"]]\ncustom_validator = \"v\"\n";
        assert!(RootPolicy::parse(text).is_err());
        let text = "permissive_mode = true\n[commands.zfs]\nbinary = \"/usr/sbin/zfs\"\nallowed_argv = [[\"x\"]]\n";
        assert!(RootPolicy::parse(text).is_err());
    }

    #[test]
    fn bad_child_path_rejected() {
        for path in [
            "",
            "relative/bin",
            "/usr/bin:.",
            "/usr/bin::/bin",
            "/usr/../tmp",
        ] {
            let text = format!(
                "path = \"{path}\"\n[commands.zfs]\nbinary = \"/usr/sbin/zfs\"\nallowed_argv = [[\"x\"]]\n"
            );
            assert!(RootPolicy::parse(&text).is_err(), "{path:?}");
        }
    }

    #[test]
    fn anchored_patterns_in_root_policy() {
        let p = RootPolicy::parse(
            r#"
[commands.zfs]
binary = "/usr/sbin/zfs"
allowed_patterns = ["list", "-H", "bigdata/rehearsal-[a-z-]+"]
max_args = 3
"#,
        )
        .unwrap();
        let root = ElevationContext::sudo_root("u");
        assert!(p
            .authorize(
                &root,
                &s(&["zfs", "list", "-H", "bigdata/rehearsal-sandbox-data"])
            )
            .is_ok());
        assert!(p
            .authorize(&root, &s(&["zfs", "list", "bigdata/rehearsal-x; rm -rf /"]))
            .is_err());
        assert!(p.authorize(&root, &s(&["zfs", "destroy"])).is_err());
        assert!(
            p.authorize(&root, &s(&["zfs", "list", "-H", "-H", "-H"]))
                .is_err(),
            "max_args still applies"
        );
    }

    /// The shipped example must parse and accept exactly its eight lines.
    #[test]
    fn shipped_aorg_example_policy() {
        let text = include_str!("../../examples/aorg-sandbox-root-policy.toml");
        let p = RootPolicy::parse(text).expect("example policy parses");
        let root = ElevationContext::sudo_root("jdfalk");
        let lines = [
            "/usr/sbin/zfs snapshot bigdata/BD/bigdata/books@rehearsal-sandbox bigdata/BD/bigdata/books/ao-appdata@rehearsal-sandbox",
            "/usr/sbin/zfs destroy bigdata/BD/bigdata/books@rehearsal-sandbox",
            "/usr/sbin/zfs destroy bigdata/BD/bigdata/books/ao-appdata@rehearsal-sandbox",
            "/usr/sbin/zfs destroy bigdata/BD/bigdata/books-sandbox",
            "/usr/sbin/zfs clone bigdata/BD/bigdata/books@rehearsal-sandbox bigdata/BD/bigdata/books-sandbox",
            "/usr/sbin/zfs destroy bigdata/rehearsal-sandbox-data",
            "/usr/sbin/zfs clone -o mountpoint=/mnt/aorg-sandbox/data bigdata/BD/bigdata/books/ao-appdata@rehearsal-sandbox bigdata/rehearsal-sandbox-data",
            "/usr/bin/chown -R 1000:1000 /mnt/aorg-sandbox/data",
        ];
        for line in lines {
            let argv: Vec<String> = line.split(' ').map(String::from).collect();
            let ok = p
                .authorize(&root, &argv)
                .unwrap_or_else(|e| panic!("{line}: {e}"));
            assert_eq!(ok.binary.to_str().unwrap(), argv[0]);
            // Also by rule name.
            let mut by_name = argv.clone();
            by_name[0] = Path::new(&argv[0])
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            assert!(p.authorize(&root, &by_name).is_ok(), "{line} by name");
        }
        let total: usize = p.commands.values().map(|r| r.allowed_argv.len()).sum();
        assert_eq!(
            total,
            lines.len(),
            "example allows exactly the listed commands"
        );
        assert!(p.commands.values().all(|r| r.allowed_patterns.is_empty()));
        assert!(p.commands.values().all(|r| r.requires_elevation));

        let refused: &[&[&str]] = &[
            &["zfs", "destroy", "-r", "bigdata/BD/bigdata/books"],
            &["zfs", "destroy", "bigdata/BD/bigdata/books"],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data; rm -rf /"],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data", "-r"],
            &[
                "zfs",
                "destroy",
                "-R",
                "bigdata/BD/bigdata/books@rehearsal-sandbox",
            ],
            &["chown", "-R", "0:0", "/mnt/aorg-sandbox/data"],
            &["chown", "-R", "1000:1000", "/"],
            &["chown", "-R", "1000:1000", "/mnt/aorg-sandbox/data", "/etc"],
            &[
                "zfs",
                "clone",
                "-o",
                "mountpoint=/",
                "bigdata/BD/bigdata/books/ao-appdata@rehearsal-sandbox",
                "bigdata/rehearsal-sandbox-data",
            ],
        ];
        for argv in refused {
            assert!(
                p.authorize(&root, &s(argv)).is_err(),
                "should refuse {:?}",
                argv
            );
        }
    }

    #[cfg(unix)]
    mod fs_checks {
        use super::super::*;
        use std::os::unix::fs::{symlink, PermissionsExt};

        fn uid() -> u32 {
            // SAFETY: no arguments, cannot fail.
            unsafe { libc::geteuid() }
        }

        fn write_policy(dir: &Path) -> PathBuf {
            let etc = dir.join("etc-safe");
            std::fs::create_dir(&etc).unwrap();
            std::fs::set_permissions(&etc, std::fs::Permissions::from_mode(0o755)).unwrap();
            let f = etc.join("root-policy.toml");
            std::fs::write(&f, super::ONE_RULE).unwrap();
            std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
            f
        }

        fn loader(dir: &Path, f: &Path, trusted_uid: u32) -> RootPolicyLoader {
            RootPolicyLoader {
                path: f.to_path_buf(),
                trusted_uid,
                trust_anchor: Some(dir.to_path_buf()),
            }
        }

        fn tmp() -> tempfile::TempDir {
            let d = tempfile::tempdir().unwrap();
            std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
            d
        }

        #[test]
        fn loads_when_owned_and_not_writable() {
            let d = tmp();
            let f = write_policy(d.path());
            let p = loader(d.path(), &f, uid()).load().unwrap();
            assert!(p.commands.contains_key("zfs"));
        }

        #[test]
        fn refuses_wrong_owner() {
            let d = tmp();
            let f = write_policy(d.path());
            let err = loader(d.path(), &f, uid().wrapping_add(1))
                .load()
                .unwrap_err();
            assert!(err.to_string().contains("owned by uid"), "{err}");
        }

        #[test]
        fn refuses_group_writable_file() {
            let d = tmp();
            let f = write_policy(d.path());
            std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o664)).unwrap();
            let err = loader(d.path(), &f, uid()).load().unwrap_err();
            assert!(err.to_string().contains("writable"), "{err}");
        }

        #[test]
        fn refuses_world_writable_dir() {
            let d = tmp();
            let f = write_policy(d.path());
            std::fs::set_permissions(f.parent().unwrap(), std::fs::Permissions::from_mode(0o777))
                .unwrap();
            let err = loader(d.path(), &f, uid()).load().unwrap_err();
            assert!(err.to_string().contains("writable"), "{err}");
        }

        #[test]
        fn refuses_writable_ancestor() {
            let d = tmp();
            let f = write_policy(d.path());
            std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o757)).unwrap();
            let err = loader(d.path(), &f, uid()).load().unwrap_err();
            assert!(err.to_string().contains("writable"), "{err}");
        }

        #[test]
        fn refuses_symlinked_policy_file() {
            let d = tmp();
            let f = write_policy(d.path());
            let link = f.parent().unwrap().join("link.toml");
            symlink(&f, &link).unwrap();
            assert!(loader(d.path(), &link, uid()).load().is_err());
        }

        #[test]
        fn system_loader_is_fixed() {
            let l = RootPolicyLoader::system();
            assert_eq!(l.path, PathBuf::from(ROOT_POLICY_PATH));
            assert_eq!(l.trusted_uid, 0);
            assert!(l.trust_anchor.is_none());
        }

        #[test]
        fn verify_binary_rejects_shell_symlink() {
            // A root-owned name that resolves to a shell is still refused.
            let d = tmp();
            let link = d.path().join("innocent");
            symlink("/bin/sh", &link).unwrap();
            let err = verify_binary(&link, 0).unwrap_err().to_string();
            assert!(err.contains("shell") || err.contains("owned"), "{err}");
        }

        #[test]
        fn verify_binary_accepts_system_binary() {
            // /bin/ls is root-owned on Linux and macOS.
            let canon = verify_binary(Path::new("/bin/ls"), 0);
            assert!(canon.is_ok(), "{:?}", canon);
        }

        #[test]
        fn verify_binary_rejects_user_owned() {
            let d = tmp();
            let b = d.path().join("tool");
            std::fs::write(&b, "#!/bin/true\n").unwrap();
            std::fs::set_permissions(&b, std::fs::Permissions::from_mode(0o755)).unwrap();
            if uid() != 0 {
                assert!(verify_binary(&b, 0).is_err());
            }
        }
    }
}
