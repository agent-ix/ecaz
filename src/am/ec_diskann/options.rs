use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting, PostgresGucEnum};

use super::{
    ECDISKANN_DEFAULT_ALPHA, ECDISKANN_DEFAULT_BUILD_LIST_SIZE, ECDISKANN_DEFAULT_GRAPH_DEGREE,
    ECDISKANN_DEFAULT_RERANK_BUDGET, ECDISKANN_DEFAULT_SCAN_LIST_SIZE, ECDISKANN_DEFAULT_TOP_K,
    ECDISKANN_MAX_ALPHA, ECDISKANN_MAX_BUILD_LIST_SIZE, ECDISKANN_MAX_GRAPH_DEGREE,
    ECDISKANN_MAX_RERANK_BUDGET, ECDISKANN_MAX_SCAN_LIST_SIZE, ECDISKANN_MAX_TOP_K,
    ECDISKANN_MIN_ALPHA, ECDISKANN_MIN_BUILD_LIST_SIZE, ECDISKANN_MIN_GRAPH_DEGREE,
    ECDISKANN_MIN_RERANK_BUDGET, ECDISKANN_MIN_SCAN_LIST_SIZE, ECDISKANN_MIN_TOP_K,
};

const ECDISKANN_SESSION_LIST_SIZE_UNSET: i32 = -1;

static ECDISKANN_LIST_SIZE_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(ECDISKANN_SESSION_LIST_SIZE_UNSET);
static ECDISKANN_PREFILTER_KIND_GUC: GucSetting<PrefilterKind> =
    GucSetting::<PrefilterKind>::new(PrefilterKind::Auto);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PostgresGucEnum)]
pub(super) enum PrefilterKind {
    #[name = c"auto"]
    Auto,
    #[name = c"binary_sidecar"]
    BinarySidecar,
    #[name = c"grouped_pq"]
    GroupedPq,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TqDiskannReloptions {
    vl_len_: i32,
    graph_degree: i32,
    build_list_size: i32,
    list_size: i32,
    rerank_budget: i32,
    top_k: i32,
    // Postgres real reloptions are stored as C doubles; alpha is downcast to
    // f32 when constructing `TqDiskannOptions` per ADR-034 / task 17 decision
    // (pgvectorscale-compatible f32 surface, f64 storage inside the relopt
    // blob is a Postgres implementation detail).
    alpha: f64,
    storage_format_offset: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StorageFormat {
    PqFastScan,
}

impl StorageFormat {
    pub(super) const DEFAULT: Self = Self::PqFastScan;

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::PqFastScan => "pq_fastscan",
        }
    }

    fn parse_reloption(raw: &str) -> Result<Self, String> {
        match raw {
            "pq_fastscan" => Ok(Self::PqFastScan),
            other => Err(format!(
                "invalid ec_diskann storage_format reloption: expected 'pq_fastscan', got {:?}",
                other
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct TqDiskannOptions {
    pub(super) graph_degree: i32,
    pub(super) build_list_size: i32,
    pub(super) list_size: i32,
    pub(super) rerank_budget: i32,
    pub(super) top_k: i32,
    pub(super) alpha: f32,
    pub(super) storage_format: StorageFormat,
}

impl TqDiskannOptions {
    pub(super) const DEFAULT: Self = Self {
        graph_degree: ECDISKANN_DEFAULT_GRAPH_DEGREE,
        build_list_size: ECDISKANN_DEFAULT_BUILD_LIST_SIZE,
        list_size: ECDISKANN_DEFAULT_SCAN_LIST_SIZE,
        rerank_budget: ECDISKANN_DEFAULT_RERANK_BUDGET,
        top_k: ECDISKANN_DEFAULT_TOP_K,
        alpha: ECDISKANN_DEFAULT_ALPHA,
        storage_format: StorageFormat::DEFAULT,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ListSizeSource {
    Relation,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ScanTuning {
    pub(super) relation_list_size: i32,
    pub(super) session_list_size: Option<i32>,
    pub(super) effective_list_size: i32,
    pub(super) source: ListSizeSource,
}

pub(super) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ec_diskann.list_size",
        c"Session override for ec_diskann search breadth.",
        c"Overrides ec_diskann index list_size reloptions when set to 1-10000; -1 uses the relation value.",
        &ECDISKANN_LIST_SIZE_GUC,
        ECDISKANN_SESSION_LIST_SIZE_UNSET,
        ECDISKANN_MAX_SCAN_LIST_SIZE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_enum_guc(
        c"ec_diskann.prefilter_kind",
        c"Session override for ec_diskann traversal prefilter.",
        c"Diagnostic override used for Task 29a A/B measurement. Values: auto uses binary_sidecar when present and falls back to grouped_pq; binary_sidecar requires persisted binary sidecars; grouped_pq forces the legacy grouped-PQ prefilter.",
        &ECDISKANN_PREFILTER_KIND_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_session_list_size() -> i32 {
    ECDISKANN_LIST_SIZE_GUC.get()
}

pub(super) fn current_prefilter_kind() -> PrefilterKind {
    ECDISKANN_PREFILTER_KIND_GUC.get()
}

pub(super) fn resolve_scan_tuning(options: &TqDiskannOptions) -> ScanTuning {
    resolve_scan_tuning_values(options.list_size, current_session_list_size())
}

fn resolve_scan_tuning_values(relation_list_size: i32, session_list_size: i32) -> ScanTuning {
    if session_list_size == ECDISKANN_SESSION_LIST_SIZE_UNSET {
        ScanTuning {
            relation_list_size,
            session_list_size: None,
            effective_list_size: relation_list_size,
            source: ListSizeSource::Relation,
        }
    } else {
        ScanTuning {
            relation_list_size,
            session_list_size: Some(session_list_size),
            effective_list_size: session_list_size,
            source: ListSizeSource::Session,
        }
    }
}

pub(super) unsafe extern "C-unwind" fn ec_diskann_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut relopts = pg_sys::local_relopts::default();

            pg_sys::init_local_reloptions(&mut relopts, size_of::<TqDiskannReloptions>());
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"graph_degree\0".as_ptr().cast(),
                b"Fixed neighbor count per Vamana graph node (R).\0"
                    .as_ptr()
                    .cast(),
                ECDISKANN_DEFAULT_GRAPH_DEGREE,
                ECDISKANN_MIN_GRAPH_DEGREE,
                ECDISKANN_MAX_GRAPH_DEGREE,
                offset_of!(TqDiskannReloptions, graph_degree) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"build_list_size\0".as_ptr().cast(),
                b"Candidate list width used during Vamana graph construction (L).\0"
                    .as_ptr()
                    .cast(),
                ECDISKANN_DEFAULT_BUILD_LIST_SIZE,
                ECDISKANN_MIN_BUILD_LIST_SIZE,
                ECDISKANN_MAX_BUILD_LIST_SIZE,
                offset_of!(TqDiskannReloptions, build_list_size) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"list_size\0".as_ptr().cast(),
                b"Greedy frontier width used during ec_diskann scan (L_search).\0"
                    .as_ptr()
                    .cast(),
                ECDISKANN_DEFAULT_SCAN_LIST_SIZE,
                ECDISKANN_MIN_SCAN_LIST_SIZE,
                ECDISKANN_MAX_SCAN_LIST_SIZE,
                offset_of!(TqDiskannReloptions, list_size) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"rerank_budget\0".as_ptr().cast(),
                b"How many ec_diskann scan candidates to exact-rerank from the heap.\0"
                    .as_ptr()
                    .cast(),
                ECDISKANN_DEFAULT_RERANK_BUDGET,
                ECDISKANN_MIN_RERANK_BUDGET,
                ECDISKANN_MAX_RERANK_BUDGET,
                offset_of!(TqDiskannReloptions, rerank_budget) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"top_k\0".as_ptr().cast(),
                b"How many exact-reranked ec_diskann results to return before executor truncation.\0"
                    .as_ptr()
                    .cast(),
                ECDISKANN_DEFAULT_TOP_K,
                ECDISKANN_MIN_TOP_K,
                ECDISKANN_MAX_TOP_K,
                offset_of!(TqDiskannReloptions, top_k) as i32,
            );
            pg_sys::add_local_real_reloption(
                &mut relopts,
                b"alpha\0".as_ptr().cast(),
                b"Vamana alpha-pruning slack; pgvectorscale-compatible f32.\0"
                    .as_ptr()
                    .cast(),
                ECDISKANN_DEFAULT_ALPHA as f64,
                ECDISKANN_MIN_ALPHA as f64,
                ECDISKANN_MAX_ALPHA as f64,
                offset_of!(TqDiskannReloptions, alpha) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                b"storage_format\0".as_ptr().cast(),
                b"Index storage format (only 'pq_fastscan' is accepted).\0"
                    .as_ptr()
                    .cast(),
                ptr::null(),
                None,
                None,
                offset_of!(TqDiskannReloptions, storage_format_offset) as i32,
            );
            pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
        })
    }
}

unsafe fn read_string_reloption(
    rd_options: *mut pg_sys::varlena,
    offset: i32,
    name: &str,
) -> Option<String> {
    if offset == 0 {
        return None;
    }

    let value_ptr = unsafe {
        rd_options
            .cast::<u8>()
            .add(offset as usize)
            .cast::<std::ffi::c_char>()
    };
    let value = unsafe { std::ffi::CStr::from_ptr(value_ptr) }
        .to_str()
        .unwrap_or_else(|e| pgrx::error!("invalid ec_diskann {name} reloption: {e}"));
    if value.is_empty() {
        pgrx::error!("invalid ec_diskann {name} reloption: value must not be empty");
    }
    Some(value.to_owned())
}

#[allow(dead_code)]
pub(super) unsafe fn relation_options(index_relation: pg_sys::Relation) -> TqDiskannOptions {
    let rd_options = unsafe { (*index_relation).rd_options };
    if rd_options.is_null() {
        return TqDiskannOptions::DEFAULT;
    }

    let reloptions = unsafe { &*rd_options.cast::<TqDiskannReloptions>() };
    let storage_format = match unsafe {
        read_string_reloption(
            rd_options,
            reloptions.storage_format_offset,
            "storage_format",
        )
    } {
        Some(value) => {
            StorageFormat::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
        }
        None => StorageFormat::DEFAULT,
    };

    TqDiskannOptions {
        graph_degree: reloptions.graph_degree,
        build_list_size: reloptions.build_list_size,
        list_size: reloptions.list_size,
        rerank_budget: reloptions.rerank_budget,
        top_k: reloptions.top_k,
        alpha: reloptions.alpha as f32,
        storage_format,
    }
}

#[allow(dead_code)]
pub(super) fn storage_format_name(fmt: StorageFormat) -> &'static str {
    fmt.as_str()
}

#[cfg(test)]
mod tests {
    use super::{
        current_prefilter_kind, resolve_scan_tuning_values, ListSizeSource, PrefilterKind,
        ScanTuning, StorageFormat, TqDiskannOptions, ECDISKANN_DEFAULT_RERANK_BUDGET,
        ECDISKANN_DEFAULT_SCAN_LIST_SIZE, ECDISKANN_DEFAULT_TOP_K,
        ECDISKANN_SESSION_LIST_SIZE_UNSET,
    };

    #[test]
    fn diskann_default_options_include_scan_runtime_defaults() {
        let defaults = TqDiskannOptions::DEFAULT;
        assert_eq!(defaults.list_size, ECDISKANN_DEFAULT_SCAN_LIST_SIZE);
        assert_eq!(defaults.rerank_budget, ECDISKANN_DEFAULT_RERANK_BUDGET);
        assert_eq!(defaults.top_k, ECDISKANN_DEFAULT_TOP_K);
        assert_eq!(defaults.storage_format, StorageFormat::PqFastScan);
    }

    #[test]
    fn resolve_scan_tuning_uses_relation_value_when_session_is_default() {
        assert_eq!(
            resolve_scan_tuning_values(128, ECDISKANN_SESSION_LIST_SIZE_UNSET),
            ScanTuning {
                relation_list_size: 128,
                session_list_size: None,
                effective_list_size: 128,
                source: ListSizeSource::Relation,
            }
        );
    }

    #[test]
    fn resolve_scan_tuning_uses_session_value_when_non_default() {
        assert_eq!(
            resolve_scan_tuning_values(128, 512),
            ScanTuning {
                relation_list_size: 128,
                session_list_size: Some(512),
                effective_list_size: 512,
                source: ListSizeSource::Session,
            }
        );
    }

    #[test]
    fn resolve_scan_tuning_keeps_default_relation_when_both_are_default() {
        assert_eq!(
            resolve_scan_tuning_values(
                ECDISKANN_DEFAULT_SCAN_LIST_SIZE,
                ECDISKANN_SESSION_LIST_SIZE_UNSET
            ),
            ScanTuning {
                relation_list_size: ECDISKANN_DEFAULT_SCAN_LIST_SIZE,
                session_list_size: None,
                effective_list_size: ECDISKANN_DEFAULT_SCAN_LIST_SIZE,
                source: ListSizeSource::Relation,
            }
        );
    }

    #[test]
    fn resolve_scan_tuning_uses_explicit_default_session_override() {
        assert_eq!(
            resolve_scan_tuning_values(512, ECDISKANN_DEFAULT_SCAN_LIST_SIZE),
            ScanTuning {
                relation_list_size: 512,
                session_list_size: Some(ECDISKANN_DEFAULT_SCAN_LIST_SIZE),
                effective_list_size: ECDISKANN_DEFAULT_SCAN_LIST_SIZE,
                source: ListSizeSource::Session,
            }
        );
    }

    #[test]
    fn prefilter_kind_guc_defaults_to_auto() {
        assert_eq!(current_prefilter_kind(), PrefilterKind::Auto);
    }
}
