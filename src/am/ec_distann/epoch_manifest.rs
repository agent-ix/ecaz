//! FR-082 epoch lifecycle: Building → Published → Retired, with a retention gate
//! (Task 165 M3, AC-1 / AC-3 / AC-6).
//!
//! The lifecycle state is persisted in the metadata page (format v4:
//! `epoch_state`, `active_epoch`, `in_flight_count`), so a Published epoch, its
//! number, and the in-flight retention count survive a restart. This is the
//! single-index realization of the epoch manifest: the build IS the publish; a
//! republish swaps the active epoch atomically (one metadata-page write); retire
//! reclaims only after in-flight reaches zero, with a logged operator override
//! for a wedged count (a crashed coordinator that never decremented).
//!
//! See `reviews/task-165/014-fr082-lifecycle-design.md` for the full mapping to
//! the six FR-082 sub-ACs. Per-scan in-flight increment/decrement (the counter
//! that feeds the gate in production) and the epoch-swap consistency drills are
//! the next slices; this module lands the durable state machine + endpoints.

use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern, pg_sys};
use std::ptr::NonNull;

use crate::storage::relation_guard::IndexRelationGuard;

use super::ambuild::{overwrite_metadata_page_handle, read_metadata_from_index_handle};
use super::page::{
    DistannMetadataPage, DISTANN_EPOCH_STATE_BUILDING, DISTANN_EPOCH_STATE_PUBLISHED,
    DISTANN_EPOCH_STATE_RETIRED,
};

fn epoch_state_name(state: u8) -> &'static str {
    match state {
        DISTANN_EPOCH_STATE_BUILDING => "building",
        DISTANN_EPOCH_STATE_PUBLISHED => "published",
        DISTANN_EPOCH_STATE_RETIRED => "retired",
        _ => "unknown",
    }
}

/// Open the index under `RowExclusiveLock` (serializes concurrent lifecycle
/// mutations against each other while allowing `AccessShareLock` scans), read the
/// metadata, mutate it, and write it back atomically (one WAL-logged page write).
fn with_metadata_mut<T>(
    index_oid: pg_sys::Oid,
    caller: &'static str,
    mutate: impl FnOnce(&mut DistannMetadataPage) -> Result<T, String>,
) -> Result<T, String> {
    let guard = IndexRelationGuard::open(
        index_oid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        caller,
    );
    let handle = NonNull::new(guard.as_ptr())
        .ok_or_else(|| format!("{caller} got a null index relation"))?;
    let mut metadata = read_metadata_from_index_handle(handle)?;
    let result = mutate(&mut metadata)?;
    overwrite_metadata_page_handle(handle, &metadata);
    Ok(result)
}

fn read_metadata(index_oid: pg_sys::Oid, caller: &'static str) -> Result<DistannMetadataPage, String> {
    let guard = IndexRelationGuard::open(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        caller,
    );
    let handle = NonNull::new(guard.as_ptr())
        .ok_or_else(|| format!("{caller} got a null index relation"))?;
    read_metadata_from_index_handle(handle)
}

/// FR-082-AC-1: publish `epoch` as the active Published graph. Atomic per node
/// (a single metadata-page write); a republish swaps the active epoch and resets
/// the in-flight retention count for the new epoch.
#[pg_extern(volatile)]
fn ec_distann_publish_epoch(index_regclass: pg_sys::Oid, epoch: i64) {
    let epoch = u64::try_from(epoch)
        .unwrap_or_else(|_| pgrx::error!("ec_distann_publish_epoch: epoch must be non-negative"));
    with_metadata_mut(index_regclass, "ec_distann_publish_epoch", |metadata| {
        metadata.epoch_state = DISTANN_EPOCH_STATE_PUBLISHED;
        metadata.active_epoch = epoch;
        metadata.in_flight_count = 0;
        Ok(())
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
}

/// FR-082-AC-3: retire the active epoch, but only once its in-flight query count
/// reaches zero. A non-zero count is the retention gate — the call errors and the
/// epoch stays Published (its storage is retained).
#[pg_extern(volatile)]
fn ec_distann_retire_epoch(index_regclass: pg_sys::Oid) {
    with_metadata_mut(index_regclass, "ec_distann_retire_epoch", |metadata| {
        if metadata.epoch_state == DISTANN_EPOCH_STATE_RETIRED {
            return Ok(());
        }
        if metadata.in_flight_count != 0 {
            return Err(format!(
                "ec_distann_retire_epoch: retention gate — {} scan(s) in flight against epoch {}; \
                 retire is blocked until they drain (or use ec_distann_force_retire_epoch)",
                metadata.in_flight_count, metadata.active_epoch
            ));
        }
        metadata.epoch_state = DISTANN_EPOCH_STATE_RETIRED;
        Ok(())
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
}

/// FR-082-AC-6: operator override for a wedged in-flight count (e.g. a crashed
/// coordinator that never decremented). Force-retires regardless of the count,
/// clears it, and logs a WARNING so the override is auditable.
#[pg_extern(volatile)]
fn ec_distann_force_retire_epoch(index_regclass: pg_sys::Oid) {
    with_metadata_mut(index_regclass, "ec_distann_force_retire_epoch", |metadata| {
        pgrx::warning!(
            "ec_distann operator force-retire of epoch {} with {} in-flight scan(s) (wedged count override)",
            metadata.active_epoch,
            metadata.in_flight_count
        );
        metadata.epoch_state = DISTANN_EPOCH_STATE_RETIRED;
        metadata.in_flight_count = 0;
        Ok(())
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
}

/// Read the persisted epoch lifecycle state.
#[pg_extern(stable)]
fn ec_distann_epoch_status(
    index_regclass: pg_sys::Oid,
) -> TableIterator<
    'static,
    (
        name!(epoch_state, String),
        name!(active_epoch, i64),
        name!(in_flight_count, i64),
    ),
> {
    let metadata =
        read_metadata(index_regclass, "ec_distann_epoch_status").unwrap_or_else(|e| pgrx::error!("{e}"));
    TableIterator::new(std::iter::once((
        epoch_state_name(metadata.epoch_state).to_owned(),
        metadata.active_epoch as i64,
        i64::from(metadata.in_flight_count),
    )))
}

/// Debug/test helper: set the in-flight retention count directly so the retention
/// gate (AC-3) and override (AC-6) can be drilled deterministically before the
/// per-scan increment/decrement wiring lands.
#[pg_extern(volatile)]
fn ec_distann_debug_set_in_flight(index_regclass: pg_sys::Oid, count: i64) {
    let count = u32::try_from(count)
        .unwrap_or_else(|_| pgrx::error!("ec_distann_debug_set_in_flight: count out of range"));
    with_metadata_mut(index_regclass, "ec_distann_debug_set_in_flight", |metadata| {
        metadata.in_flight_count = count;
        Ok(())
    })
    .unwrap_or_else(|e| pgrx::error!("{e}"));
}
