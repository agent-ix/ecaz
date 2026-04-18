use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::pg_sys;

use super::{
    TQDISKANN_DEFAULT_ALPHA, TQDISKANN_DEFAULT_BUILD_LIST_SIZE, TQDISKANN_DEFAULT_GRAPH_DEGREE,
    TQDISKANN_MAX_ALPHA, TQDISKANN_MAX_BUILD_LIST_SIZE, TQDISKANN_MAX_GRAPH_DEGREE,
    TQDISKANN_MIN_ALPHA, TQDISKANN_MIN_BUILD_LIST_SIZE, TQDISKANN_MIN_GRAPH_DEGREE,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TqDiskannReloptions {
    vl_len_: i32,
    graph_degree: i32,
    build_list_size: i32,
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
                "invalid tqdiskann storage_format reloption: expected 'pq_fastscan', got {:?}",
                other
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct TqDiskannOptions {
    pub(super) graph_degree: i32,
    pub(super) build_list_size: i32,
    pub(super) alpha: f32,
    pub(super) storage_format: StorageFormat,
}

impl TqDiskannOptions {
    pub(super) const DEFAULT: Self = Self {
        graph_degree: TQDISKANN_DEFAULT_GRAPH_DEGREE,
        build_list_size: TQDISKANN_DEFAULT_BUILD_LIST_SIZE,
        alpha: TQDISKANN_DEFAULT_ALPHA,
        storage_format: StorageFormat::DEFAULT,
    };
}

pub(super) unsafe extern "C-unwind" fn tqdiskann_amoptions(
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
                TQDISKANN_DEFAULT_GRAPH_DEGREE,
                TQDISKANN_MIN_GRAPH_DEGREE,
                TQDISKANN_MAX_GRAPH_DEGREE,
                offset_of!(TqDiskannReloptions, graph_degree) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"build_list_size\0".as_ptr().cast(),
                b"Candidate list width used during Vamana graph construction (L).\0"
                    .as_ptr()
                    .cast(),
                TQDISKANN_DEFAULT_BUILD_LIST_SIZE,
                TQDISKANN_MIN_BUILD_LIST_SIZE,
                TQDISKANN_MAX_BUILD_LIST_SIZE,
                offset_of!(TqDiskannReloptions, build_list_size) as i32,
            );
            pg_sys::add_local_real_reloption(
                &mut relopts,
                b"alpha\0".as_ptr().cast(),
                b"Vamana alpha-pruning slack; pgvectorscale-compatible f32.\0"
                    .as_ptr()
                    .cast(),
                TQDISKANN_DEFAULT_ALPHA as f64,
                TQDISKANN_MIN_ALPHA as f64,
                TQDISKANN_MAX_ALPHA as f64,
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
        .unwrap_or_else(|e| pgrx::error!("invalid tqdiskann {name} reloption: {e}"));
    if value.is_empty() {
        pgrx::error!("invalid tqdiskann {name} reloption: value must not be empty");
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
        Some(value) => StorageFormat::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}")),
        None => StorageFormat::DEFAULT,
    };

    TqDiskannOptions {
        graph_degree: reloptions.graph_degree,
        build_list_size: reloptions.build_list_size,
        alpha: reloptions.alpha as f32,
        storage_format,
    }
}

#[allow(dead_code)]
pub(super) fn storage_format_name(fmt: StorageFormat) -> &'static str {
    fmt.as_str()
}
