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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TqExplainCounters {
    pub stats_bootstrap_expansions: u32,
    pub stats_bootstrap_pages_read: u32,
    pub stats_linear_pages_read: u32,
    pub stats_elements_scored: u32,
    pub stats_elements_skipped: u32,
    pub stats_heap_tids_returned: u32,
    pub stats_quantizer_cache_hit: bool,
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

impl TqExplainCounters {
    pub(crate) fn record_bootstrap_expansion(&mut self) {
        self.stats_bootstrap_expansions += 1;
    }

    pub(crate) fn record_bootstrap_page_read(&mut self) {
        self.stats_bootstrap_pages_read += 1;
    }

    pub(crate) fn record_linear_page_read(&mut self) {
        self.stats_linear_pages_read += 1;
    }

    pub(crate) fn record_element_scored(&mut self) {
        self.stats_elements_scored += 1;
    }

    pub(crate) fn record_element_skipped(&mut self) {
        self.stats_elements_skipped += 1;
    }

    pub(crate) fn record_heap_tid_returned(&mut self) {
        self.stats_heap_tids_returned += 1;
    }

    pub(crate) fn record_quantizer_cache_hit(&mut self) {
        self.stats_quantizer_cache_hit = true;
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        explain_counter_definitions, explain_option_snapshot, ExplainCounterDefinition,
        ExplainOptionSnapshot, TqExplainCounters,
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

    #[test]
    fn explain_counters_record_each_staged_statistic() {
        let mut counters = TqExplainCounters::default();

        counters.record_bootstrap_expansion();
        counters.record_bootstrap_page_read();
        counters.record_linear_page_read();
        counters.record_element_scored();
        counters.record_element_skipped();
        counters.record_heap_tid_returned();
        counters.record_quantizer_cache_hit();

        assert_eq!(
            counters,
            TqExplainCounters {
                stats_bootstrap_expansions: 1,
                stats_bootstrap_pages_read: 1,
                stats_linear_pages_read: 1,
                stats_elements_scored: 1,
                stats_elements_skipped: 1,
                stats_heap_tids_returned: 1,
                stats_quantizer_cache_hit: true,
            }
        );
    }

    #[test]
    fn explain_counters_reset_back_to_zero_state() {
        let mut counters = TqExplainCounters {
            stats_bootstrap_expansions: 2,
            stats_bootstrap_pages_read: 3,
            stats_linear_pages_read: 5,
            stats_elements_scored: 7,
            stats_elements_skipped: 11,
            stats_heap_tids_returned: 13,
            stats_quantizer_cache_hit: true,
        };

        counters.reset();

        assert_eq!(counters, TqExplainCounters::default());
    }
}
