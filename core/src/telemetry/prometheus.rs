use std::sync::Arc;

use crate::telemetry::metrics::NodeMetrics;

/// Render metrics in Prometheus text exposition format.
pub fn render_metrics(node_id: &str, metrics: &Arc<NodeMetrics>) -> String {
    let snap = metrics.snapshot();
    let labels = format!("node_id=\"{}\"", node_id);

    let mut out = String::with_capacity(2048);

    macro_rules! gauge {
        ($name:expr, $help:expr, $value:expr) => {
            out.push_str(&format!(
                "# HELP {} {}\n# TYPE {} gauge\n{}{{{}}} {}\n",
                $name, $help, $name, $name, labels, $value
            ));
        };
    }

    macro_rules! counter {
        ($name:expr, $help:expr, $value:expr) => {
            out.push_str(&format!(
                "# HELP {} {}\n# TYPE {} counter\n{}{{{}}} {}\n",
                $name, $help, $name, $name, labels, $value
            ));
        };
    }

    gauge!("cloakmesh_active_circuits", "Number of currently active circuits", snap.active_circuits);
    counter!("cloakmesh_circuits_built_total", "Total circuits built", snap.total_circuits_built);
    counter!("cloakmesh_circuits_failed_total", "Total circuit build failures", snap.total_circuits_failed);
    counter!("cloakmesh_bytes_relayed_total", "Total bytes relayed", snap.bytes_relayed);
    counter!("cloakmesh_cells_sent_total", "Total cells sent", snap.cells_sent);
    counter!("cloakmesh_cells_received_total", "Total cells received", snap.cells_received);
    counter!("cloakmesh_cover_cells_sent_total", "Total cover traffic cells sent", snap.cover_cells_sent);
    gauge!("cloakmesh_dht_entries", "Current DHT entries", snap.dht_entries);
    counter!("cloakmesh_dht_lookups_total", "Total DHT lookups", snap.dht_lookups);
    counter!("cloakmesh_dht_lookup_failures_total", "Total DHT lookup failures", snap.dht_lookup_failures);
    counter!("cloakmesh_handshakes_completed_total", "Total completed Noise handshakes", snap.handshakes_completed);
    counter!("cloakmesh_handshakes_failed_total", "Total failed Noise handshakes", snap.handshakes_failed);
    counter!("cloakmesh_capability_failures_total", "Total capability verification failures", snap.capability_failures);

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_valid_prometheus_format() {
        let m = NodeMetrics::new();
        m.inc_active_circuits();
        m.inc_cells_sent(false);
        let output = render_metrics("test-node", &m);
        assert!(output.contains("cloakmesh_active_circuits"));
        assert!(output.contains("# TYPE cloakmesh_active_circuits gauge"));
        assert!(output.contains("node_id=\"test-node\""));
    }
}
