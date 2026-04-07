#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadStreamSnapshot {
    pub graph_stream_mode: &'static str,
    pub linear_stream_mode: &'static str,
    pub graph_stream_access_pattern: &'static str,
    pub linear_stream_access_pattern: &'static str,
    pub pg18_callback_surface_ready: bool,
    pub pg18_scan_wiring_ready: bool,
    pub pg18_vacuum_wiring_ready: bool,
}

pub(crate) fn stream_snapshot() -> ReadStreamSnapshot {
    ReadStreamSnapshot {
        graph_stream_mode: "READ_STREAM_DEFAULT",
        linear_stream_mode: "READ_STREAM_SEQUENTIAL",
        graph_stream_access_pattern: "random",
        linear_stream_access_pattern: "sequential",
        pg18_callback_surface_ready: false,
        pg18_scan_wiring_ready: false,
        pg18_vacuum_wiring_ready: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{stream_snapshot, ReadStreamSnapshot};

    #[test]
    fn stream_snapshot_stays_explicitly_unwired_until_pg18_support_exists() {
        assert_eq!(
            stream_snapshot(),
            ReadStreamSnapshot {
                graph_stream_mode: "READ_STREAM_DEFAULT",
                linear_stream_mode: "READ_STREAM_SEQUENTIAL",
                graph_stream_access_pattern: "random",
                linear_stream_access_pattern: "sequential",
                pg18_callback_surface_ready: false,
                pg18_scan_wiring_ready: false,
                pg18_vacuum_wiring_ready: false,
            }
        );
    }
}
