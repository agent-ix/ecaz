#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExplainOptionSnapshot {
    pub option_name: &'static str,
    pub pg18_custom_explain_option_ready: bool,
    pub pg18_explain_per_node_hook_ready: bool,
}

pub(crate) fn explain_option_snapshot() -> ExplainOptionSnapshot {
    ExplainOptionSnapshot {
        option_name: "tqvector",
        pg18_custom_explain_option_ready: false,
        pg18_explain_per_node_hook_ready: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{explain_option_snapshot, ExplainOptionSnapshot};

    #[test]
    fn explain_option_snapshot_stays_explicitly_unwired_until_pg18_support_exists() {
        assert_eq!(
            explain_option_snapshot(),
            ExplainOptionSnapshot {
                option_name: "tqvector",
                pg18_custom_explain_option_ready: false,
                pg18_explain_per_node_hook_ready: false,
            }
        );
    }
}
