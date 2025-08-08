// file: src/logger.rs
// version: 1.0.0
// guid: 5a9fbb43-1e0b-4bea-a858-b74b58176503

use crate::error::Result;
use tracing_subscriber::{fmt, EnvFilter};

/// Setup logging for the application
pub fn setup_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    fmt().with_env_filter(filter).with_target(false).init();

    Ok(())
}
