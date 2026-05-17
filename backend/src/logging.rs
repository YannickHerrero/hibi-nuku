use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

/// Initialise structured logging.
///
/// Respects `NUKU_LOG` (RUST_LOG-style filter); falls back to `info`.
pub fn init() {
    let filter = EnvFilter::try_from_env("NUKU_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(false)
        .compact()
        .init();
}
