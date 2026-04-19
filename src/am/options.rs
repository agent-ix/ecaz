use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting};

use super::{
    TQHNSW_DEFAULT_EF_CONSTRUCTION, TQHNSW_DEFAULT_EF_SEARCH, TQHNSW_DEFAULT_M,
    TQHNSW_MAX_EF_CONSTRUCTION, TQHNSW_MAX_EF_SEARCH, TQHNSW_MAX_M, TQHNSW_MIN_EF_CONSTRUCTION,
    TQHNSW_MIN_EF_SEARCH, TQHNSW_MIN_M,
};

const TQHNSW_SESSION_EF_SEARCH_UNSET: i32 = -1;

static TQHNSW_EF_SEARCH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(TQHNSW_SESSION_EF_SEARCH_UNSET);
static TQHNSW_DISABLE_BINARY_PREFILTER_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);
static TQHNSW_FORCE_BINARY_DERIVATION_GUC: GucSetting<bool> = GucSetting::<bool>::new(false);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StorageFormat {
    TurboQuant,
    PqFastScan,
}

impl StorageFormat {
    pub(super) const DEFAULT: Self = Self::TurboQuant;

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::TurboQuant => "turboquant",
            Self::PqFastScan => "pq_fastscan",
        }
    }

    fn parse_reloption(raw: &str) -> Result<Self, String> {
        match raw {
            "turboquant" => Ok(Self::TurboQuant),
            "pq_fastscan" => Ok(Self::PqFastScan),
            other => Err(format!(
                "invalid tqhnsw storage_format reloption: expected one of [turboquant, pq_fastscan], got {:?}",
                other
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct TqHnswOptions {
    pub(super) m: i32,
    pub(super) ef_construction: i32,
    pub(super) ef_search: i32,
    pub(super) build_source_column: Option<String>,
    pub(super) rerank_source_column: Option<String>,
    pub(super) storage_format: StorageFormat,
}

impl TqHnswOptions {
    const DEFAULT: Self = Self {
        m: TQHNSW_DEFAULT_M,
        ef_construction: TQHNSW_DEFAULT_EF_CONSTRUCTION,
        ef_search: TQHNSW_DEFAULT_EF_SEARCH,
        build_source_column: None,
        rerank_source_column: None,
        storage_format: StorageFormat::DEFAULT,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EfSearchSource {
    Relation,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ScanTuning {
    pub(super) relation_ef_search: i32,
    pub(super) session_ef_search: Option<i32>,
    pub(super) effective_ef_search: i32,
    pub(super) source: EfSearchSource,
}

pub(super) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"tqhnsw.ef_search",
        c"Session override for tqhnsw search breadth.",
        c"Overrides tqhnsw index ef_search reloptions when set to 1-1000; -1 uses the relation value.",
        &TQHNSW_EF_SEARCH_GUC,
        TQHNSW_SESSION_EF_SEARCH_UNSET,
        TQHNSW_MAX_EF_SEARCH,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"tqhnsw.disable_binary_prefilter",
        c"Disable ADR-031 binary prefilter runtime behavior.",
        c"Diagnostic override used for A/B comparison; when enabled, tqhnsw skips ADR-031 binary-query preparation so scans fall back to the pre-ADR-031 eager exact-scoring path.",
        &TQHNSW_DISABLE_BINARY_PREFILTER_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_bool_guc(
        c"tqhnsw.force_binary_derivation",
        c"Force ADR-031 scans to ignore persisted binary sidecars.",
        c"Diagnostic override used for A/B comparison; when enabled, tqhnsw derives binary words from code bytes even if persisted sidecars are present.",
        &TQHNSW_FORCE_BINARY_DERIVATION_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_session_ef_search() -> i32 {
    TQHNSW_EF_SEARCH_GUC.get()
}

#[cfg(test)]
pub(super) fn disable_binary_prefilter() -> bool {
    false
}

#[cfg(not(test))]
pub(super) fn disable_binary_prefilter() -> bool {
    TQHNSW_DISABLE_BINARY_PREFILTER_GUC.get()
}

#[cfg(test)]
pub(super) fn force_binary_derivation() -> bool {
    false
}

#[cfg(not(test))]
pub(super) fn force_binary_derivation() -> bool {
    TQHNSW_FORCE_BINARY_DERIVATION_GUC.get()
}

pub(super) fn resolve_scan_tuning(options: &TqHnswOptions) -> ScanTuning {
    resolve_scan_tuning_values(options.ef_search, current_session_ef_search())
}

fn resolve_scan_tuning_values(relation_ef_search: i32, session_ef_search: i32) -> ScanTuning {
    if session_ef_search == TQHNSW_SESSION_EF_SEARCH_UNSET {
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

pub(super) unsafe extern "C-unwind" fn tqhnsw_amoptions(
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
                TQHNSW_DEFAULT_M,
                TQHNSW_MIN_M,
                TQHNSW_MAX_M,
                offset_of!(TqHnswReloptions, m) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"ef_construction\0".as_ptr().cast(),
                b"Candidate list width used during graph construction.\0"
                    .as_ptr()
                    .cast(),
                TQHNSW_DEFAULT_EF_CONSTRUCTION,
                TQHNSW_MIN_EF_CONSTRUCTION,
                TQHNSW_MAX_EF_CONSTRUCTION,
                offset_of!(TqHnswReloptions, ef_construction) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"ef_search\0".as_ptr().cast(),
                b"Candidate list width used during scan search.\0"
                    .as_ptr()
                    .cast(),
                TQHNSW_DEFAULT_EF_SEARCH,
                TQHNSW_MIN_EF_SEARCH,
                TQHNSW_MAX_EF_SEARCH,
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
        .unwrap_or_else(|e| pgrx::error!("invalid tqhnsw {name} reloption: {e}"));
    if value.is_empty() {
        pgrx::error!("invalid tqhnsw {name} reloption: value must not be empty");
    }
    Some(value.to_owned())
}

pub(super) unsafe fn relation_options(index_relation: pg_sys::Relation) -> TqHnswOptions {
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
        EfSearchSource, ScanTuning, StorageFormat, TQHNSW_DEFAULT_EF_SEARCH,
        TQHNSW_SESSION_EF_SEARCH_UNSET,
    };

    #[test]
    fn resolve_scan_tuning_uses_relation_value_when_session_is_default() {
        assert_eq!(
            resolve_scan_tuning_values(128, TQHNSW_SESSION_EF_SEARCH_UNSET),
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
            resolve_scan_tuning_values(TQHNSW_DEFAULT_EF_SEARCH, TQHNSW_SESSION_EF_SEARCH_UNSET),
            ScanTuning {
                relation_ef_search: TQHNSW_DEFAULT_EF_SEARCH,
                session_ef_search: None,
                effective_ef_search: TQHNSW_DEFAULT_EF_SEARCH,
                source: EfSearchSource::Relation,
            }
        );
    }

    #[test]
    fn resolve_scan_tuning_uses_explicit_default_session_override() {
        assert_eq!(
            resolve_scan_tuning_values(128, TQHNSW_DEFAULT_EF_SEARCH),
            ScanTuning {
                relation_ef_search: 128,
                session_ef_search: Some(TQHNSW_DEFAULT_EF_SEARCH),
                effective_ef_search: TQHNSW_DEFAULT_EF_SEARCH,
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
