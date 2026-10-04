// file: src/config.rs
// version: 1.2.0
// guid: 6ea31d79-e2bf-4304-a841-22bf1e595512
// last-edited: 2026-10-04

use crate::error::{AgentError, Result};
use crate::security::allowlist::{AllowlistConfig, AllowlistOverlay};
use crate::security::elevation::ElevationContext;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

/// Application configuration.
///
/// The optional `allowlist` field, when set, REPLACES the SecurityManager's
/// built-in command list with a richer per-command policy (regex argument
/// patterns, forbidden flags, max-args caps, custom validators). When absent,
/// behavior is identical to pre-1.1 — the legacy hardcoded allowed set is used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub logging: LoggingConfig,
    pub safety: SafetyConfig,
    pub git: GitConfig,
    pub execution: ExecutionConfig,
    /// Optional rich command allowlist. When `None`, defaults are used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowlist: Option<AllowlistConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub working_directory: Option<PathBuf>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file_rotation: bool,
    pub max_log_size: String,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub dry_run: bool,
    pub confirm_destructive: bool,
    pub backup_before_delete: bool,
    pub validate_paths: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_stage: bool,
    pub require_message: bool,
    pub push_hooks: bool,
    pub safe_force_push: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub shell: Option<String>,
    pub environment_isolation: bool,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<u8>,
    pub max_execution_time: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                working_directory: None,
                timeout_seconds: 300,
                max_retries: 3,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: LogFormat::Pretty,
                file_rotation: true,
                max_log_size: "10MB".to_string(),
                retention_days: 30,
            },
            safety: SafetyConfig {
                dry_run: false,
                confirm_destructive: true,
                backup_before_delete: true,
                validate_paths: true,
            },
            git: GitConfig {
                auto_stage: false,
                require_message: true,
                push_hooks: true,
                safe_force_push: true,
            },
            execution: ExecutionConfig {
                shell: None,
                environment_isolation: false,
                resource_limits: ResourceLimits {
                    max_memory_mb: Some(1024),
                    max_cpu_percent: Some(80),
                    max_execution_time: Some(600),
                },
            },
            allowlist: None,
        }
    }
}

impl Config {
    /// Load configuration from default discovery paths (user dir, project file).
    pub async fn load() -> Result<Self> {
        Self::load_with_paths(None, None).await
    }

    /// Load configuration with optional explicit `--config` and
    /// `--policy-overlay` paths. The overlay, when supplied, is applied to the
    /// config's allowlist (or to the default allowlist if the config has none),
    /// and the merged result must be strictly narrower than the base.
    ///
    /// Refuses to run at all when the process is elevated: see
    /// [`Config::load_for_context`].
    pub async fn load_with_paths(
        explicit_config: Option<&Path>,
        overlay_path: Option<&Path>,
    ) -> Result<Self> {
        Self::load_for_context(&ElevationContext::detect(), explicit_config, overlay_path).await
    }

    /// Like [`Config::load_with_paths`], with the elevation context supplied
    /// by the caller.
    ///
    /// When `ctx` is elevated this refuses every source this loader reads:
    /// `--config`, `--policy-overlay`, the user config under `$HOME` (sudo may
    /// keep HOME), `./.safe-ai-util.toml` in a caller-chosen directory, and
    /// the `COPILOT_AGENT_*` environment overrides. An elevated run's only
    /// policy is the fixed root policy read by the `exec` subcommand.
    pub async fn load_for_context(
        ctx: &ElevationContext,
        explicit_config: Option<&Path>,
        overlay_path: Option<&Path>,
    ) -> Result<Self> {
        if ctx.is_elevated() {
            let what = match (explicit_config, overlay_path) {
                (Some(_), _) => "--config",
                (None, Some(_)) => "--policy-overlay",
                (None, None) => "user/project configuration and environment overrides",
            };
            return Err(AgentError::security(format!(
                "{} refused when elevated ({}); only `exec` under the fixed root policy may run",
                what,
                ctx.describe()
            )));
        }

        let mut config = Self::default();

        // Discovery: --config wins over user config wins over project config.
        if let Some(path) = explicit_config {
            info!("Loading explicit configuration from: {}", path.display());
            config = Self::load_from_file(path).await?;
        } else {
            if let Some(user_config) = Self::user_config_path() {
                if user_config.exists() {
                    info!("Loading user configuration from: {}", user_config.display());
                    config = Self::load_from_file(&user_config).await?;
                }
            }

            let project_config = Path::new(".safe-ai-util.toml");
            if project_config.exists() {
                info!(
                    "Loading project configuration from: {}",
                    project_config.display()
                );
                let project_config = Self::load_from_file(project_config).await?;
                config = Self::merge_configs(config, project_config);
            }
        }

        config = Self::apply_env_overrides(config)?;

        // Apply policy overlay last — must narrow whatever allowlist we have.
        if let Some(overlay_path) = overlay_path {
            info!("Applying policy overlay from: {}", overlay_path.display());
            let overlay = Self::load_overlay_from_file(overlay_path).await?;
            let base = config
                .allowlist
                .clone()
                .unwrap_or_else(AllowlistConfig::secure_default);
            let merged = base.apply_overlay(&overlay)?;
            config.allowlist = Some(merged);
        }

        debug!("Final configuration: {:#?}", config);
        Ok(config)
    }

    /// Load an overlay TOML file.
    async fn load_overlay_from_file(path: &Path) -> Result<AllowlistOverlay> {
        let content = fs::read_to_string(path).await.map_err(|e| {
            AgentError::config(format!(
                "Failed to read policy overlay {}: {}",
                path.display(),
                e
            ))
        })?;
        toml::from_str(&content).map_err(|e| {
            AgentError::config(format!(
                "Failed to parse policy overlay {}: {}",
                path.display(),
                e
            ))
        })
    }

    /// Get the user configuration file path
    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("safe-ai-util").join("config.toml"))
    }

    /// Load configuration from a TOML file
    async fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).await.map_err(|e| {
            AgentError::config(format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            ))
        })?;

        toml::from_str(&content).map_err(|e| {
            AgentError::config(format!(
                "Failed to parse config file {}: {}",
                path.display(),
                e
            ))
        })
    }

    /// Merge two configurations, with the second taking precedence
    fn merge_configs(_base: Self, override_config: Self) -> Self {
        // For now, just return the override config
        // In a full implementation, we'd merge each field intelligently
        override_config
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(mut config: Self) -> Result<Self> {
        if let Ok(level) = std::env::var("COPILOT_AGENT_LOG_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(dry_run) = std::env::var("COPILOT_AGENT_DRY_RUN") {
            config.safety.dry_run = dry_run.parse().unwrap_or(false);
        }

        if let Ok(timeout) = std::env::var("COPILOT_AGENT_TIMEOUT") {
            if let Ok(timeout_secs) = timeout.parse::<u64>() {
                config.general.timeout_seconds = timeout_secs;
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_omits_allowlist_field() {
        let cfg = Config::default();
        assert!(cfg.allowlist.is_none());
        let serialized = toml::to_string(&cfg).expect("config serializes");
        // skip_serializing_if + None means the field should not appear.
        assert!(
            !serialized.contains("[allowlist]"),
            "serialized config unexpectedly contained allowlist section: {serialized}"
        );
    }

    #[test]
    fn config_roundtrips_with_allowlist_section() {
        let toml_input = r#"
[general]
working_directory = "/tmp"
timeout_seconds = 60
max_retries = 1

[logging]
level = "warn"
format = "Json"
file_rotation = false
max_log_size = "1MB"
retention_days = 1

[safety]
dry_run = false
confirm_destructive = false
backup_before_delete = false
validate_paths = false

[git]
auto_stage = false
require_message = false
push_hooks = false
safe_force_push = false

[execution]
environment_isolation = false

[execution.resource_limits]

[allowlist]
permissive_mode = false
always_allowed = ["git", "make"]
blocked = ["bash", "curl"]

[allowlist.conditionally_allowed]
"#;
        let cfg: Config = toml::from_str(toml_input).expect("parses");
        let policy = cfg.allowlist.expect("allowlist present");
        assert!(policy.always_allowed.contains("git"));
        assert!(policy.always_allowed.contains("make"));
        assert!(policy.blocked.contains("bash"));
        assert!(!policy.permissive_mode);
    }

    #[tokio::test]
    async fn elevated_refuses_explicit_config() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("permissive.toml");
        std::fs::write(&cfg, "[allowlist]\npermissive_mode = true\n").unwrap();
        let err = Config::load_for_context(&ElevationContext::sudo_root("u"), Some(&cfg), None)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("--config"), "{err}");
    }

    #[tokio::test]
    async fn elevated_refuses_overlay() {
        let dir = tempfile::tempdir().unwrap();
        let ov = dir.path().join("overlay.toml");
        std::fs::write(&ov, "permissive_mode = false\n").unwrap();
        let err = Config::load_for_context(&ElevationContext::sudo_root("u"), None, Some(&ov))
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("--policy-overlay"), "{err}");
    }

    #[tokio::test]
    async fn elevated_refuses_default_discovery() {
        let mut ctx = ElevationContext::unprivileged(1000);
        ctx.euid = 0;
        let err = Config::load_for_context(&ctx, None, None)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("refused when elevated"), "{err}");
    }

    #[tokio::test]
    async fn unprivileged_overlay_still_applies() {
        let dir = tempfile::tempdir().unwrap();
        let ov = dir.path().join("overlay.toml");
        std::fs::write(&ov, "blocked = [\"make\"]\n").unwrap();
        let cfg = Config::load_for_context(&ElevationContext::unprivileged(1000), None, Some(&ov))
            .await
            .unwrap();
        assert!(cfg.allowlist.unwrap().blocked.contains("make"));
    }
}
