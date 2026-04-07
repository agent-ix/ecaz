#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExplainOptionSnapshot {
    pub option_name: &'static str,
    pub pg18_custom_explain_option_ready: bool,
    pub pg18_explain_per_node_hook_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExplainCounterDefinition {
    pub counter_name: &'static str,
    pub counter_type: &'static str,
    pub increments_when: &'static str,
}

const EXPLAIN_COUNTER_DEFINITIONS: [ExplainCounterDefinition; 7] = [
    ExplainCounterDefinition {
        counter_name: "stats_bootstrap_expansions",
        counter_type: "u32",
        increments_when: "a bootstrap frontier candidate is expanded",
    },
    ExplainCounterDefinition {
        counter_name: "stats_bootstrap_pages_read",
        counter_type: "u32",
        increments_when: "a page is read during bootstrap phase",
    },
    ExplainCounterDefinition {
        counter_name: "stats_linear_pages_read",
        counter_type: "u32",
        increments_when: "a page is read during linear scan phase",
    },
    ExplainCounterDefinition {
        counter_name: "stats_elements_scored",
        counter_type: "u32",
        increments_when: "an element is scored via PreparedQuery",
    },
    ExplainCounterDefinition {
        counter_name: "stats_elements_skipped",
        counter_type: "u32",
        increments_when: "an element is skipped (deleted or already emitted)",
    },
    ExplainCounterDefinition {
        counter_name: "stats_heap_tids_returned",
        counter_type: "u32",
        increments_when: "a heap TID is returned via amgettuple",
    },
    ExplainCounterDefinition {
        counter_name: "stats_quantizer_cache_hit",
        counter_type: "bool",
        increments_when: "ProdQuantizer was reused from cache",
    },
];

pub(crate) fn explain_option_snapshot() -> ExplainOptionSnapshot {
    ExplainOptionSnapshot {
        option_name: "tqvector",
        pg18_custom_explain_option_ready: false,
        pg18_explain_per_node_hook_ready: false,
    }
}

pub(crate) fn explain_counter_definitions() -> &'static [ExplainCounterDefinition] {
    &EXPLAIN_COUNTER_DEFINITIONS
}

#[cfg(test)]
mod tests {
    use super::{
        explain_counter_definitions, explain_option_snapshot, ExplainCounterDefinition,
        ExplainOptionSnapshot,
    };

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

    #[test]
    fn explain_counter_definitions_match_the_staged_fr024_contract() {
        assert_eq!(
            explain_counter_definitions(),
            &[
                ExplainCounterDefinition {
                    counter_name: "stats_bootstrap_expansions",
                    counter_type: "u32",
                    increments_when: "a bootstrap frontier candidate is expanded",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_bootstrap_pages_read",
                    counter_type: "u32",
                    increments_when: "a page is read during bootstrap phase",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_linear_pages_read",
                    counter_type: "u32",
                    increments_when: "a page is read during linear scan phase",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_elements_scored",
                    counter_type: "u32",
                    increments_when: "an element is scored via PreparedQuery",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_elements_skipped",
                    counter_type: "u32",
                    increments_when: "an element is skipped (deleted or already emitted)",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_heap_tids_returned",
                    counter_type: "u32",
                    increments_when: "a heap TID is returned via amgettuple",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_quantizer_cache_hit",
                    counter_type: "bool",
                    increments_when: "ProdQuantizer was reused from cache",
                },
            ]
        );
    }
}
