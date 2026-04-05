use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::pg_sys;

use super::{
    TQHNSW_DEFAULT_EF_CONSTRUCTION, TQHNSW_DEFAULT_M, TQHNSW_MAX_EF_CONSTRUCTION, TQHNSW_MAX_M,
    TQHNSW_MIN_EF_CONSTRUCTION, TQHNSW_MIN_M,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TqHnswReloptions {
    vl_len_: i32,
    m: i32,
    ef_construction: i32,
    build_source_column_offset: i32,
}

#[derive(Debug, Clone)]
pub(super) struct TqHnswOptions {
    pub(super) m: i32,
    pub(super) ef_construction: i32,
    pub(super) build_source_column: Option<String>,
}

impl TqHnswOptions {
    const DEFAULT: Self = Self {
        m: TQHNSW_DEFAULT_M,
        ef_construction: TQHNSW_DEFAULT_EF_CONSTRUCTION,
        build_source_column: None,
    };
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
            pg_sys::add_local_string_reloption(
                &mut relopts,
                b"build_source_column\0".as_ptr().cast(),
                b"Optional heap column name supplying raw real[] vectors for ambuild graph construction.\0"
                    .as_ptr()
                    .cast(),
                ptr::null(),
                None,
                None,
                offset_of!(TqHnswReloptions, build_source_column_offset) as i32,
            );
            pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
        })
    }
}

pub(super) unsafe fn relation_options(index_relation: pg_sys::Relation) -> TqHnswOptions {
    let rd_options = unsafe { (*index_relation).rd_options };
    if rd_options.is_null() {
        return TqHnswOptions::DEFAULT;
    }

    let reloptions = unsafe { &*rd_options.cast::<TqHnswReloptions>() };
    let build_source_column = if reloptions.build_source_column_offset == 0 {
        None
    } else {
        let value_ptr = unsafe {
            rd_options
                .cast::<u8>()
                .add(reloptions.build_source_column_offset as usize)
                .cast::<std::ffi::c_char>()
        };
        let value = unsafe { std::ffi::CStr::from_ptr(value_ptr) }
            .to_str()
            .unwrap_or_else(|e| pgrx::error!("invalid tqhnsw build_source_column reloption: {e}"));
        if value.is_empty() {
            pgrx::error!("invalid tqhnsw build_source_column reloption: value must not be empty");
        }
        Some(value.to_owned())
    };

    TqHnswOptions {
        m: reloptions.m,
        ef_construction: reloptions.ef_construction,
        build_source_column,
    }
}
