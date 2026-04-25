#[cfg(feature = "pg18")]
use std::ffi::{c_void, CStr, CString};
#[cfg(feature = "pg18")]
use std::ptr;
#[cfg(feature = "pg18")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "pg18")]
use std::sync::OnceLock;

#[cfg(feature = "pg18")]
use pgrx::pg_sys;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExplainPropertyValue {
    Integer(u32),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExplainProperty {
    pub property_name: &'static str,
    pub value: ExplainPropertyValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExplainOutputGroup {
    pub group_label: &'static str,
    pub opened_with: &'static str,
    pub closed_with: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExplainNodeKind {
    IndexScan,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExplainHookContext<'a> {
    pub explain_option_enabled: bool,
    pub node_kind: ExplainNodeKind,
    pub access_method_name: &'a str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TqExplainCounters {
    pub stats_bootstrap_expansions: u32,
    pub stats_bootstrap_pages_read: u32,
    pub stats_linear_pages_read: u32,
    pub stats_elements_scored: u32,
    pub stats_elements_skipped: u32,
    pub stats_heap_tids_returned: u32,
    pub stats_parallel_handoffs_foreign_selected_pending: u32,
    pub stats_parallel_handoffs_foreign_admitted_head: u32,
    pub stats_parallel_contributor_hidden_publishes: u32,
    pub stats_parallel_contributor_publish_missing_hidden: u32,
    pub stats_parallel_contributor_publish_duplicate_active: u32,
    pub stats_parallel_contributor_publish_handoff_ready: u32,
    pub stats_parallel_contributor_publish_ordered_after_visible: u32,
    pub stats_parallel_contributor_publish_no_visible_owner: u32,
    pub stats_parallel_contributor_duplicate_retires: u32,
    pub stats_parallel_contributor_output_limit_exits: u32,
    pub stats_parallel_contributor_poll_limit_exits: u32,
    pub stats_parallel_contributor_poll_limit_missing_hidden: u32,
    pub stats_parallel_contributor_poll_limit_duplicate_active: u32,
    pub stats_parallel_contributor_poll_limit_handoff_ready: u32,
    pub stats_parallel_contributor_poll_limit_ordered_after_visible: u32,
    pub stats_parallel_contributor_poll_limit_no_visible_owner: u32,
    pub stats_parallel_blocked_foreign_selected_pending: u32,
    pub stats_parallel_blocked_foreign_admitted_head: u32,
    pub stats_parallel_blocked_admission_window: u32,
    pub stats_parallel_local_only_emits: u32,
    pub stats_parallel_local_only_emits_foreign_selected_pending: u32,
    pub stats_parallel_local_only_emits_foreign_admitted_head: u32,
    pub stats_parallel_deferred_local_emits: u32,
    pub stats_parallel_deferred_local_emits_foreign_selected_pending: u32,
    pub stats_parallel_deferred_local_emits_foreign_admitted_head: u32,
    pub stats_quantizer_cache_hit: bool,
}

const EXPLAIN_COUNTER_DEFINITIONS: [ExplainCounterDefinition; 32] = [
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
        counter_name: "stats_parallel_handoffs_foreign_selected_pending",
        counter_type: "u32",
        increments_when:
            "a worker drains a foreign selected pending output through the shared handoff seam",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_handoffs_foreign_admitted_head",
        counter_type: "u32",
        increments_when:
            "a worker drains a foreign admitted head through the shared handoff seam",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_hidden_publishes",
        counter_type: "u32",
        increments_when:
            "a non-emitting contributor publishes hidden output behind the elected emitter",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_publish_missing_hidden",
        counter_type: "u32",
        increments_when:
            "a hidden contributor publish finds no still-published hidden row to diagnose",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_publish_duplicate_active",
        counter_type: "u32",
        increments_when:
            "a hidden contributor publish matches an active or emitted visible heap TID",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_publish_handoff_ready",
        counter_type: "u32",
        increments_when:
            "a hidden contributor publish orders before the visible owner row",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_publish_ordered_after_visible",
        counter_type: "u32",
        increments_when:
            "a hidden contributor publish orders after the visible owner row",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_publish_no_visible_owner",
        counter_type: "u32",
        increments_when:
            "a hidden contributor publish finds no selected or admitted visible owner row to compare",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_duplicate_retires",
        counter_type: "u32",
        increments_when:
            "a non-emitting contributor retires a hidden row whose next heap TID was already emitted",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_output_limit_exits",
        counter_type: "u32",
        increments_when: "a non-emitting contributor exits after reaching the hidden output budget",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_poll_limit_exits",
        counter_type: "u32",
        increments_when:
            "a non-emitting contributor exits after waiting for a staged hidden output to drain",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_poll_limit_missing_hidden",
        counter_type: "u32",
        increments_when:
            "a contributor poll-limit exit finds no still-published hidden row to diagnose",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_poll_limit_duplicate_active",
        counter_type: "u32",
        increments_when:
            "a contributor poll-limit exit waited on a hidden row matching an active or emitted visible heap TID",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_poll_limit_handoff_ready",
        counter_type: "u32",
        increments_when:
            "a contributor poll-limit exit waited on a hidden row that ordered before the visible owner row",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_poll_limit_ordered_after_visible",
        counter_type: "u32",
        increments_when:
            "a contributor poll-limit exit waited on a hidden row that ordered after the visible owner row",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_contributor_poll_limit_no_visible_owner",
        counter_type: "u32",
        increments_when:
            "a contributor poll-limit exit found no selected or admitted visible owner row to compare",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_blocked_foreign_selected_pending",
        counter_type: "u32",
        increments_when: "parallel scan staging is blocked by a foreign selected pending output",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_blocked_foreign_admitted_head",
        counter_type: "u32",
        increments_when: "parallel scan staging is blocked by a foreign admitted head",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_blocked_admission_window",
        counter_type: "u32",
        increments_when:
            "parallel scan staging is blocked because the current output loses the admitted window",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_local_only_emits",
        counter_type: "u32",
        increments_when:
            "a hidden local-only row is still locally emitted because shared retry did not drain a foreign output first",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_local_only_emits_foreign_selected_pending",
        counter_type: "u32",
        increments_when:
            "a hidden local-only row is still locally emitted while blocked by a foreign selected pending output",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_local_only_emits_foreign_admitted_head",
        counter_type: "u32",
        increments_when:
            "a hidden local-only row is still locally emitted while blocked by a foreign admitted head",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_deferred_local_emits",
        counter_type: "u32",
        increments_when:
            "a deferred blocked row is still locally emitted because no shared handoff-ready deferred row remains",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_deferred_local_emits_foreign_selected_pending",
        counter_type: "u32",
        increments_when:
            "a deferred blocked row is still locally emitted while blocked by a foreign selected pending output",
    },
    ExplainCounterDefinition {
        counter_name: "stats_parallel_deferred_local_emits_foreign_admitted_head",
        counter_type: "u32",
        increments_when:
            "a deferred blocked row is still locally emitted while blocked by a foreign admitted head",
    },
    ExplainCounterDefinition {
        counter_name: "stats_quantizer_cache_hit",
        counter_type: "bool",
        increments_when: "ProdQuantizer was reused from cache",
    },
];

pub(crate) fn explain_option_snapshot() -> ExplainOptionSnapshot {
    ExplainOptionSnapshot {
        option_name: "ecaz",
        pg18_custom_explain_option_ready: cfg!(feature = "pg18"),
        pg18_explain_per_node_hook_ready: cfg!(feature = "pg18"),
    }
}

pub(crate) fn explain_counter_definitions() -> &'static [ExplainCounterDefinition] {
    &EXPLAIN_COUNTER_DEFINITIONS
}

pub(crate) fn should_emit_explain_properties(context: ExplainHookContext<'_>) -> bool {
    context.explain_option_enabled
        && context.node_kind == ExplainNodeKind::IndexScan
        && context.access_method_name == "ec_hnsw"
}

pub(crate) fn explain_output_group() -> ExplainOutputGroup {
    ExplainOutputGroup {
        group_label: "Ecaz Stats",
        opened_with: "ExplainOpenGroup",
        closed_with: "ExplainCloseGroup",
    }
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

    pub(crate) fn record_parallel_handoff_foreign_selected_pending(&mut self) {
        self.stats_parallel_handoffs_foreign_selected_pending += 1;
    }

    pub(crate) fn record_parallel_handoff_foreign_admitted_head(&mut self) {
        self.stats_parallel_handoffs_foreign_admitted_head += 1;
    }

    pub(crate) fn record_parallel_contributor_hidden_publish(&mut self) {
        self.stats_parallel_contributor_hidden_publishes += 1;
    }

    pub(crate) fn record_parallel_contributor_publish_missing_hidden(&mut self) {
        self.stats_parallel_contributor_publish_missing_hidden += 1;
    }

    pub(crate) fn record_parallel_contributor_publish_duplicate_active(&mut self) {
        self.stats_parallel_contributor_publish_duplicate_active += 1;
    }

    pub(crate) fn record_parallel_contributor_publish_handoff_ready(&mut self) {
        self.stats_parallel_contributor_publish_handoff_ready += 1;
    }

    pub(crate) fn record_parallel_contributor_publish_ordered_after_visible(&mut self) {
        self.stats_parallel_contributor_publish_ordered_after_visible += 1;
    }

    pub(crate) fn record_parallel_contributor_publish_no_visible_owner(&mut self) {
        self.stats_parallel_contributor_publish_no_visible_owner += 1;
    }

    pub(crate) fn record_parallel_contributor_duplicate_retire(&mut self) {
        self.stats_parallel_contributor_duplicate_retires += 1;
    }

    pub(crate) fn record_parallel_contributor_output_limit_exit(&mut self) {
        self.stats_parallel_contributor_output_limit_exits += 1;
    }

    pub(crate) fn record_parallel_contributor_poll_limit_exit(&mut self) {
        self.stats_parallel_contributor_poll_limit_exits += 1;
    }

    pub(crate) fn record_parallel_contributor_poll_limit_missing_hidden(&mut self) {
        self.stats_parallel_contributor_poll_limit_missing_hidden += 1;
    }

    pub(crate) fn record_parallel_contributor_poll_limit_duplicate_active(&mut self) {
        self.stats_parallel_contributor_poll_limit_duplicate_active += 1;
    }

    pub(crate) fn record_parallel_contributor_poll_limit_handoff_ready(&mut self) {
        self.stats_parallel_contributor_poll_limit_handoff_ready += 1;
    }

    pub(crate) fn record_parallel_contributor_poll_limit_ordered_after_visible(&mut self) {
        self.stats_parallel_contributor_poll_limit_ordered_after_visible += 1;
    }

    pub(crate) fn record_parallel_contributor_poll_limit_no_visible_owner(&mut self) {
        self.stats_parallel_contributor_poll_limit_no_visible_owner += 1;
    }

    pub(crate) fn record_parallel_blocked_foreign_selected_pending(&mut self) {
        self.stats_parallel_blocked_foreign_selected_pending += 1;
    }

    pub(crate) fn record_parallel_blocked_foreign_admitted_head(&mut self) {
        self.stats_parallel_blocked_foreign_admitted_head += 1;
    }

    pub(crate) fn record_parallel_blocked_admission_window(&mut self) {
        self.stats_parallel_blocked_admission_window += 1;
    }

    pub(crate) fn record_parallel_local_only_emit(&mut self) {
        self.stats_parallel_local_only_emits += 1;
    }

    pub(crate) fn record_parallel_local_only_emit_foreign_selected_pending(&mut self) {
        self.stats_parallel_local_only_emits_foreign_selected_pending += 1;
    }

    pub(crate) fn record_parallel_local_only_emit_foreign_admitted_head(&mut self) {
        self.stats_parallel_local_only_emits_foreign_admitted_head += 1;
    }

    pub(crate) fn record_parallel_deferred_local_emit(&mut self) {
        self.stats_parallel_deferred_local_emits += 1;
    }

    pub(crate) fn record_parallel_deferred_local_emit_foreign_selected_pending(&mut self) {
        self.stats_parallel_deferred_local_emits_foreign_selected_pending += 1;
    }

    pub(crate) fn record_parallel_deferred_local_emit_foreign_admitted_head(&mut self) {
        self.stats_parallel_deferred_local_emits_foreign_admitted_head += 1;
    }

    pub(crate) fn record_quantizer_cache_hit(&mut self) {
        self.stats_quantizer_cache_hit = true;
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn explain_properties(self) -> [ExplainProperty; 32] {
        [
            ExplainProperty {
                property_name: "Bootstrap Expansions",
                value: ExplainPropertyValue::Integer(self.stats_bootstrap_expansions),
            },
            ExplainProperty {
                property_name: "Bootstrap Pages Read",
                value: ExplainPropertyValue::Integer(self.stats_bootstrap_pages_read),
            },
            ExplainProperty {
                property_name: "Linear Pages Read",
                value: ExplainPropertyValue::Integer(self.stats_linear_pages_read),
            },
            ExplainProperty {
                property_name: "Elements Scored",
                value: ExplainPropertyValue::Integer(self.stats_elements_scored),
            },
            ExplainProperty {
                property_name: "Elements Skipped",
                value: ExplainPropertyValue::Integer(self.stats_elements_skipped),
            },
            ExplainProperty {
                property_name: "Heap TIDs Returned",
                value: ExplainPropertyValue::Integer(self.stats_heap_tids_returned),
            },
            ExplainProperty {
                property_name: "Parallel Handoffs: Foreign Selected",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_handoffs_foreign_selected_pending,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Handoffs: Foreign Head",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_handoffs_foreign_admitted_head,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Hidden Publishes",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_hidden_publishes,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Publish: Missing Hidden",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_publish_missing_hidden,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Publish: Duplicate Active",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_publish_duplicate_active,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Publish: Handoff Ready",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_publish_handoff_ready,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Publish: Ordered After Visible",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_publish_ordered_after_visible,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Publish: No Visible Owner",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_publish_no_visible_owner,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Duplicate Retires",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_duplicate_retires,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Output Limit Exits",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_output_limit_exits,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Poll Limit Exits",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_poll_limit_exits,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Poll Limit: Missing Hidden",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_poll_limit_missing_hidden,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Poll Limit: Duplicate Active",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_poll_limit_duplicate_active,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Poll Limit: Handoff Ready",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_poll_limit_handoff_ready,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Poll Limit: Ordered After Visible",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_poll_limit_ordered_after_visible,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Contributor Poll Limit: No Visible Owner",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_contributor_poll_limit_no_visible_owner,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Blocked: Foreign Selected",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_blocked_foreign_selected_pending,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Blocked: Foreign Head",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_blocked_foreign_admitted_head,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Blocked: Admission Window",
                value: ExplainPropertyValue::Integer(self.stats_parallel_blocked_admission_window),
            },
            ExplainProperty {
                property_name: "Parallel Local-only Emits",
                value: ExplainPropertyValue::Integer(self.stats_parallel_local_only_emits),
            },
            ExplainProperty {
                property_name: "Parallel Local-only Emits: Foreign Selected",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_local_only_emits_foreign_selected_pending,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Local-only Emits: Foreign Head",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_local_only_emits_foreign_admitted_head,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Deferred Local Emits",
                value: ExplainPropertyValue::Integer(self.stats_parallel_deferred_local_emits),
            },
            ExplainProperty {
                property_name: "Parallel Deferred Local Emits: Foreign Selected",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_deferred_local_emits_foreign_selected_pending,
                ),
            },
            ExplainProperty {
                property_name: "Parallel Deferred Local Emits: Foreign Head",
                value: ExplainPropertyValue::Integer(
                    self.stats_parallel_deferred_local_emits_foreign_admitted_head,
                ),
            },
            ExplainProperty {
                property_name: "Quantizer Cache Hit",
                value: ExplainPropertyValue::Bool(self.stats_quantizer_cache_hit),
            },
        ]
    }
}

#[cfg(feature = "pg18")]
static PREVIOUS_EXPLAIN_PER_NODE_HOOK: OnceLock<pg_sys::explain_per_node_hook_type> =
    OnceLock::new();
#[cfg(feature = "pg18")]
static ECAZ_EXPLAIN_REGISTERED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "pg18")]
fn previous_explain_per_node_hook() -> pg_sys::explain_per_node_hook_type {
    PREVIOUS_EXPLAIN_PER_NODE_HOOK
        .get()
        .copied()
        .unwrap_or(None)
}

#[cfg(feature = "pg18")]
fn explain_extension_id() -> i32 {
    unsafe { pg_sys::GetExplainExtensionId(c"ecaz".as_ptr()) }
}

#[cfg(feature = "pg18")]
unsafe fn explain_option_enabled(es: *mut pg_sys::ExplainState) -> bool {
    let state = unsafe { pg_sys::GetExplainExtensionState(es, explain_extension_id()) };
    if state.is_null() {
        return false;
    }

    unsafe { *(state.cast::<bool>()) }
}

#[cfg(feature = "pg18")]
unsafe fn explain_node_kind(planstate: *mut pg_sys::PlanState) -> ExplainNodeKind {
    match unsafe { (*planstate).type_ } {
        pg_sys::NodeTag::T_IndexScanState => ExplainNodeKind::IndexScan,
        _ => ExplainNodeKind::Other,
    }
}

#[cfg(feature = "pg18")]
unsafe fn explain_access_method_name(index_state: *mut pg_sys::IndexScanState) -> Option<String> {
    let index_relation = unsafe { (*index_state).iss_RelationDesc };
    if index_relation.is_null() {
        return None;
    }

    let am_oid = unsafe { (*(*index_relation).rd_rel).relam };
    let am_name_ptr = unsafe { pg_sys::get_am_name(am_oid) };
    if am_name_ptr.is_null() {
        return None;
    }

    let name = unsafe { CStr::from_ptr(am_name_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { pg_sys::pfree(am_name_ptr.cast()) };
    Some(name)
}

#[cfg(feature = "pg18")]
unsafe fn emit_explain_properties(es: *mut pg_sys::ExplainState, counters: TqExplainCounters) {
    let group = explain_output_group();
    let group_label = CString::new(group.group_label).expect("group label should not contain NUL");
    unsafe {
        pg_sys::ExplainOpenGroup(group_label.as_ptr(), group_label.as_ptr(), true, es);
    }

    for property in counters.explain_properties() {
        let property_name =
            CString::new(property.property_name).expect("property name should not contain NUL");
        unsafe {
            match property.value {
                ExplainPropertyValue::Integer(value) => pg_sys::ExplainPropertyInteger(
                    property_name.as_ptr(),
                    ptr::null(),
                    i64::from(value),
                    es,
                ),
                ExplainPropertyValue::Bool(value) => {
                    pg_sys::ExplainPropertyBool(property_name.as_ptr(), value, es)
                }
            }
        }
    }

    unsafe {
        pg_sys::ExplainCloseGroup(group_label.as_ptr(), group_label.as_ptr(), true, es);
    }
}

#[cfg(feature = "pg18")]
unsafe extern "C-unwind" fn ecaz_explain_option_handler(
    es: *mut pg_sys::ExplainState,
    opt: *mut pg_sys::DefElem,
    _pstate: *mut pg_sys::ParseState,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let enabled = pg_sys::defGetBoolean(opt);
            let state = pg_sys::palloc0(std::mem::size_of::<bool>()).cast::<bool>();
            if state.is_null() {
                pgrx::error!("ecaz failed to allocate EXPLAIN option state");
            }
            *state = enabled;
            pg_sys::SetExplainExtensionState(es, explain_extension_id(), state.cast::<c_void>());
        })
    }
}

#[cfg(feature = "pg18")]
unsafe extern "C-unwind" fn ecaz_explain_per_node_hook(
    planstate: *mut pg_sys::PlanState,
    ancestors: *mut pg_sys::List,
    relationship: *const std::ffi::c_char,
    plan_name: *const std::ffi::c_char,
    es: *mut pg_sys::ExplainState,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if !planstate.is_null()
                && !es.is_null()
                && (*planstate).type_ == pg_sys::NodeTag::T_IndexScanState
            {
                let explain_option_enabled = explain_option_enabled(es);
                if !explain_option_enabled {
                    if let Some(previous_hook) = previous_explain_per_node_hook() {
                        previous_hook(planstate, ancestors, relationship, plan_name, es);
                    }
                    return;
                }

                let index_state = planstate.cast::<pg_sys::IndexScanState>();
                let access_method_name = explain_access_method_name(index_state)
                    .unwrap_or_else(|| "<unknown>".to_owned());
                let context = ExplainHookContext {
                    explain_option_enabled,
                    node_kind: explain_node_kind(planstate),
                    access_method_name: access_method_name.as_str(),
                };
                if should_emit_explain_properties(context) {
                    let counters =
                        crate::am::ec_hnsw::explain_counters_from_index_scan_state(index_state);
                    emit_explain_properties(es, counters);
                }
            }

            if let Some(previous_hook) = previous_explain_per_node_hook() {
                previous_hook(planstate, ancestors, relationship, plan_name, es);
            }
        })
    }
}

#[cfg(feature = "pg18")]
pub(crate) unsafe fn register_pg18_explain_hooks() {
    unsafe {
        if ECAZ_EXPLAIN_REGISTERED.load(Ordering::Acquire) {
            return;
        }

        pg_sys::RegisterExtensionExplainOption(c"ecaz".as_ptr(), Some(ecaz_explain_option_handler));
        let _ = PREVIOUS_EXPLAIN_PER_NODE_HOOK.set(pg_sys::explain_per_node_hook);
        pg_sys::explain_per_node_hook = Some(ecaz_explain_per_node_hook);
        ECAZ_EXPLAIN_REGISTERED.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        explain_counter_definitions, explain_option_snapshot, explain_output_group,
        should_emit_explain_properties, ExplainCounterDefinition, ExplainHookContext,
        ExplainNodeKind, ExplainOptionSnapshot, ExplainOutputGroup, ExplainProperty,
        ExplainPropertyValue, TqExplainCounters,
    };

    #[test]
    fn explain_option_snapshot_matches_build_target() {
        assert_eq!(
            explain_option_snapshot(),
            ExplainOptionSnapshot {
                option_name: "ecaz",
                pg18_custom_explain_option_ready: cfg!(feature = "pg18"),
                pg18_explain_per_node_hook_ready: cfg!(feature = "pg18"),
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
                    counter_name: "stats_parallel_handoffs_foreign_selected_pending",
                    counter_type: "u32",
                    increments_when:
                        "a worker drains a foreign selected pending output through the shared handoff seam",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_handoffs_foreign_admitted_head",
                    counter_type: "u32",
                    increments_when:
                        "a worker drains a foreign admitted head through the shared handoff seam",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_hidden_publishes",
                    counter_type: "u32",
                    increments_when:
                        "a non-emitting contributor publishes hidden output behind the elected emitter",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_publish_missing_hidden",
                    counter_type: "u32",
                    increments_when:
                        "a hidden contributor publish finds no still-published hidden row to diagnose",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_publish_duplicate_active",
                    counter_type: "u32",
                    increments_when:
                        "a hidden contributor publish matches an active or emitted visible heap TID",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_publish_handoff_ready",
                    counter_type: "u32",
                    increments_when:
                        "a hidden contributor publish orders before the visible owner row",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_publish_ordered_after_visible",
                    counter_type: "u32",
                    increments_when:
                        "a hidden contributor publish orders after the visible owner row",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_publish_no_visible_owner",
                    counter_type: "u32",
                    increments_when:
                        "a hidden contributor publish finds no selected or admitted visible owner row to compare",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_duplicate_retires",
                    counter_type: "u32",
                    increments_when:
                        "a non-emitting contributor retires a hidden row whose next heap TID was already emitted",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_output_limit_exits",
                    counter_type: "u32",
                    increments_when:
                        "a non-emitting contributor exits after reaching the hidden output budget",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_poll_limit_exits",
                    counter_type: "u32",
                    increments_when:
                        "a non-emitting contributor exits after waiting for a staged hidden output to drain",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_poll_limit_missing_hidden",
                    counter_type: "u32",
                    increments_when:
                        "a contributor poll-limit exit finds no still-published hidden row to diagnose",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_poll_limit_duplicate_active",
                    counter_type: "u32",
                    increments_when:
                        "a contributor poll-limit exit waited on a hidden row matching an active or emitted visible heap TID",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_poll_limit_handoff_ready",
                    counter_type: "u32",
                    increments_when:
                        "a contributor poll-limit exit waited on a hidden row that ordered before the visible owner row",
                },
                ExplainCounterDefinition {
                    counter_name:
                        "stats_parallel_contributor_poll_limit_ordered_after_visible",
                    counter_type: "u32",
                    increments_when:
                        "a contributor poll-limit exit waited on a hidden row that ordered after the visible owner row",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_contributor_poll_limit_no_visible_owner",
                    counter_type: "u32",
                    increments_when:
                        "a contributor poll-limit exit found no selected or admitted visible owner row to compare",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_blocked_foreign_selected_pending",
                    counter_type: "u32",
                    increments_when:
                        "parallel scan staging is blocked by a foreign selected pending output",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_blocked_foreign_admitted_head",
                    counter_type: "u32",
                    increments_when:
                        "parallel scan staging is blocked by a foreign admitted head",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_blocked_admission_window",
                    counter_type: "u32",
                    increments_when:
                        "parallel scan staging is blocked because the current output loses the admitted window",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_local_only_emits",
                    counter_type: "u32",
                    increments_when:
                        "a hidden local-only row is still locally emitted because shared retry did not drain a foreign output first",
                },
                ExplainCounterDefinition {
                    counter_name:
                        "stats_parallel_local_only_emits_foreign_selected_pending",
                    counter_type: "u32",
                    increments_when:
                        "a hidden local-only row is still locally emitted while blocked by a foreign selected pending output",
                },
                ExplainCounterDefinition {
                    counter_name:
                        "stats_parallel_local_only_emits_foreign_admitted_head",
                    counter_type: "u32",
                    increments_when:
                        "a hidden local-only row is still locally emitted while blocked by a foreign admitted head",
                },
                ExplainCounterDefinition {
                    counter_name: "stats_parallel_deferred_local_emits",
                    counter_type: "u32",
                    increments_when:
                        "a deferred blocked row is still locally emitted because no shared handoff-ready deferred row remains",
                },
                ExplainCounterDefinition {
                    counter_name:
                        "stats_parallel_deferred_local_emits_foreign_selected_pending",
                    counter_type: "u32",
                    increments_when:
                        "a deferred blocked row is still locally emitted while blocked by a foreign selected pending output",
                },
                ExplainCounterDefinition {
                    counter_name:
                        "stats_parallel_deferred_local_emits_foreign_admitted_head",
                    counter_type: "u32",
                    increments_when:
                        "a deferred blocked row is still locally emitted while blocked by a foreign admitted head",
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
    fn explain_output_group_matches_fr024_hook_contract() {
        assert_eq!(
            explain_output_group(),
            ExplainOutputGroup {
                group_label: "Ecaz Stats",
                opened_with: "ExplainOpenGroup",
                closed_with: "ExplainCloseGroup",
            }
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
        counters.record_parallel_handoff_foreign_selected_pending();
        counters.record_parallel_handoff_foreign_admitted_head();
        counters.record_parallel_contributor_hidden_publish();
        counters.record_parallel_contributor_publish_missing_hidden();
        counters.record_parallel_contributor_publish_duplicate_active();
        counters.record_parallel_contributor_publish_handoff_ready();
        counters.record_parallel_contributor_publish_ordered_after_visible();
        counters.record_parallel_contributor_publish_no_visible_owner();
        counters.record_parallel_contributor_duplicate_retire();
        counters.record_parallel_contributor_output_limit_exit();
        counters.record_parallel_contributor_poll_limit_exit();
        counters.record_parallel_contributor_poll_limit_missing_hidden();
        counters.record_parallel_contributor_poll_limit_duplicate_active();
        counters.record_parallel_contributor_poll_limit_handoff_ready();
        counters.record_parallel_contributor_poll_limit_ordered_after_visible();
        counters.record_parallel_contributor_poll_limit_no_visible_owner();
        counters.record_parallel_blocked_foreign_selected_pending();
        counters.record_parallel_blocked_foreign_admitted_head();
        counters.record_parallel_blocked_admission_window();
        counters.record_parallel_local_only_emit();
        counters.record_parallel_local_only_emit_foreign_selected_pending();
        counters.record_parallel_local_only_emit_foreign_admitted_head();
        counters.record_parallel_deferred_local_emit();
        counters.record_parallel_deferred_local_emit_foreign_selected_pending();
        counters.record_parallel_deferred_local_emit_foreign_admitted_head();
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
                stats_parallel_handoffs_foreign_selected_pending: 1,
                stats_parallel_handoffs_foreign_admitted_head: 1,
                stats_parallel_contributor_hidden_publishes: 1,
                stats_parallel_contributor_publish_missing_hidden: 1,
                stats_parallel_contributor_publish_duplicate_active: 1,
                stats_parallel_contributor_publish_handoff_ready: 1,
                stats_parallel_contributor_publish_ordered_after_visible: 1,
                stats_parallel_contributor_publish_no_visible_owner: 1,
                stats_parallel_contributor_duplicate_retires: 1,
                stats_parallel_contributor_output_limit_exits: 1,
                stats_parallel_contributor_poll_limit_exits: 1,
                stats_parallel_contributor_poll_limit_missing_hidden: 1,
                stats_parallel_contributor_poll_limit_duplicate_active: 1,
                stats_parallel_contributor_poll_limit_handoff_ready: 1,
                stats_parallel_contributor_poll_limit_ordered_after_visible: 1,
                stats_parallel_contributor_poll_limit_no_visible_owner: 1,
                stats_parallel_blocked_foreign_selected_pending: 1,
                stats_parallel_blocked_foreign_admitted_head: 1,
                stats_parallel_blocked_admission_window: 1,
                stats_parallel_local_only_emits: 1,
                stats_parallel_local_only_emits_foreign_selected_pending: 1,
                stats_parallel_local_only_emits_foreign_admitted_head: 1,
                stats_parallel_deferred_local_emits: 1,
                stats_parallel_deferred_local_emits_foreign_selected_pending: 1,
                stats_parallel_deferred_local_emits_foreign_admitted_head: 1,
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
            stats_parallel_handoffs_foreign_selected_pending: 17,
            stats_parallel_handoffs_foreign_admitted_head: 19,
            stats_parallel_contributor_hidden_publishes: 23,
            stats_parallel_contributor_publish_missing_hidden: 29,
            stats_parallel_contributor_publish_duplicate_active: 31,
            stats_parallel_contributor_publish_handoff_ready: 37,
            stats_parallel_contributor_publish_ordered_after_visible: 41,
            stats_parallel_contributor_publish_no_visible_owner: 43,
            stats_parallel_contributor_duplicate_retires: 47,
            stats_parallel_contributor_output_limit_exits: 53,
            stats_parallel_contributor_poll_limit_exits: 59,
            stats_parallel_contributor_poll_limit_missing_hidden: 61,
            stats_parallel_contributor_poll_limit_duplicate_active: 67,
            stats_parallel_contributor_poll_limit_handoff_ready: 71,
            stats_parallel_contributor_poll_limit_ordered_after_visible: 73,
            stats_parallel_contributor_poll_limit_no_visible_owner: 79,
            stats_parallel_blocked_foreign_selected_pending: 83,
            stats_parallel_blocked_foreign_admitted_head: 89,
            stats_parallel_blocked_admission_window: 97,
            stats_parallel_local_only_emits: 101,
            stats_parallel_local_only_emits_foreign_selected_pending: 103,
            stats_parallel_local_only_emits_foreign_admitted_head: 107,
            stats_parallel_deferred_local_emits: 109,
            stats_parallel_deferred_local_emits_foreign_selected_pending: 113,
            stats_parallel_deferred_local_emits_foreign_admitted_head: 127,
            stats_quantizer_cache_hit: true,
        };

        counters.reset();

        assert_eq!(counters, TqExplainCounters::default());
    }

    #[test]
    fn explain_properties_render_the_current_counter_values() {
        let counters = TqExplainCounters {
            stats_bootstrap_expansions: 2,
            stats_bootstrap_pages_read: 3,
            stats_linear_pages_read: 5,
            stats_elements_scored: 7,
            stats_elements_skipped: 11,
            stats_heap_tids_returned: 13,
            stats_parallel_handoffs_foreign_selected_pending: 17,
            stats_parallel_handoffs_foreign_admitted_head: 19,
            stats_parallel_contributor_hidden_publishes: 23,
            stats_parallel_contributor_publish_missing_hidden: 29,
            stats_parallel_contributor_publish_duplicate_active: 31,
            stats_parallel_contributor_publish_handoff_ready: 37,
            stats_parallel_contributor_publish_ordered_after_visible: 41,
            stats_parallel_contributor_publish_no_visible_owner: 43,
            stats_parallel_contributor_duplicate_retires: 47,
            stats_parallel_contributor_output_limit_exits: 53,
            stats_parallel_contributor_poll_limit_exits: 59,
            stats_parallel_contributor_poll_limit_missing_hidden: 61,
            stats_parallel_contributor_poll_limit_duplicate_active: 67,
            stats_parallel_contributor_poll_limit_handoff_ready: 71,
            stats_parallel_contributor_poll_limit_ordered_after_visible: 73,
            stats_parallel_contributor_poll_limit_no_visible_owner: 79,
            stats_parallel_blocked_foreign_selected_pending: 83,
            stats_parallel_blocked_foreign_admitted_head: 89,
            stats_parallel_blocked_admission_window: 97,
            stats_parallel_local_only_emits: 101,
            stats_parallel_local_only_emits_foreign_selected_pending: 103,
            stats_parallel_local_only_emits_foreign_admitted_head: 107,
            stats_parallel_deferred_local_emits: 109,
            stats_parallel_deferred_local_emits_foreign_selected_pending: 113,
            stats_parallel_deferred_local_emits_foreign_admitted_head: 127,
            stats_quantizer_cache_hit: true,
        };

        assert_eq!(
            counters.explain_properties(),
            [
                ExplainProperty {
                    property_name: "Bootstrap Expansions",
                    value: ExplainPropertyValue::Integer(2),
                },
                ExplainProperty {
                    property_name: "Bootstrap Pages Read",
                    value: ExplainPropertyValue::Integer(3),
                },
                ExplainProperty {
                    property_name: "Linear Pages Read",
                    value: ExplainPropertyValue::Integer(5),
                },
                ExplainProperty {
                    property_name: "Elements Scored",
                    value: ExplainPropertyValue::Integer(7),
                },
                ExplainProperty {
                    property_name: "Elements Skipped",
                    value: ExplainPropertyValue::Integer(11),
                },
                ExplainProperty {
                    property_name: "Heap TIDs Returned",
                    value: ExplainPropertyValue::Integer(13),
                },
                ExplainProperty {
                    property_name: "Parallel Handoffs: Foreign Selected",
                    value: ExplainPropertyValue::Integer(17),
                },
                ExplainProperty {
                    property_name: "Parallel Handoffs: Foreign Head",
                    value: ExplainPropertyValue::Integer(19),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Hidden Publishes",
                    value: ExplainPropertyValue::Integer(23),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Publish: Missing Hidden",
                    value: ExplainPropertyValue::Integer(29),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Publish: Duplicate Active",
                    value: ExplainPropertyValue::Integer(31),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Publish: Handoff Ready",
                    value: ExplainPropertyValue::Integer(37),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Publish: Ordered After Visible",
                    value: ExplainPropertyValue::Integer(41),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Publish: No Visible Owner",
                    value: ExplainPropertyValue::Integer(43),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Duplicate Retires",
                    value: ExplainPropertyValue::Integer(47),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Output Limit Exits",
                    value: ExplainPropertyValue::Integer(53),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Poll Limit Exits",
                    value: ExplainPropertyValue::Integer(59),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Poll Limit: Missing Hidden",
                    value: ExplainPropertyValue::Integer(61),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Poll Limit: Duplicate Active",
                    value: ExplainPropertyValue::Integer(67),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Poll Limit: Handoff Ready",
                    value: ExplainPropertyValue::Integer(71),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Poll Limit: Ordered After Visible",
                    value: ExplainPropertyValue::Integer(73),
                },
                ExplainProperty {
                    property_name: "Parallel Contributor Poll Limit: No Visible Owner",
                    value: ExplainPropertyValue::Integer(79),
                },
                ExplainProperty {
                    property_name: "Parallel Blocked: Foreign Selected",
                    value: ExplainPropertyValue::Integer(83),
                },
                ExplainProperty {
                    property_name: "Parallel Blocked: Foreign Head",
                    value: ExplainPropertyValue::Integer(89),
                },
                ExplainProperty {
                    property_name: "Parallel Blocked: Admission Window",
                    value: ExplainPropertyValue::Integer(97),
                },
                ExplainProperty {
                    property_name: "Parallel Local-only Emits",
                    value: ExplainPropertyValue::Integer(101),
                },
                ExplainProperty {
                    property_name: "Parallel Local-only Emits: Foreign Selected",
                    value: ExplainPropertyValue::Integer(103),
                },
                ExplainProperty {
                    property_name: "Parallel Local-only Emits: Foreign Head",
                    value: ExplainPropertyValue::Integer(107),
                },
                ExplainProperty {
                    property_name: "Parallel Deferred Local Emits",
                    value: ExplainPropertyValue::Integer(109),
                },
                ExplainProperty {
                    property_name: "Parallel Deferred Local Emits: Foreign Selected",
                    value: ExplainPropertyValue::Integer(113),
                },
                ExplainProperty {
                    property_name: "Parallel Deferred Local Emits: Foreign Head",
                    value: ExplainPropertyValue::Integer(127),
                },
                ExplainProperty {
                    property_name: "Quantizer Cache Hit",
                    value: ExplainPropertyValue::Bool(true),
                },
            ]
        );
    }

    #[test]
    fn explain_property_emission_requires_option_index_scan_and_ec_hnsw_access_method() {
        assert!(should_emit_explain_properties(ExplainHookContext {
            explain_option_enabled: true,
            node_kind: ExplainNodeKind::IndexScan,
            access_method_name: "ec_hnsw",
        }));
        assert!(!should_emit_explain_properties(ExplainHookContext {
            explain_option_enabled: false,
            node_kind: ExplainNodeKind::IndexScan,
            access_method_name: "ec_hnsw",
        }));
        assert!(!should_emit_explain_properties(ExplainHookContext {
            explain_option_enabled: true,
            node_kind: ExplainNodeKind::Other,
            access_method_name: "ec_hnsw",
        }));
        assert!(!should_emit_explain_properties(ExplainHookContext {
            explain_option_enabled: true,
            node_kind: ExplainNodeKind::IndexScan,
            access_method_name: "btree",
        }));
    }
}
