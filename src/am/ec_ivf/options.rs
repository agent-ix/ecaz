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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StorageFormat {
    Auto = 0,
    TurboQuant = 1,
    PqFastScan = 2,
    RaBitQ = 3,
}

impl StorageFormat {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "turboquant" => Ok(Self::TurboQuant),
            "pq_fastscan" => Ok(Self::PqFastScan),
            "rabitq" => Ok(Self::RaBitQ),
            other => Err(format!(
                "invalid ec_ivf storage_format reloption: expected 'auto', 'turboquant', 'pq_fastscan', or 'rabitq', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::TurboQuant => "turboquant",
            Self::PqFastScan => "pq_fastscan",
            Self::RaBitQ => "rabitq",
        }
    }

    pub(super) fn validate_v1_supported(self) -> Result<(), String> {
        match self {
            Self::Auto | Self::TurboQuant => Ok(()),
            Self::PqFastScan | Self::RaBitQ => Err(format!(
                "ec_ivf storage_format {} is not supported yet; use storage_format = 'auto' or storage_format = 'turboquant'",
                self.reloption_name()
            )),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RerankMode {
    Auto = 0,
    Off = 1,
    HeapF32 = 2,
    SourceColumn = 3,
}

impl RerankMode {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "off" => Ok(Self::Off),
            "heap_f32" => Ok(Self::HeapF32),
            "source_column" => Ok(Self::SourceColumn),
            other => Err(format!(
                "invalid ec_ivf rerank reloption: expected 'auto', 'off', 'heap_f32', or 'source_column', got '{other}'"
            )),
        }
    }

    pub(super) fn reloption_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Off => "off",
            Self::HeapF32 => "heap_f32",
            Self::SourceColumn => "source_column",
        }
    }

    pub(super) fn v1_effective(self) -> Self {
        match self {
            Self::Auto => Self::Off,
            other => other,
        }
    }

    pub(super) fn validate_v1_supported(self) -> Result<(), String> {
        match self {
            Self::Auto | Self::Off | Self::HeapF32 => Ok(()),
            Self::SourceColumn => Err(format!(
                "ec_ivf rerank mode {} is not supported yet; use rerank = 'off', rerank = 'auto', or rerank = 'heap_f32'",
                self.reloption_name()
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct EcIvfOptions {
    pub(super) nlists: i32,
    pub(super) nprobe: i32,
    pub(super) training_sample_rows: i32,
    pub(super) seed: i32,
    pub(super) storage_format: StorageFormat,
    pub(super) rerank: RerankMode,
}

impl EcIvfOptions {
    const DEFAULT: Self = Self {
        nlists: EC_IVF_DEFAULT_NLISTS,
        nprobe: EC_IVF_DEFAULT_NPROBE,
        training_sample_rows: EC_IVF_DEFAULT_TRAINING_SAMPLE_ROWS,
        seed: EC_IVF_DEFAULT_SEED,
        storage_format: StorageFormat::Auto,
        rerank: RerankMode::Auto,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NprobeResolution {
    pub(super) relation_nprobe: u32,
    pub(super) session_nprobe: Option<u32>,
    pub(super) effective_nprobe: u32,
    pub(super) source: &'static str,
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

pub(super) fn current_session_nprobe() -> i32 {
    EC_IVF_NPROBE_GUC.get()
}

pub(super) fn resolve_scan_nprobe(nlists: u32, relation_nprobe: u32) -> NprobeResolution {
    let session_nprobe = match current_session_nprobe() {
        value if value > 0 => Some(value as u32),
        _ => None,
    };
    if nlists == 0 {
        return NprobeResolution {
            relation_nprobe,
            session_nprobe,
            effective_nprobe: 0,
            source: "none",
        };
    }

    let (requested, source) = match session_nprobe {
        Some(value) => (value, "session"),
        None if relation_nprobe == 0 => (auto_nprobe(nlists), "auto"),
        None => (relation_nprobe, "relation"),
    };

    NprobeResolution {
        relation_nprobe,
        session_nprobe,
        effective_nprobe: requested.clamp(1, nlists),
        source,
    }
}

fn auto_nprobe(nlists: u32) -> u32 {
    if nlists == 0 {
        return 0;
    }
    (nlists as f64).sqrt().ceil() as u32
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
        .unwrap_or_else(|e| pgrx::error!("invalid ec_ivf {name} reloption: {e}"));
    if value.is_empty() {
        pgrx::error!("invalid ec_ivf {name} reloption: value must not be empty");
    }
    Some(value.to_owned())
}

pub(super) unsafe fn relation_options(index_relation: pg_sys::Relation) -> EcIvfOptions {
    let rd_options = unsafe { (*index_relation).rd_options };
    if rd_options.is_null() {
        return EcIvfOptions::DEFAULT;
    }

    let reloptions = unsafe { &*rd_options.cast::<EcIvfReloptions>() };
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
        None => StorageFormat::Auto,
    };
    let rerank = match unsafe {
        read_string_reloption(rd_options, reloptions.rerank_offset, "rerank")
    } {
        Some(value) => RerankMode::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}")),
        None => RerankMode::Auto,
    };

    EcIvfOptions {
        nlists: reloptions.nlists,
        nprobe: reloptions.nprobe,
        training_sample_rows: reloptions.training_sample_rows,
        seed: reloptions.seed,
        storage_format,
        rerank,
    }
}
