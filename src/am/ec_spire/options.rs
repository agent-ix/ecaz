use std::ffi::CString;
use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting};

use super::quantizer::SpireAssignmentPayloadFormat;
use super::{
    EC_SPIRE_DEFAULT_BOUNDARY_REPLICA_COUNT, EC_SPIRE_DEFAULT_LOCAL_STORE_COUNT,
    EC_SPIRE_DEFAULT_NLISTS, EC_SPIRE_DEFAULT_NPROBE, EC_SPIRE_DEFAULT_PQ_GROUP_SIZE,
    EC_SPIRE_DEFAULT_RECURSIVE_FANOUT, EC_SPIRE_DEFAULT_RERANK_WIDTH, EC_SPIRE_DEFAULT_SEED,
    EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS, EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT,
    EC_SPIRE_MAX_LOCAL_STORE_COUNT, EC_SPIRE_MAX_NLISTS, EC_SPIRE_MAX_NPROBE,
    EC_SPIRE_MAX_PQ_GROUP_SIZE, EC_SPIRE_MAX_RECURSIVE_FANOUT, EC_SPIRE_MAX_RERANK_WIDTH,
    EC_SPIRE_MAX_SEED, EC_SPIRE_MAX_TRAINING_SAMPLE_ROWS, EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT,
    EC_SPIRE_MIN_LOCAL_STORE_COUNT, EC_SPIRE_MIN_NLISTS, EC_SPIRE_MIN_NPROBE,
    EC_SPIRE_MIN_PQ_GROUP_SIZE, EC_SPIRE_MIN_RECURSIVE_FANOUT, EC_SPIRE_MIN_RERANK_WIDTH,
    EC_SPIRE_MIN_SEED, EC_SPIRE_MIN_TRAINING_SAMPLE_ROWS,
};

const EC_SPIRE_SESSION_NPROBE_UNSET: i32 = -1;
const EC_SPIRE_SESSION_RERANK_WIDTH_UNSET: i32 = -1;

static EC_SPIRE_NPROBE_GUC: GucSetting<i32> = GucSetting::<i32>::new(EC_SPIRE_SESSION_NPROBE_UNSET);
static EC_SPIRE_RERANK_WIDTH_GUC: GucSetting<i32> =
    GucSetting::<i32>::new(EC_SPIRE_SESSION_RERANK_WIDTH_UNSET);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct EcSpireReloptions {
    vl_len_: i32,
    nlists: i32,
    recursive_fanout: i32,
    local_store_count: i32,
    boundary_replica_count: i32,
    nprobe: i32,
    rerank_width: i32,
    training_sample_rows: i32,
    seed: i32,
    pq_group_size: i32,
    storage_format_offset: i32,
    quantizer_offset: i32,
    local_store_tablespaces_offset: i32,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireStorageFormat {
    Auto = 0,
    TurboQuant = 1,
    PqFastScan = 2,
    RaBitQ = 3,
}

impl SpireStorageFormat {
    pub(super) fn parse_reloption(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "turboquant" => Ok(Self::TurboQuant),
            "pq_fastscan" => Ok(Self::PqFastScan),
            "rabitq" => Ok(Self::RaBitQ),
            other => Err(format!(
                "invalid ec_spire storage_format reloption: expected 'auto', 'turboquant', 'pq_fastscan', or 'rabitq', got '{other}'"
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

    pub(super) fn assignment_payload_format(self) -> SpireAssignmentPayloadFormat {
        match self {
            Self::Auto | Self::TurboQuant => SpireAssignmentPayloadFormat::TurboQuant,
            Self::PqFastScan => SpireAssignmentPayloadFormat::PqFastScan,
            Self::RaBitQ => SpireAssignmentPayloadFormat::RaBitQ,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct EcSpireOptions {
    pub(super) nlists: i32,
    pub(super) recursive_fanout: i32,
    pub(super) local_store_count: i32,
    pub(super) boundary_replica_count: i32,
    pub(super) nprobe: i32,
    pub(super) rerank_width: i32,
    pub(super) training_sample_rows: i32,
    pub(super) seed: i32,
    pub(super) pq_group_size: i32,
    pub(super) storage_format: SpireStorageFormat,
    pub(super) local_store_tablespaces: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireLocalStoreTablespacePlanEntry {
    pub(super) local_store_id: u32,
    pub(super) tablespace_oid: u32,
}

impl EcSpireOptions {
    const DEFAULT: Self = Self {
        nlists: EC_SPIRE_DEFAULT_NLISTS,
        recursive_fanout: EC_SPIRE_DEFAULT_RECURSIVE_FANOUT,
        local_store_count: EC_SPIRE_DEFAULT_LOCAL_STORE_COUNT,
        boundary_replica_count: EC_SPIRE_DEFAULT_BOUNDARY_REPLICA_COUNT,
        nprobe: EC_SPIRE_DEFAULT_NPROBE,
        rerank_width: EC_SPIRE_DEFAULT_RERANK_WIDTH,
        training_sample_rows: EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS,
        seed: EC_SPIRE_DEFAULT_SEED,
        pq_group_size: EC_SPIRE_DEFAULT_PQ_GROUP_SIZE,
        storage_format: SpireStorageFormat::Auto,
        local_store_tablespaces: None,
    };

    pub(super) fn requested_pq_group_size(&self) -> Option<usize> {
        if self.pq_group_size > 0 {
            Some(self.pq_group_size as usize)
        } else {
            None
        }
    }

    pub(super) fn assignment_payload_format(&self) -> SpireAssignmentPayloadFormat {
        self.storage_format.assignment_payload_format()
    }

    pub(super) fn recursive_fanout(&self) -> Option<u32> {
        validate_recursive_fanout_value(self.recursive_fanout)
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        match self.recursive_fanout {
            0 => None,
            value if value >= 2 => Some(value as u32),
            _ => unreachable!("recursive_fanout validation rejects value 1"),
        }
    }
}

fn validate_recursive_fanout_value(value: i32) -> Result<(), String> {
    if value == 0 || value >= 2 {
        Ok(())
    } else {
        Err("ec_spire recursive_fanout reloption must be 0 or at least 2".to_owned())
    }
}

fn validate_local_store_count_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_LOCAL_STORE_COUNT..=EC_SPIRE_MAX_LOCAL_STORE_COUNT).contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "ec_spire local_store_count reloption must be between {EC_SPIRE_MIN_LOCAL_STORE_COUNT} and {EC_SPIRE_MAX_LOCAL_STORE_COUNT}, got {value}"
        ))
    }
}

fn validate_boundary_replica_count_value(value: i32) -> Result<(), String> {
    if (EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT..=EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT).contains(&value)
    {
        Ok(())
    } else {
        Err(format!(
            "ec_spire boundary_replica_count reloption must be between {EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT} and {EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT}, got {value}"
        ))
    }
}

fn normalize_local_store_tablespaces_reloption(
    value: &str,
    local_store_count: i32,
) -> Result<String, String> {
    validate_local_store_count_value(local_store_count)?;
    let names = value.split(',').map(str::trim).collect::<Vec<_>>();
    if names.iter().any(|name| name.is_empty()) {
        return Err(
            "ec_spire local_store_tablespaces reloption must not contain empty names".to_owned(),
        );
    }
    let expected_count = usize::try_from(local_store_count)
        .map_err(|_| "ec_spire local_store_count must be non-negative".to_owned())?;
    if names.len() != expected_count {
        return Err(format!(
            "ec_spire local_store_tablespaces reloption must name exactly {expected_count} tablespace(s), got {}",
            names.len()
        ));
    }
    Ok(names.join(","))
}

pub(super) fn plan_local_store_tablespaces_with_resolver(
    local_store_count: i32,
    index_tablespace_oid: u32,
    local_store_tablespaces: Option<&str>,
    mut resolve_tablespace: impl FnMut(&str) -> Result<u32, String>,
) -> Result<Vec<SpireLocalStoreTablespacePlanEntry>, String> {
    validate_local_store_count_value(local_store_count)?;
    let store_count = usize::try_from(local_store_count)
        .map_err(|_| "ec_spire local_store_count must be non-negative".to_owned())?;
    let tablespace_oids = if let Some(local_store_tablespaces) = local_store_tablespaces {
        let normalized = normalize_local_store_tablespaces_reloption(
            local_store_tablespaces,
            local_store_count,
        )?;
        normalized
            .split(',')
            .map(&mut resolve_tablespace)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![index_tablespace_oid; store_count]
    };
    tablespace_oids
        .into_iter()
        .enumerate()
        .map(|(index, tablespace_oid)| {
            let local_store_id = u32::try_from(index)
                .map_err(|_| "ec_spire local_store_id exceeds u32".to_owned())?;
            Ok(SpireLocalStoreTablespacePlanEntry {
                local_store_id,
                tablespace_oid,
            })
        })
        .collect()
}

pub(super) unsafe fn resolve_local_store_tablespace_plan(
    index_relation: pg_sys::Relation,
    options: &EcSpireOptions,
) -> Result<Vec<SpireLocalStoreTablespacePlanEntry>, String> {
    if index_relation.is_null() {
        return Err("ec_spire local store tablespace plan needs a valid index relation".to_owned());
    }
    let index_tablespace_oid = unsafe { (*(*index_relation).rd_rel).reltablespace }.into();
    plan_local_store_tablespaces_with_resolver(
        options.local_store_count,
        index_tablespace_oid,
        options.local_store_tablespaces.as_deref(),
        |name| unsafe { resolve_tablespace_name(name) },
    )
}

unsafe fn resolve_tablespace_name(name: &str) -> Result<u32, String> {
    let c_name = CString::new(name)
        .map_err(|_| "ec_spire local_store_tablespaces cannot contain NUL bytes".to_owned())?;
    let oid = unsafe { pg_sys::get_tablespace_oid(c_name.as_ptr(), true) };
    if oid == pg_sys::InvalidOid {
        return Err(format!(
            "ec_spire local_store_tablespaces names unknown tablespace '{name}'"
        ));
    }
    Ok(oid.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireNprobeResolution {
    pub(super) relation_nprobe: u32,
    pub(super) session_nprobe: Option<u32>,
    pub(super) effective_nprobe: u32,
    pub(super) source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireRerankWidthResolution {
    pub(super) relation_rerank_width: i32,
    pub(super) session_rerank_width: Option<i32>,
    pub(super) effective_rerank_width: i32,
    pub(super) source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireCandidateDedupeMode {
    NoReplicaDedupeDisabled,
    VecIdDedupeEnabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireSingleLevelScanPlan {
    pub(super) leaf_count: u32,
    pub(super) nprobe: u32,
    pub(super) nprobe_source: &'static str,
    pub(super) payload_format: SpireAssignmentPayloadFormat,
    pub(super) rerank_width: usize,
    pub(super) rerank_width_source: &'static str,
    pub(super) candidate_limit: Option<usize>,
    pub(super) dedupe_mode: SpireCandidateDedupeMode,
}

pub(super) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ec_spire.nprobe",
        c"Session override for ec_spire leaf PID probe count.",
        c"Overrides ec_spire index nprobe reloption when set to 1 or higher; -1 uses the relation value.",
        &EC_SPIRE_NPROBE_GUC,
        EC_SPIRE_SESSION_NPROBE_UNSET,
        EC_SPIRE_MAX_NPROBE,
        GucContext::Userset,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_spire.rerank_width",
        c"Session override for ec_spire exact-rerank frontier width.",
        c"Overrides ec_spire index rerank_width reloption when set to 0 or higher; -1 uses the relation value.",
        &EC_SPIRE_RERANK_WIDTH_GUC,
        EC_SPIRE_SESSION_RERANK_WIDTH_UNSET,
        EC_SPIRE_MAX_RERANK_WIDTH,
        GucContext::Userset,
        GucFlags::default(),
    );
}

pub(super) fn current_session_nprobe() -> i32 {
    EC_SPIRE_NPROBE_GUC.get()
}

pub(super) fn current_session_rerank_width() -> i32 {
    EC_SPIRE_RERANK_WIDTH_GUC.get()
}

pub(super) fn resolve_scan_nprobe(nlists: u32, relation_nprobe: u32) -> SpireNprobeResolution {
    resolve_scan_nprobe_values(nlists, relation_nprobe, current_session_nprobe())
}

pub(super) fn resolve_scan_rerank_width(relation_rerank_width: i32) -> SpireRerankWidthResolution {
    resolve_scan_rerank_width_values(relation_rerank_width, current_session_rerank_width())
}

pub(super) fn resolve_single_level_scan_plan(
    leaf_count: u32,
    options: EcSpireOptions,
) -> Result<SpireSingleLevelScanPlan, String> {
    resolve_single_level_scan_plan_values(
        leaf_count,
        options,
        current_session_nprobe(),
        current_session_rerank_width(),
    )
}

fn resolve_single_level_scan_plan_values(
    leaf_count: u32,
    options: EcSpireOptions,
    session_nprobe_value: i32,
    session_rerank_width_value: i32,
) -> Result<SpireSingleLevelScanPlan, String> {
    let relation_nprobe = u32::try_from(options.nprobe)
        .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
    if options.rerank_width < 0 {
        return Err("ec_spire rerank_width reloption must be non-negative".to_owned());
    }

    let nprobe = resolve_scan_nprobe_values(leaf_count, relation_nprobe, session_nprobe_value);
    let rerank_width =
        resolve_scan_rerank_width_values(options.rerank_width, session_rerank_width_value);
    let rerank_width_usize = usize::try_from(rerank_width.effective_rerank_width)
        .map_err(|_| "ec_spire rerank_width exceeds usize".to_owned())?;
    let candidate_limit = if rerank_width_usize > 0 {
        Some(rerank_width_usize)
    } else {
        None
    };

    Ok(SpireSingleLevelScanPlan {
        leaf_count,
        nprobe: nprobe.effective_nprobe,
        nprobe_source: nprobe.source,
        payload_format: options.assignment_payload_format(),
        rerank_width: rerank_width_usize,
        rerank_width_source: rerank_width.source,
        candidate_limit,
        dedupe_mode: if options.boundary_replica_count > 0 {
            SpireCandidateDedupeMode::VecIdDedupeEnabled
        } else {
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        },
    })
}

fn resolve_scan_nprobe_values(
    nlists: u32,
    relation_nprobe: u32,
    session_nprobe_value: i32,
) -> SpireNprobeResolution {
    let session_nprobe = match session_nprobe_value {
        value if value > 0 => Some(value as u32),
        _ => None,
    };
    if nlists == 0 {
        return SpireNprobeResolution {
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

    SpireNprobeResolution {
        relation_nprobe,
        session_nprobe,
        effective_nprobe: requested.clamp(1, nlists),
        source,
    }
}

fn resolve_scan_rerank_width_values(
    relation_rerank_width: i32,
    session_rerank_width_value: i32,
) -> SpireRerankWidthResolution {
    let session_rerank_width = match session_rerank_width_value {
        value if value >= 0 => Some(value),
        _ => None,
    };
    let (effective_rerank_width, source) = match session_rerank_width {
        Some(value) => (value.clamp(0, EC_SPIRE_MAX_RERANK_WIDTH), "session"),
        None => (relation_rerank_width, "relation"),
    };

    SpireRerankWidthResolution {
        relation_rerank_width,
        session_rerank_width,
        effective_rerank_width,
        source,
    }
}

fn auto_nprobe(nlists: u32) -> u32 {
    if nlists == 0 {
        return 0;
    }
    (nlists as f64).sqrt().ceil() as u32
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut relopts = pg_sys::local_relopts::default();

            pg_sys::init_local_reloptions(&mut relopts, size_of::<EcSpireReloptions>());
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"nlists".as_ptr(),
                c"Number of single-level SPIRE-IVF leaf PIDs; 0 chooses an automatic value."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_NLISTS,
                EC_SPIRE_MIN_NLISTS,
                EC_SPIRE_MAX_NLISTS,
                offset_of!(EcSpireReloptions, nlists) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"recursive_fanout".as_ptr(),
                c"Opt-in recursive SPIRE routing fanout; 0 keeps single-level build behavior, values must be at least 2."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_RECURSIVE_FANOUT,
                EC_SPIRE_MIN_RECURSIVE_FANOUT,
                EC_SPIRE_MAX_RECURSIVE_FANOUT,
                offset_of!(EcSpireReloptions, recursive_fanout) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"local_store_count".as_ptr(),
                c"Number of local SPIRE partition-store relations to plan for; 1 keeps embedded single-store behavior."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_LOCAL_STORE_COUNT,
                EC_SPIRE_MIN_LOCAL_STORE_COUNT,
                EC_SPIRE_MAX_LOCAL_STORE_COUNT,
                offset_of!(EcSpireReloptions, local_store_count) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"boundary_replica_count".as_ptr(),
                c"Maximum secondary SPIRE leaf assignments per vector; 0 keeps primary-only assignment."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_BOUNDARY_REPLICA_COUNT,
                EC_SPIRE_MIN_BOUNDARY_REPLICA_COUNT,
                EC_SPIRE_MAX_BOUNDARY_REPLICA_COUNT,
                offset_of!(EcSpireReloptions, boundary_replica_count) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"nprobe".as_ptr(),
                c"Number of SPIRE leaf PIDs to probe during scan; 0 chooses an automatic value."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_NPROBE,
                EC_SPIRE_MIN_NPROBE,
                EC_SPIRE_MAX_NPROBE,
                offset_of!(EcSpireReloptions, nprobe) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"rerank_width".as_ptr(),
                c"Number of quantized candidates to exact-rerank; 0 reranks the full candidate frontier."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_RERANK_WIDTH,
                EC_SPIRE_MIN_RERANK_WIDTH,
                EC_SPIRE_MAX_RERANK_WIDTH,
                offset_of!(EcSpireReloptions, rerank_width) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"training_sample_rows".as_ptr(),
                c"Maximum rows sampled for SPIRE centroid training; 0 chooses an automatic value."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_TRAINING_SAMPLE_ROWS,
                EC_SPIRE_MIN_TRAINING_SAMPLE_ROWS,
                EC_SPIRE_MAX_TRAINING_SAMPLE_ROWS,
                offset_of!(EcSpireReloptions, training_sample_rows) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"seed".as_ptr(),
                c"Deterministic seed for SPIRE centroid training and quantizer defaults.".as_ptr(),
                EC_SPIRE_DEFAULT_SEED,
                EC_SPIRE_MIN_SEED,
                EC_SPIRE_MAX_SEED,
                offset_of!(EcSpireReloptions, seed) as i32,
            );
            pg_sys::add_local_int_reloption(
                &mut relopts,
                c"pq_group_size".as_ptr(),
                c"Grouped-PQ subvector size for storage_format = 'pq_fastscan'; 0 chooses the default."
                    .as_ptr(),
                EC_SPIRE_DEFAULT_PQ_GROUP_SIZE,
                EC_SPIRE_MIN_PQ_GROUP_SIZE,
                EC_SPIRE_MAX_PQ_GROUP_SIZE,
                offset_of!(EcSpireReloptions, pq_group_size) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"storage_format".as_ptr(),
                c"SPIRE assignment payload quantizer profile: 'turboquant', 'pq_fastscan', 'rabitq', or 'auto'."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcSpireReloptions, storage_format_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"quantizer".as_ptr(),
                c"Alias for storage_format: SPIRE assignment payload quantizer profile 'turboquant', 'pq_fastscan', 'rabitq', or 'auto'."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcSpireReloptions, quantizer_offset) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                c"local_store_tablespaces".as_ptr(),
                c"Comma-separated tablespace names for local SPIRE stores; repeated names are allowed for same-device baselines."
                    .as_ptr(),
                ptr::null(),
                None,
                None,
                offset_of!(EcSpireReloptions, local_store_tablespaces_offset) as i32,
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
        .unwrap_or_else(|e| pgrx::error!("invalid ec_spire {name} reloption: {e}"));
    if value.is_empty() {
        pgrx::error!("invalid ec_spire {name} reloption: value must not be empty");
    }
    Some(value.to_owned())
}

pub(super) unsafe fn relation_options(index_relation: pg_sys::Relation) -> EcSpireOptions {
    let rd_options = unsafe { (*index_relation).rd_options };
    if rd_options.is_null() {
        return EcSpireOptions::DEFAULT;
    }

    let reloptions = unsafe { &*rd_options.cast::<EcSpireReloptions>() };
    validate_recursive_fanout_value(reloptions.recursive_fanout)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_local_store_count_value(reloptions.local_store_count)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    validate_boundary_replica_count_value(reloptions.boundary_replica_count)
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let storage_format_reloption = unsafe {
        read_string_reloption(
            rd_options,
            reloptions.storage_format_offset,
            "storage_format",
        )
    };
    let quantizer_reloption =
        unsafe { read_string_reloption(rd_options, reloptions.quantizer_offset, "quantizer") };
    if let (Some(storage_format), Some(quantizer)) =
        (&storage_format_reloption, &quantizer_reloption)
    {
        if storage_format != quantizer {
            pgrx::error!(
                "ec_spire storage_format and quantizer reloptions conflict: storage_format = '{}', quantizer = '{}'",
                storage_format,
                quantizer
            );
        }
    }
    let storage_format = storage_format_reloption
        .or(quantizer_reloption)
        .map(|value| {
            SpireStorageFormat::parse_reloption(&value).unwrap_or_else(|e| pgrx::error!("{e}"))
        })
        .unwrap_or(SpireStorageFormat::Auto);
    let local_store_tablespaces = unsafe {
        read_string_reloption(
            rd_options,
            reloptions.local_store_tablespaces_offset,
            "local_store_tablespaces",
        )
    }
    .map(|value| {
        normalize_local_store_tablespaces_reloption(&value, reloptions.local_store_count)
            .unwrap_or_else(|e| pgrx::error!("{e}"))
    });

    EcSpireOptions {
        nlists: reloptions.nlists,
        recursive_fanout: reloptions.recursive_fanout,
        local_store_count: reloptions.local_store_count,
        boundary_replica_count: reloptions.boundary_replica_count,
        nprobe: reloptions.nprobe,
        rerank_width: reloptions.rerank_width,
        training_sample_rows: reloptions.training_sample_rows,
        seed: reloptions.seed,
        pq_group_size: reloptions.pq_group_size,
        storage_format,
        local_store_tablespaces,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_local_store_tablespaces_reloption, plan_local_store_tablespaces_with_resolver,
        resolve_scan_nprobe_values, resolve_scan_rerank_width_values,
        resolve_single_level_scan_plan_values, validate_boundary_replica_count_value,
        validate_local_store_count_value, validate_recursive_fanout_value, EcSpireOptions,
        SpireCandidateDedupeMode, SpireStorageFormat,
    };
    use crate::am::ec_spire::quantizer::SpireAssignmentPayloadFormat;

    #[test]
    fn storage_format_reloption_parses_and_maps_to_assignment_payload_format() {
        assert_eq!(
            SpireStorageFormat::parse_reloption("auto").unwrap(),
            SpireStorageFormat::Auto
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("turboquant").unwrap(),
            SpireStorageFormat::TurboQuant
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("pq_fastscan").unwrap(),
            SpireStorageFormat::PqFastScan
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("rabitq").unwrap(),
            SpireStorageFormat::RaBitQ
        );
        assert!(SpireStorageFormat::parse_reloption("bad").is_err());

        assert_eq!(
            SpireStorageFormat::Auto.assignment_payload_format(),
            SpireAssignmentPayloadFormat::TurboQuant
        );
        assert_eq!(
            SpireStorageFormat::RaBitQ.assignment_payload_format(),
            SpireAssignmentPayloadFormat::RaBitQ
        );
    }

    #[test]
    fn recursive_fanout_validation_rejects_one() {
        assert!(validate_recursive_fanout_value(0).is_ok());
        assert!(validate_recursive_fanout_value(2).is_ok());
        assert!(validate_recursive_fanout_value(32).is_ok());
        assert!(validate_recursive_fanout_value(1).is_err());
    }

    #[test]
    fn local_store_count_validation_bounds_phase4_surface() {
        assert!(validate_local_store_count_value(1).is_ok());
        assert!(validate_local_store_count_value(16).is_ok());
        assert!(validate_local_store_count_value(0).is_err());
        assert!(validate_local_store_count_value(17).is_err());
    }

    #[test]
    fn boundary_replica_count_validation_bounds_phase5_surface() {
        assert!(validate_boundary_replica_count_value(0).is_ok());
        assert!(validate_boundary_replica_count_value(8).is_ok());
        assert!(validate_boundary_replica_count_value(-1).is_err());
        assert!(validate_boundary_replica_count_value(9).is_err());
    }

    #[test]
    fn local_store_tablespaces_normalizes_and_allows_repeated_names() {
        assert_eq!(
            normalize_local_store_tablespaces_reloption("fast_a, fast_a", 2).unwrap(),
            "fast_a,fast_a"
        );
        assert_eq!(
            normalize_local_store_tablespaces_reloption("fast_a", 1).unwrap(),
            "fast_a"
        );
        assert!(normalize_local_store_tablespaces_reloption("fast_a", 2).is_err());
        assert!(normalize_local_store_tablespaces_reloption("fast_a,", 2).is_err());
    }

    #[test]
    fn local_store_tablespace_plan_resolves_names_and_repeats() {
        let plan = plan_local_store_tablespaces_with_resolver(
            3,
            999,
            Some("fast_a,fast_a,fast_b"),
            |name| match name {
                "fast_a" => Ok(10),
                "fast_b" => Ok(11),
                other => Err(format!("unknown tablespace {other}")),
            },
        )
        .unwrap();

        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].local_store_id, 0);
        assert_eq!(plan[0].tablespace_oid, 10);
        assert_eq!(plan[1].local_store_id, 1);
        assert_eq!(plan[1].tablespace_oid, 10);
        assert_eq!(plan[2].local_store_id, 2);
        assert_eq!(plan[2].tablespace_oid, 11);
    }

    #[test]
    fn local_store_tablespace_plan_inherits_index_tablespace_by_default() {
        let plan =
            plan_local_store_tablespaces_with_resolver(2, 999, None, |_| unreachable!()).unwrap();

        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].local_store_id, 0);
        assert_eq!(plan[0].tablespace_oid, 999);
        assert_eq!(plan[1].local_store_id, 1);
        assert_eq!(plan[1].tablespace_oid, 999);
    }

    #[test]
    fn local_store_tablespace_plan_rejects_unknown_or_mismatched_names() {
        assert!(
            plan_local_store_tablespaces_with_resolver(2, 999, Some("fast_a"), |_| Ok(10)).is_err()
        );
        assert!(
            plan_local_store_tablespaces_with_resolver(1, 999, Some("missing"), |name| Err(
                format!("unknown tablespace {name}")
            ),)
            .is_err()
        );
    }

    #[test]
    fn scan_nprobe_resolution_uses_session_relation_and_auto_sources() {
        assert_eq!(resolve_scan_nprobe_values(0, 5, -1).effective_nprobe, 0);

        let auto = resolve_scan_nprobe_values(17, 0, -1);
        assert_eq!(auto.effective_nprobe, 5);
        assert_eq!(auto.source, "auto");

        let relation = resolve_scan_nprobe_values(17, 3, -1);
        assert_eq!(relation.effective_nprobe, 3);
        assert_eq!(relation.source, "relation");

        let session = resolve_scan_nprobe_values(17, 3, 99);
        assert_eq!(session.session_nprobe, Some(99));
        assert_eq!(session.effective_nprobe, 17);
        assert_eq!(session.source, "session");
    }

    #[test]
    fn scan_rerank_width_resolution_uses_session_or_relation() {
        let relation = resolve_scan_rerank_width_values(128, -1);
        assert_eq!(relation.effective_rerank_width, 128);
        assert_eq!(relation.source, "relation");

        let session = resolve_scan_rerank_width_values(128, 0);
        assert_eq!(session.session_rerank_width, Some(0));
        assert_eq!(session.effective_rerank_width, 0);
        assert_eq!(session.source, "session");
    }

    #[test]
    fn default_options_match_phase1_config_contract() {
        let options = EcSpireOptions::DEFAULT;

        assert_eq!(options.nlists, 0);
        assert_eq!(options.recursive_fanout, 0);
        assert_eq!(options.recursive_fanout(), None);
        assert_eq!(options.local_store_count, 1);
        assert_eq!(options.boundary_replica_count, 0);
        assert_eq!(options.nprobe, 0);
        assert_eq!(options.rerank_width, 0);
        assert_eq!(options.training_sample_rows, 0);
        assert_eq!(options.seed, 42);
        assert_eq!(options.requested_pq_group_size(), None);
        assert_eq!(options.storage_format, SpireStorageFormat::Auto);
        assert_eq!(options.local_store_tablespaces, None);
        assert_eq!(
            options.assignment_payload_format(),
            SpireAssignmentPayloadFormat::TurboQuant
        );
    }

    #[test]
    fn single_level_scan_plan_resolves_runtime_knobs() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 4,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: 3,
            rerank_width: 128,
            training_sample_rows: 1000,
            seed: 7,
            pq_group_size: 0,
            storage_format: SpireStorageFormat::RaBitQ,
            local_store_tablespaces: Some("fast_a".to_owned()),
        };

        let plan = resolve_single_level_scan_plan_values(17, options.clone(), -1, -1).unwrap();

        assert_eq!(plan.leaf_count, 17);
        assert_eq!(plan.nprobe, 3);
        assert_eq!(plan.nprobe_source, "relation");
        assert_eq!(plan.payload_format, SpireAssignmentPayloadFormat::RaBitQ);
        assert_eq!(plan.rerank_width, 128);
        assert_eq!(plan.rerank_width_source, "relation");
        assert_eq!(plan.candidate_limit, Some(128));
        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        );
        assert_eq!(options.recursive_fanout(), Some(4));
    }

    #[test]
    fn single_level_scan_plan_uses_session_overrides_and_full_rerank() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: 0,
            rerank_width: 128,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            storage_format: SpireStorageFormat::Auto,
            local_store_tablespaces: None,
        };

        let plan = resolve_single_level_scan_plan_values(17, options, 99, 0).unwrap();

        assert_eq!(plan.nprobe, 17);
        assert_eq!(plan.nprobe_source, "session");
        assert_eq!(
            plan.payload_format,
            SpireAssignmentPayloadFormat::TurboQuant
        );
        assert_eq!(plan.rerank_width, 0);
        assert_eq!(plan.rerank_width_source, "session");
        assert_eq!(plan.candidate_limit, None);
        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        );
    }

    #[test]
    fn single_level_scan_plan_rejects_invalid_manual_options() {
        let invalid = EcSpireOptions {
            nlists: 0,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: -1,
            rerank_width: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            storage_format: SpireStorageFormat::Auto,
            local_store_tablespaces: None,
        };
        assert!(resolve_single_level_scan_plan_values(1, invalid.clone(), -1, -1).is_err());

        let invalid = EcSpireOptions {
            nprobe: 0,
            rerank_width: -1,
            ..invalid
        };
        assert!(resolve_single_level_scan_plan_values(1, invalid, -1, -1).is_err());
    }

    #[test]
    fn single_level_scan_plan_enables_vec_id_dedupe_for_replica_capable_indexes() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 1,
            nprobe: 3,
            rerank_width: 128,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            storage_format: SpireStorageFormat::Auto,
            local_store_tablespaces: None,
        };

        let plan = resolve_single_level_scan_plan_values(17, options, -1, -1).unwrap();

        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::VecIdDedupeEnabled
        );
    }
}
