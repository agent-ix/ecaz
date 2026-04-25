use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting};

use super::{
    EC_IVF_DEFAULT_NLISTS, EC_IVF_DEFAULT_NPROBE, EC_IVF_DEFAULT_SEED,
    EC_IVF_DEFAULT_TRAINING_SAMPLE_ROWS, EC_IVF_MAX_NLISTS, EC_IVF_MAX_NPROBE, EC_IVF_MAX_SEED,
    EC_IVF_MAX_TRAINING_SAMPLE_ROWS, EC_IVF_MIN_NLISTS, EC_IVF_MIN_NPROBE, EC_IVF_MIN_SEED,
    EC_IVF_MIN_TRAINING_SAMPLE_ROWS,
};

const EC_IVF_SESSION_NPROBE_UNSET: i32 = -1;

static EC_IVF_NPROBE_GUC: GucSetting<i32> = GucSetting::<i32>::new(EC_IVF_SESSION_NPROBE_UNSET);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct EcIvfReloptions {
    vl_len_: i32,
    nlists: i32,
    nprobe: i32,
    training_sample_rows: i32,
    seed: i32,
    storage_format_offset: i32,
    rerank_offset: i32,
}

pub(super) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ec_ivf.nprobe",
        c"Session override for ec_ivf posting-list probe count.",
        c"Overrides ec_ivf index nprobe reloption when set to 1 or higher; -1 uses the relation value.",
        &EC_IVF_NPROBE_GUC,
        EC_IVF_SESSION_NPROBE_UNSET,
        EC_IVF_MAX_NPROBE,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut relopts = pg_sys::local_relopts::default();

            pg_sys::init_local_reloptions(&mut relopts, size_of::<EcIvfReloptions>());
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"nlists".as_ptr(),
                c"Number of IVF centroid posting lists; 0 chooses an automatic value.".as_ptr(),
                EC_IVF_DEFAULT_NLISTS,
                EC_IVF_MIN_NLISTS,
                EC_IVF_MAX_NLISTS,
                offset_of!(EcIvfReloptions, nlists) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"nprobe".as_ptr(),
                c"Number of IVF posting lists to probe during scan; 0 chooses an automatic value."
                    .as_ptr(),
                EC_IVF_DEFAULT_NPROBE,
                EC_IVF_MIN_NPROBE,
                EC_IVF_MAX_NPROBE,
                offset_of!(EcIvfReloptions, nprobe) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"training_sample_rows".as_ptr(),
                c"Maximum rows sampled for centroid training; 0 chooses an automatic value."
                    .as_ptr(),
                EC_IVF_DEFAULT_TRAINING_SAMPLE_ROWS,
                EC_IVF_MIN_TRAINING_SAMPLE_ROWS,
                EC_IVF_MAX_TRAINING_SAMPLE_ROWS,
                offset_of!(EcIvfReloptions, training_sample_rows) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"seed".as_ptr(),
                c"Deterministic seed for IVF centroid training.".as_ptr(),
                EC_IVF_DEFAULT_SEED,
                EC_IVF_MIN_SEED,
                EC_IVF_MAX_SEED,
                offset_of!(EcIvfReloptions, seed) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"storage_format".as_ptr(),
                c"IVF posting-list quantizer profile: 'turboquant', 'pq_fastscan', 'rabitq', or 'auto'."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcIvfReloptions, storage_format_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"rerank".as_ptr(),
                c"IVF rerank mode: 'off', 'heap_f32', 'source_column', or 'auto'.".as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcIvfReloptions, rerank_offset) as i32,
            );
            pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
        })
    }
}
