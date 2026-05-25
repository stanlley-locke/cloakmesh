use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use std::fs::File;
use std::sync::Arc;

use crate::config::LogLevel;

/// Initialize the global tracing subscriber.
///
/// Dual-logging:
/// - Terminal (stdout): beautiful, color-coded human-readable output
/// - File (core_json.log): strict JSON structured logs for the Gateway & React UI
pub fn init(level: LogLevel, _json: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("cloakmesh={level},warn")));

    let stdout_layer = fmt::layer()
        .pretty()
        .with_filter(filter.clone());

    if let Ok(file) = File::create("core_json.log") {
        let file_layer = fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_writer(Arc::new(file))
            .with_filter(filter);

        tracing_subscriber::registry()
            .with(stdout_layer)
            .with(file_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(stdout_layer)
            .init();
    }
}

/// Create a span for a circuit operation with standard fields.
#[macro_export]
macro_rules! circuit_span {
    ($circuit_id:expr) => {
        tracing::info_span!("circuit", circuit_id = %$circuit_id)
    };
}

/// Create a span for a DHT operation.
#[macro_export]
macro_rules! dht_span {
    ($op:expr, $key:expr) => {
        tracing::info_span!("dht", op = $op, key = %hex::encode($key))
    };
}
