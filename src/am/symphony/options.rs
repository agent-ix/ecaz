use std::mem::{offset_of, size_of};

use pgrx::pg_sys;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SymphonyReloptions {
    vl_len_: i32,
    m: i32,
    ef_construction: i32,
    padding_factor: i32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SymphonyOptions {
    pub(super) m: i32,
    pub(super) ef_construction: i32,
    pub(super) padding_factor: i32,
}

impl SymphonyOptions {
    const DEFAULT: Self = Self {
        m: super::SYMPHONY_DEFAULT_M as i32,
        ef_construction: super::SYMPHONY_DEFAULT_EF_CONSTRUCTION as i32,
        padding_factor: super::SYMPHONY_BOOTSTRAP_PADDING_FACTOR as i32,
    };
}

pub(super) unsafe extern "C-unwind" fn symphony_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut relopts = pg_sys::local_relopts::default();

            pg_sys::init_local_reloptions(&mut relopts, size_of::<SymphonyReloptions>());
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"m\0".as_ptr().cast(),
                b"Maximum graph degree per layer.\0".as_ptr().cast(),
                super::SYMPHONY_DEFAULT_M as i32,
                super::SYMPHONY_MIN_M,
                super::SYMPHONY_MAX_M,
                offset_of!(SymphonyReloptions, m) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"ef_construction\0".as_ptr().cast(),
                b"Candidate list width used during graph construction.\0"
                    .as_ptr()
                    .cast(),
                super::SYMPHONY_DEFAULT_EF_CONSTRUCTION as i32,
                super::SYMPHONY_MIN_EF_CONSTRUCTION,
                super::SYMPHONY_MAX_EF_CONSTRUCTION,
                offset_of!(SymphonyReloptions, ef_construction) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"padding_factor\0".as_ptr().cast(),
                b"Neighbor-list padding factor; 1 disables padding for the Phase-0 oracle seam.\0"
                    .as_ptr()
                    .cast(),
                super::SYMPHONY_BOOTSTRAP_PADDING_FACTOR as i32,
                super::SYMPHONY_MIN_PADDING_FACTOR,
                super::SYMPHONY_MAX_PADDING_FACTOR,
                offset_of!(SymphonyReloptions, padding_factor) as i32,
            );
            pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
        })
    }
}

pub(super) unsafe fn relation_options(index_relation: pg_sys::Relation) -> SymphonyOptions {
    let rd_options = unsafe { (*index_relation).rd_options };
    if rd_options.is_null() {
        return SymphonyOptions::DEFAULT;
    }

    let reloptions = unsafe { &*rd_options.cast::<SymphonyReloptions>() };
    SymphonyOptions {
        m: reloptions.m,
        ef_construction: reloptions.ef_construction,
        padding_factor: reloptions.padding_factor,
    }
}
