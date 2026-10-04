// file: src/commands/exec.rs
// version: 1.0.0
// guid: 7c1a5e93-2b8d-4f60-9e4a-3d6b0f2c8e17
// last-edited: 2026-10-04

//! `exec`: run one allowlisted binary with an exact argv under the fixed root
//! policy. This is the only subcommand an elevated invocation may use.
//!
//! * The policy comes only from [`ROOT_POLICY_PATH`] (ownership-checked).
//! * The binary is the absolute path named in the policy, symlink-resolved and
//!   ownership-checked; PATH is never consulted.
//! * argv is passed straight to execve: no shell, no splitting, no rewriting.
//! * The child gets an empty environment plus the policy's fixed PATH, `/` as
//!   its working directory and `/dev/null` as stdin.
//! * Every allowed and refused invocation is logged to syslog (authpriv) and,
//!   for anything that would execute, to the policy's root-owned log file.
//!   A run is refused if that file cannot be written.
//!
//! [`ROOT_POLICY_PATH`]: crate::security::root_policy::ROOT_POLICY_PATH

use crate::security::elevation::{check_cli_request, CliRequest, ElevationContext};
use crate::security::root_policy::{verify_binary, AuthorizedExec, RootPolicy, RootPolicyLoader};
use clap::{Arg, ArgMatches, Command};
use serde::Serialize;
use std::io::{self, Write};
use std::path::Path;

/// Exit status when the gate or the policy refuses the invocation. Distinct
/// from a child's own non-zero exit so callers can tell "not allowed" from
/// "zfs failed".
pub const EXIT_REFUSED: i32 = 77; // EX_NOPERM
/// Exit status when the root policy cannot be loaded or is unsafe.
pub const EXIT_POLICY: i32 = 78; // EX_CONFIG
/// Exit status when the audit log cannot be written (nothing is run).
pub const EXIT_AUDIT: i32 = 74; // EX_IOERR
/// Exit status when an allowed child could not be started.
pub const EXIT_SPAWN: i32 = 71; // EX_OSERR

/// Build the `exec` subcommand. All remaining arguments, including ones that
/// start with `-`, are captured verbatim as the argv to authorize.
pub fn build_command() -> Command {
    Command::new("exec")
        .about("Run one allowlisted binary with an exact argv under the fixed root policy")
        .long_about(
            "Run one binary named in /etc/safe-ai-util/root-policy.toml with an argv that \
             exactly matches the policy. Intended to be the only command in a sudoers rule. \
             --config, --policy-overlay and --args-file are refused. Without sudo, only \
             --dry-run is accepted (validate and print, never run).",
        )
        .arg(
            Arg::new("argv")
                .value_name("COMMAND [ARGS]...")
                .help("Rule name or the rule's absolute binary path, followed by its arguments")
                .num_args(1..)
                .required(true)
                .trailing_var_arg(true)
                .allow_hyphen_values(true),
        )
}

/// One audit record. Serialized as a single JSON line, so arguments with
/// embedded newlines cannot forge extra records.
#[derive(Debug, Clone, Serialize)]
pub struct ExecLogEntry<'a> {
    pub ts: String,
    pub pid: u32,
    pub event: &'a str,
    pub euid: u32,
    pub ruid: u32,
    pub sudo_user: Option<&'a str>,
    pub sudo_uid: Option<&'a str>,
    pub argv: &'a [String],
    pub rule: Option<&'a str>,
    pub binary: Option<String>,
    pub reason: Option<String>,
    pub exit_code: Option<i32>,
    pub dry_run: bool,
}

impl<'a> ExecLogEntry<'a> {
    fn new(ctx: &'a ElevationContext, argv: &'a [String], event: &'a str, dry_run: bool) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            pid: std::process::id(),
            event,
            euid: ctx.euid,
            ruid: ctx.ruid,
            sudo_user: ctx.sudo_user.as_deref(),
            sudo_uid: ctx.sudo_uid.as_deref(),
            argv,
            rule: None,
            binary: None,
            reason: None,
            exit_code: None,
            dry_run,
        }
    }

    fn to_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{\"event\":\"unserializable\"}".into())
    }

    fn is_refusal(&self) -> bool {
        self.event.starts_with("refused") || self.event == "error"
    }
}

/// Where audit records go.
pub trait AuditSink {
    /// Record one entry. An error means the record was not durably written.
    fn record(&mut self, entry: &ExecLogEntry<'_>) -> io::Result<()>;
}

/// Best-effort syslog plus an optional mandatory log file.
pub struct SystemAuditSink {
    file: Option<std::fs::File>,
}

impl SystemAuditSink {
    /// Syslog only (used for refusals before the policy is loaded and for dry
    /// runs, which never execute anything).
    pub fn syslog_only() -> Self {
        Self { file: None }
    }

    /// Syslog plus the policy's log file, opened append-only after checking
    /// that its directory chain and the file itself are root-owned and not
    /// group/other-writable.
    pub fn with_root_log(path: &Path) -> io::Result<Self> {
        Ok(Self {
            file: Some(open_root_log(path)?),
        })
    }
}

impl AuditSink for SystemAuditSink {
    fn record(&mut self, entry: &ExecLogEntry<'_>) -> io::Result<()> {
        let line = entry.to_line();
        syslog_send(entry.is_refusal(), &line);
        if let Some(f) = self.file.as_mut() {
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
            f.sync_data()?;
        }
        Ok(())
    }
}

#[cfg(unix)]
fn open_root_log(path: &Path) -> io::Result<std::fs::File> {
    use crate::security::root_policy::verify_dir_chain;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let to_io = |e: crate::error::AgentError| {
        io::Error::new(io::ErrorKind::PermissionDenied, e.to_string())
    };
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "log path has no parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "log path has no file name"))?;
    let dir = verify_dir_chain(parent, 0, None).map_err(to_io)?;
    let f = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .mode(0o640)
        .custom_flags(libc::O_NOFOLLOW)
        .open(dir.join(name))?;
    let meta = f.metadata()?;
    if !meta.is_file() || meta.uid() != 0 || meta.mode() & 0o022 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "{} must be a root-owned file not writable by group or other",
                path.display()
            ),
        ));
    }
    Ok(f)
}

#[cfg(not(unix))]
fn open_root_log(_path: &Path) -> io::Result<std::fs::File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "exec is only supported on unix",
    ))
}

#[cfg(unix)]
fn syslog_send(warning: bool, line: &str) {
    use std::os::unix::net::UnixDatagram;
    // facility authpriv (10); severity warning (4) or notice (5).
    let pri = 10 * 8 + if warning { 4 } else { 5 };
    let msg = format!("<{}>safe-ai-util[{}]: {}", pri, std::process::id(), line);
    if let Ok(sock) = UnixDatagram::unbound() {
        for target in ["/dev/log", "/var/run/syslog"] {
            if sock.send_to(msg.as_bytes(), target).is_ok() {
                return;
            }
        }
    }
}

#[cfg(not(unix))]
fn syslog_send(_warning: bool, _line: &str) {}

/// Exit status of a finished child, as a shell would report it.
pub type ChildExit = i32;

/// The child process: exact argv, no shell, empty environment except the
/// policy PATH, `/` as working directory, `/dev/null` as stdin.
fn child_command(binary: &Path, auth: &AuthorizedExec) -> std::process::Command {
    let mut cmd = std::process::Command::new(binary);
    cmd.args(&auth.args)
        .env_clear()
        .env("PATH", &auth.child_path)
        .current_dir("/")
        .stdin(std::process::Stdio::null());
    cmd
}

/// Start the authorized child and wait for it.
pub fn spawn_child(binary: &Path, auth: &AuthorizedExec) -> io::Result<ChildExit> {
    let status = child_command(binary, auth).status()?;
    if let Some(code) = status.code() {
        return Ok(code);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return Ok(128 + sig);
        }
    }
    Ok(1)
}

/// Everything `run_exec` needs from the outside world, so tests can drive the
/// whole flow without root, a real `/etc` policy or a real child.
pub struct ExecEnv<'a> {
    pub loader: &'a RootPolicyLoader,
    /// Open the sink used once the policy is loaded and the run would execute.
    pub open_sink: &'a dyn Fn(&RootPolicy) -> io::Result<Box<dyn AuditSink>>,
    /// Sink for refusals before the policy is loaded, and for dry runs.
    pub early_sink: &'a mut dyn AuditSink,
    /// Resolve and check the binary; returns the path to execute.
    pub verify: &'a dyn Fn(&Path) -> crate::Result<std::path::PathBuf>,
    pub spawn: &'a dyn Fn(&Path, &AuthorizedExec) -> io::Result<ChildExit>,
}

fn refuse(sink: &mut dyn AuditSink, mut entry: ExecLogEntry<'_>, reason: String, code: i32) -> i32 {
    eprintln!("safe-ai-util: REFUSED: {}", reason);
    entry.reason = Some(reason);
    let _ = sink.record(&entry);
    code
}

/// Authorize and (unless `dry_run`) run `argv`. Returns the process exit code.
pub fn run_exec(ctx: &ElevationContext, argv: &[String], dry_run: bool, env: ExecEnv<'_>) -> i32 {
    if !ctx.is_elevated() && !dry_run {
        let e = ExecLogEntry::new(ctx, argv, "refused", dry_run);
        return refuse(
            env.early_sink,
            e,
            "exec runs only under sudo; use --dry-run to check a command without sudo".into(),
            EXIT_REFUSED,
        );
    }

    let policy = match env.loader.load() {
        Ok(p) => p,
        Err(err) => {
            let e = ExecLogEntry::new(ctx, argv, "refused-policy", dry_run);
            return refuse(
                env.early_sink,
                e,
                format!("root policy {}: {}", env.loader.path.display(), err),
                EXIT_POLICY,
            );
        }
    };

    // Dry runs never execute, so they do not need the durable log.
    let mut late_sink: Option<Box<dyn AuditSink>> = None;
    if !dry_run {
        match (env.open_sink)(&policy) {
            Ok(s) => late_sink = Some(s),
            Err(err) => {
                let e = ExecLogEntry::new(ctx, argv, "refused-audit", dry_run);
                return refuse(
                    env.early_sink,
                    e,
                    format!("cannot open audit log {}: {}", policy.log_path(), err),
                    EXIT_AUDIT,
                );
            }
        }
    }
    let sink: &mut dyn AuditSink = match late_sink.as_mut() {
        Some(s) => s.as_mut(),
        None => env.early_sink,
    };

    let auth = match policy.authorize(ctx, argv) {
        Ok(a) => a,
        Err(err) => {
            let e = ExecLogEntry::new(ctx, argv, "refused", dry_run);
            return refuse(sink, e, err.to_string(), EXIT_REFUSED);
        }
    };

    let binary = match (env.verify)(&auth.binary) {
        Ok(b) => b,
        Err(err) => {
            let mut e = ExecLogEntry::new(ctx, argv, "refused-binary", dry_run);
            e.rule = Some(&auth.rule);
            return refuse(sink, e, err.to_string(), EXIT_REFUSED);
        }
    };

    let mut allowed = ExecLogEntry::new(
        ctx,
        argv,
        if dry_run {
            "dry-run-allowed"
        } else {
            "allowed"
        },
        dry_run,
    );
    allowed.rule = Some(&auth.rule);
    allowed.binary = Some(binary.display().to_string());
    if let Err(err) = sink.record(&allowed) {
        eprintln!("safe-ai-util: REFUSED: audit log write failed: {}", err);
        return EXIT_AUDIT;
    }

    if dry_run {
        println!("ALLOWED (dry run): {} {:?}", binary.display(), auth.args);
        return 0;
    }

    match (env.spawn)(&binary, &auth) {
        Ok(code) => {
            let mut done = ExecLogEntry::new(ctx, argv, "completed", dry_run);
            done.rule = Some(&auth.rule);
            done.binary = Some(binary.display().to_string());
            done.exit_code = Some(code);
            let _ = sink.record(&done);
            code
        }
        Err(err) => {
            let mut e = ExecLogEntry::new(ctx, argv, "error", dry_run);
            e.rule = Some(&auth.rule);
            e.reason = Some(format!("spawn failed: {}", err));
            let _ = sink.record(&e);
            eprintln!(
                "safe-ai-util: failed to start {}: {}",
                binary.display(),
                err
            );
            EXIT_SPAWN
        }
    }
}

fn exec_argv(matches: &ArgMatches) -> Vec<String> {
    matches
        .get_many::<String>("argv")
        .map(|v| v.cloned().collect())
        .unwrap_or_default()
}

fn system_env_run(ctx: &ElevationContext, argv: &[String], dry_run: bool) -> i32 {
    let loader = RootPolicyLoader::system();
    let mut early = SystemAuditSink::syslog_only();
    let open_sink = |p: &RootPolicy| -> io::Result<Box<dyn AuditSink>> {
        Ok(Box::new(SystemAuditSink::with_root_log(Path::new(
            p.log_path(),
        ))?))
    };
    let verify = |b: &Path| verify_binary(b, 0);
    run_exec(
        ctx,
        argv,
        dry_run,
        ExecEnv {
            loader: &loader,
            open_sink: &open_sink,
            early_sink: &mut early,
            verify: &verify,
            spawn: &spawn_child,
        },
    )
}

/// Gate an elevated invocation and run it. Called from `main` before logging,
/// config discovery or the executor are set up, so nothing caller-controlled
/// (cwd config, `$HOME` config, `SAFE_AI_UTIL_*` paths, `--config`) is read.
/// Returns the process exit code.
pub fn run_elevated(ctx: &ElevationContext, matches: &ArgMatches) -> i32 {
    let req = CliRequest {
        config: matches.get_one::<String>("config").map(String::as_str),
        policy_overlay: matches
            .get_one::<String>("policy-overlay")
            .map(String::as_str),
        args_file: matches.get_one::<String>("args-file").map(String::as_str),
        subcommand: matches.subcommand_name(),
    };
    if let Err(err) = check_cli_request(ctx, &req) {
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let e = ExecLogEntry::new(ctx, &argv, "refused-gate", false);
        return refuse(
            &mut SystemAuditSink::syslog_only(),
            e,
            err.to_string(),
            EXIT_REFUSED,
        );
    }
    let Some(("exec", sub)) = matches.subcommand() else {
        // check_cli_request already guarantees `exec`.
        return EXIT_REFUSED;
    };
    let dry_run = matches.get_flag("dry-run");
    system_env_run(ctx, &exec_argv(sub), dry_run)
}

/// `exec` reached without elevation. It still uses only the fixed root
/// policy; `--config`/`--policy-overlay`/`--args-file` are refused so a dry
/// run can never report a decision the real (sudo) run would not make.
pub fn run_unprivileged(ctx: &ElevationContext, matches: &ArgMatches, sub: &ArgMatches) -> i32 {
    for flag in ["config", "policy-overlay", "args-file"] {
        if matches.get_one::<String>(flag).is_some() {
            eprintln!("safe-ai-util: REFUSED: --{} is not accepted by exec", flag);
            return EXIT_REFUSED;
        }
    }
    system_env_run(ctx, &exec_argv(sub), matches.get_flag("dry-run"))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::security::root_policy::RootPolicyLoader;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

    const POLICY: &str = r#"
log_file = "/var/log/safe-ai-util/test.log"
[commands.zfs]
binary = "/usr/sbin/zfs"
requires_elevation = true
allowed_argv = [["destroy", "bigdata/rehearsal-sandbox-data"]]
"#;

    #[derive(Clone, Default)]
    struct MemSink {
        lines: Rc<RefCell<Vec<String>>>,
        fail: bool,
    }
    impl AuditSink for MemSink {
        fn record(&mut self, e: &ExecLogEntry<'_>) -> io::Result<()> {
            if self.fail {
                return Err(io::Error::new(io::ErrorKind::Other, "disk full"));
            }
            self.lines.borrow_mut().push(e.to_line());
            Ok(())
        }
    }

    type SpawnLog = Rc<RefCell<Vec<(PathBuf, Vec<String>, String)>>>;

    struct Harness {
        _dir: tempfile::TempDir,
        loader: RootPolicyLoader,
        early: MemSink,
        late: MemSink,
        spawned: SpawnLog,
    }

    impl Harness {
        fn new(policy: &str) -> Self {
            use std::os::unix::fs::PermissionsExt;
            let dir = tempfile::tempdir().unwrap();
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
            let f = dir.path().join("root-policy.toml");
            std::fs::write(&f, policy).unwrap();
            std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
            // SAFETY: no arguments, cannot fail.
            let uid = unsafe { libc::geteuid() };
            Self {
                loader: RootPolicyLoader {
                    path: f,
                    trusted_uid: uid,
                    trust_anchor: Some(dir.path().to_path_buf()),
                },
                _dir: dir,
                early: MemSink::default(),
                late: MemSink::default(),
                spawned: Rc::default(),
            }
        }

        fn run(&mut self, ctx: &ElevationContext, argv: &[&str], dry_run: bool) -> i32 {
            let argv: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
            let late = self.late.clone();
            let open_sink = move |_: &RootPolicy| -> io::Result<Box<dyn AuditSink>> {
                Ok(Box::new(late.clone()))
            };
            let verify = |b: &Path| Ok(b.to_path_buf());
            let spawned = self.spawned.clone();
            let spawn = move |b: &Path, a: &AuthorizedExec| -> io::Result<ChildExit> {
                spawned
                    .borrow_mut()
                    .push((b.to_path_buf(), a.args.clone(), a.child_path.clone()));
                Ok(0)
            };
            let mut early = self.early.clone();
            run_exec(
                ctx,
                &argv,
                dry_run,
                ExecEnv {
                    loader: &self.loader,
                    open_sink: &open_sink,
                    early_sink: &mut early,
                    verify: &verify,
                    spawn: &spawn,
                },
            )
        }
    }

    #[test]
    fn allowed_runs_exact_argv_and_logs() {
        let mut h = Harness::new(POLICY);
        let root = ElevationContext::sudo_root("jdfalk");
        let code = h.run(
            &root,
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data"],
            false,
        );
        assert_eq!(code, 0);
        let spawned = h.spawned.borrow();
        assert_eq!(spawned.len(), 1);
        assert_eq!(spawned[0].0, PathBuf::from("/usr/sbin/zfs"));
        assert_eq!(
            spawned[0].1,
            vec!["destroy", "bigdata/rehearsal-sandbox-data"]
        );
        assert_eq!(
            spawned[0].2,
            crate::security::root_policy::DEFAULT_CHILD_PATH
        );
        let lines = h.late.lines.borrow();
        assert!(lines[0].contains("\"event\":\"allowed\""), "{lines:?}");
        assert!(lines[0].contains("\"sudo_user\":\"jdfalk\""));
        assert!(lines[1].contains("\"event\":\"completed\""));
    }

    #[test]
    fn refusals_never_spawn_and_are_logged() {
        let root = ElevationContext::sudo_root("jdfalk");
        for argv in [
            &["zfs", "destroy", "-r", "bigdata/BD/bigdata/books"][..],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data; rm -rf /"],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data", "-r"],
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data", "extra"],
            &["zfs", "destroy", "bigdata/other"],
            &["sh", "-c", "id"],
        ] {
            let mut h = Harness::new(POLICY);
            assert_eq!(h.run(&root, argv, false), EXIT_REFUSED, "{argv:?}");
            assert!(h.spawned.borrow().is_empty());
            let lines = h.late.lines.borrow();
            assert_eq!(lines.len(), 1);
            assert!(lines[0].contains("\"event\":\"refused\""), "{lines:?}");
        }
    }

    #[test]
    fn unprivileged_without_dry_run_is_refused() {
        let mut h = Harness::new(POLICY);
        let user = ElevationContext::unprivileged(1000);
        let code = h.run(
            &user,
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data"],
            false,
        );
        assert_eq!(code, EXIT_REFUSED);
        assert!(h.spawned.borrow().is_empty());
        assert_eq!(h.early.lines.borrow().len(), 1);
    }

    #[test]
    fn dry_run_never_spawns() {
        let mut h = Harness::new(POLICY);
        let root = ElevationContext::sudo_root("u");
        let code = h.run(
            &root,
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data"],
            true,
        );
        assert_eq!(code, 0);
        assert!(h.spawned.borrow().is_empty());
        assert!(h.early.lines.borrow()[0].contains("dry-run-allowed"));
    }

    #[test]
    fn audit_failure_blocks_execution() {
        let mut h = Harness::new(POLICY);
        h.late.fail = true;
        let root = ElevationContext::sudo_root("u");
        let code = h.run(
            &root,
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data"],
            false,
        );
        assert_eq!(code, EXIT_AUDIT);
        assert!(h.spawned.borrow().is_empty());
    }

    #[test]
    fn unsafe_policy_file_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let mut h = Harness::new(POLICY);
        std::fs::set_permissions(&h.loader.path, std::fs::Permissions::from_mode(0o666)).unwrap();
        let root = ElevationContext::sudo_root("u");
        let code = h.run(
            &root,
            &["zfs", "destroy", "bigdata/rehearsal-sandbox-data"],
            false,
        );
        assert_eq!(code, EXIT_POLICY);
        assert!(h.spawned.borrow().is_empty());
    }

    #[test]
    fn log_lines_escape_newlines() {
        let ctx = ElevationContext::sudo_root("u");
        let argv = vec!["zfs\n{\"event\":\"allowed\"}".to_string()];
        let e = ExecLogEntry::new(&ctx, &argv, "refused", false);
        assert!(!e.to_line().contains('\n'));
    }

    #[test]
    fn spawn_child_propagates_exit_code() {
        let bin = ["/usr/bin/false", "/bin/false"]
            .into_iter()
            .find(|p| Path::new(p).exists())
            .unwrap();
        let auth = AuthorizedExec {
            rule: "t".into(),
            binary: PathBuf::from(bin),
            args: vec![],
            child_path: "/usr/bin:/bin".into(),
        };
        assert_eq!(spawn_child(Path::new(bin), &auth).unwrap(), 1);
    }

    #[test]
    fn child_env_is_cleared_and_argv_is_verbatim() {
        std::env::set_var("SAFE_AI_UTIL_TEST_LEAK", "1");
        let auth = AuthorizedExec {
            rule: "t".into(),
            binary: PathBuf::from("/usr/bin/env"),
            args: vec![],
            child_path: "/usr/bin:/bin".into(),
        };
        let out = child_command(Path::new("/usr/bin/env"), &auth)
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        assert_eq!(text.trim(), "PATH=/usr/bin:/bin", "child env: {text}");

        // An argument containing shell metacharacters reaches the child as
        // one argv element, unexpanded.
        let auth = AuthorizedExec {
            rule: "t".into(),
            binary: PathBuf::from("/bin/echo"),
            args: vec!["a; rm -rf / $(id)".into()],
            child_path: "/usr/bin:/bin".into(),
        };
        let out = child_command(Path::new("/bin/echo"), &auth)
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout), "a; rm -rf / $(id)\n");
    }
}
