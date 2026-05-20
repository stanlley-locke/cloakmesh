use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::config::LogLevel;

/// Initialize the global tracing subscriber.
///
/// - In production (JSON mode): structured JSON output suitable for log aggregators.
/// - In development (pretty mode): human-readable colored output.
pub fn init(level: LogLevel, json: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("cloakmesh={level},warn")));

    if json {
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_filter(filter),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .pretty()
                    .with_filter(filter),
            )
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
