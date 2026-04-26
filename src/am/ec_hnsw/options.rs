use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting};

use super::{
    EC_HNSW_DEFAULT_EF_CONSTRUCTION, EC_HNSW_DEFAULT_EF_SEARCH, EC_HNSW_DEFAULT_M,
    EC_HNSW_MAX_EF_CONSTRUCTION, EC_HNSW_MAX_EF_SEARCH, EC_HNSW_MAX_M, EC_HNSW_MIN_EF_CONSTRUCTION,
    EC_HNSW_MIN_EF_SEARCH, EC_HNSW_MIN_M,
};
pub(crate) use crate::quant::Family as StorageFormat;

const EC_HNSW_SESSION_EF_SEARCH_UNSET: i32 = -1;

static EC_HNSW_EF_SEARCH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_HNSW_SESSION_EF_SEARCH_UNSET);
static EC_HNSW_DISABLE_BINARY_PREFILTER_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
static EC_HNSW_FORCE_BINARY_DERIVATION_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
static EC_HNSW_ENABLE_PARALLEL_BUILD_CONCURRENT_DSM_GUC: GucSetting<bool> =
    GucSetting::<bool>::new(true);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TqHnswReloptions {
    vl_len_: i32,
    m: i32,
    ef_construction: i32,
    ef_search: i32,
    build_source_column_offset: i32,
    rerank_source_column_offset: i32,
    storage_format_offset: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct TqHnswOptions {
    pub(crate) m: i32,
    pub(crate) ef_construction: i32,
    pub(crate) ef_search: i32,
    pub(crate) build_source_column: Option<String>,
    pub(crate) rerank_source_column: Option<String>,
    pub(crate) storage_format: StorageFormat,
}

impl TqHnswOptions {
    const DEFAULT: Self = Self {
        m: EC_HNSW_DEFAULT_M,
        ef_construction: EC_HNSW_DEFAULT_EF_CONSTRUCTION,
        ef_search: EC_HNSW_DEFAULT_EF_SEARCH,
        build_source_column: None,
        rerank_source_column: None,
        storage_format: StorageFormat::DEFAULT,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EfSearchSource {
    Relation,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScanTuning {
    pub(crate) relation_ef_search: i32,
    pub(crate) session_ef_search: Option<i32>,
    pub(crate) effective_ef_search: i32,
    pub(crate) source: EfSearchSource,
}

pub(super) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ec_hnsw.ef_search",
        c"Session override for ec_hnsw search breadth.",
        c"Overrides ec_hnsw index ef_search reloptions when set to 1-1000; -1 uses the relation value.",
        &EC_HNSW_EF_SEARCH_GUC,
        EC_HNSW_SESSION_EF_SEARCH_UNSET,
        EC_HNSW_MAX_EF_SEARCH,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_hnsw.disable_binary_prefilter",
        c"Disable ADR-031 binary prefilter runtime behavior.",
        c"Diagnostic override used for A/B comparison; when enabled, ec_hnsw skips ADR-031 binary-query preparation so scans fall back to the pre-ADR-031 eager exact-scoring path.",
        &EC_HNSW_DISABLE_BINARY_PREFILTER_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_hnsw.force_binary_derivation",
        c"Force ADR-031 scans to ignore persisted binary sidecars.",
        c"Diagnostic override used for A/B comparison; when enabled, ec_hnsw derives binary words from code bytes even if persisted sidecars are present.",
        &EC_HNSW_FORCE_BINARY_DERIVATION_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"ec_hnsw.enable_parallel_build_concurrent_dsm",
        c"Enable concurrent DSM graph assembly for parallel ec_hnsw builds.",
        c"Phase-4 ADR-048 default path; when enabled, eligible parallel builds assemble the HNSW graph through a DSM-resident graph instead of serial leader graph construction. Disable only as a diagnostic fallback.",
        &EC_HNSW_ENABLE_PARALLEL_BUILD_CONCURRENT_DSM_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_session_ef_search() -> i32 {
    EC_HNSW_EF_SEARCH_GUC.get()
}

#[cfg(test)]
pub(super) fn disable_binary_prefilter() -> bool {
    false
}

#[cfg(not(test))]
pub(super) fn disable_binary_prefilter() -> bool {
    EC_HNSW_DISABLE_BINARY_PREFILTER_GUC.get()
}

#[cfg(test)]
pub(super) fn force_binary_derivation() -> bool {
    false
}

#[cfg(not(test))]
pub(super) fn force_binary_derivation() -> bool {
    EC_HNSW_FORCE_BINARY_DERIVATION_GUC.get()
}

pub(super) fn enable_parallel_build_concurrent_dsm() -> bool {
    EC_HNSW_ENABLE_PARALLEL_BUILD_CONCURRENT_DSM_GUC.get()
}

pub(crate) fn resolve_scan_tuning(options: &TqHnswOptions) -> ScanTuning {
    resolve_scan_tuning_values(options.ef_search, current_session_ef_search())
}

fn resolve_scan_tuning_values(relation_ef_search: i32, session_ef_search: i32) -> ScanTuning {
    if session_ef_search == EC_HNSW_SESSION_EF_SEARCH_UNSET {
        ScanTuning {
            relation_ef_search,
            session_ef_search: None,
            effective_ef_search: relation_ef_search,
            source: EfSearchSource::Relation,
        }
    } else {
        ScanTuning {
            relation_ef_search,
            session_ef_search: Some(session_ef_search),
            effective_ef_search: session_ef_search,
            source: EfSearchSource::Session,
        }
    }
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut relopts = pg_sys::local_relopts::default();

            pg_sys::init_local_reloptions(&mut relopts, size_of::<TqHnswReloptions>());
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"m\0".as_ptr().cast(),
                b"Maximum graph degree per layer.\0".as_ptr().cast(),
                EC_HNSW_DEFAULT_M,
                EC_HNSW_MIN_M,
                EC_HNSW_MAX_M,
                offset_of!(TqHnswReloptions, m) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"ef_construction\0".as_ptr().cast(),
                b"Candidate list width used during graph construction.\0"
                    .as_ptr()
                    .cast(),
                EC_HNSW_DEFAULT_EF_CONSTRUCTION,
                EC_HNSW_MIN_EF_CONSTRUCTION,
                EC_HNSW_MAX_EF_CONSTRUCTION,
                offset_of!(TqHnswReloptions, ef_construction) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"ef_search\0".as_ptr().cast(),
                b"Candidate list width used during scan search.\0"
                    .as_ptr()
                    .cast(),
                EC_HNSW_DEFAULT_EF_SEARCH,
                EC_HNSW_MIN_EF_SEARCH,
                EC_HNSW_MAX_EF_SEARCH,
                offset_of!(TqHnswReloptions, ef_search) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                b"build_source_column\0".as_ptr().cast(),
                b"Optional alternate heap column name supplying raw real[] or ecvector values for source-backed graph construction instead of the indexed ecvector column.\0"
                    .as_ptr()
                    .cast(),
                ptr::null(),
                None,
                None,
                offset_of!(TqHnswReloptions, build_source_column_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                b"rerank_source_column\0".as_ptr().cast(),
                b"Optional alternate heap column name supplying raw real[], bytea, or ecvector values for grouped heap_f32 rerank instead of the indexed ecvector column.\0"
                    .as_ptr()
                    .cast(),
                ptr::null(),
                None,
                None,
                offset_of!(TqHnswReloptions, rerank_source_column_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                b"storage_format\0".as_ptr().cast(),
                b"Index storage format: 'turboquant' (default) or 'pq_fastscan'.\0"
                    .as_ptr()
                    .cast(),
                ptr::null(),
                None,
                None,
                offset_of!(TqHnswReloptions, storage_format_offset) as i32,
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
        .unwrap_or_else(|e| pgrx::error!("invalid ec_hnsw {name} reloption: {e}"));
    if value.is_empty() {
        pgrx::error!("invalid ec_hnsw {name} reloption: value must not be empty");
    }
    Some(value.to_owned())
}

pub(crate) unsafe fn relation_options(index_relation: pg_sys::Relation) -> TqHnswOptions {
    let rd_options = unsafe { (*index_relation).rd_options };
    if rd_options.is_null() {
        return TqHnswOptions::DEFAULT;
    }

    let reloptions = unsafe { &*rd_options.cast::<TqHnswReloptions>() };
    let build_source_column = unsafe {
        read_string_reloption(
            rd_options,
            reloptions.build_source_column_offset,
            "build_source_column",
        )
    };
    let rerank_source_column = unsafe {
        read_string_reloption(
            rd_options,
            reloptions.rerank_source_column_offset,
            "rerank_source_column",
        )
    };
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

    TqHnswOptions {
        m: reloptions.m,
        ef_construction: reloptions.ef_construction,
        ef_search: reloptions.ef_search,
        build_source_column,
        rerank_source_column,
        storage_format,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        disable_binary_prefilter, force_binary_derivation, resolve_scan_tuning_values,
        EfSearchSource, ScanTuning, StorageFormat, EC_HNSW_DEFAULT_EF_SEARCH,
        EC_HNSW_SESSION_EF_SEARCH_UNSET,
    };

    #[test]
    fn resolve_scan_tuning_uses_relation_value_when_session_is_default() {
        assert_eq!(
            resolve_scan_tuning_values(128, EC_HNSW_SESSION_EF_SEARCH_UNSET),
            ScanTuning {
                relation_ef_search: 128,
                session_ef_search: None,
                effective_ef_search: 128,
                source: EfSearchSource::Relation,
            }
        );
    }

    #[test]
    fn resolve_scan_tuning_uses_session_value_when_non_default() {
        assert_eq!(
            resolve_scan_tuning_values(128, 7),
            ScanTuning {
                relation_ef_search: 128,
                session_ef_search: Some(7),
                effective_ef_search: 7,
                source: EfSearchSource::Session,
            }
        );
    }

    #[test]
    fn resolve_scan_tuning_keeps_default_relation_when_both_are_default() {
        assert_eq!(
            resolve_scan_tuning_values(EC_HNSW_DEFAULT_EF_SEARCH, EC_HNSW_SESSION_EF_SEARCH_UNSET),
            ScanTuning {
                relation_ef_search: EC_HNSW_DEFAULT_EF_SEARCH,
                session_ef_search: None,
                effective_ef_search: EC_HNSW_DEFAULT_EF_SEARCH,
                source: EfSearchSource::Relation,
            }
        );
    }

    #[test]
    fn resolve_scan_tuning_uses_explicit_default_session_override() {
        assert_eq!(
            resolve_scan_tuning_values(128, EC_HNSW_DEFAULT_EF_SEARCH),
            ScanTuning {
                relation_ef_search: 128,
                session_ef_search: Some(EC_HNSW_DEFAULT_EF_SEARCH),
                effective_ef_search: EC_HNSW_DEFAULT_EF_SEARCH,
                source: EfSearchSource::Session,
            }
        );
    }

    #[test]
    fn force_binary_derivation_defaults_off() {
        assert!(!force_binary_derivation());
    }

    #[test]
    fn disable_binary_prefilter_defaults_off() {
        assert!(!disable_binary_prefilter());
    }

    #[test]
    fn storage_format_reloption_accepts_supported_values() {
        assert_eq!(
            StorageFormat::parse_reloption("turboquant"),
            Ok(StorageFormat::TurboQuant)
        );
        assert_eq!(
            StorageFormat::parse_reloption("pq_fastscan"),
            Ok(StorageFormat::PqFastScan)
        );
    }

    #[test]
    fn storage_format_reloption_rejects_unknown_values() {
        let error = StorageFormat::parse_reloption("legacy_format").unwrap_err();
        assert!(error.contains("storage_format"));
        assert!(error.contains("turboquant"));
        assert!(error.contains("pq_fastscan"));
    }
}
