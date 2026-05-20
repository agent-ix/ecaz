use std::ffi::{c_int, c_void};
use std::mem::size_of;
use std::ptr::{addr_of, addr_of_mut};
use std::sync::atomic::{AtomicU32, Ordering};

use pgrx::pg_sys;

use super::parallel_slot::{
    load_worker_slot_snapshot, publish_worker_slot_runtime_snapshot, release_worker_slot,
    try_claim_worker_slot, EcParallelWorkerSlotFields,
};
pub(crate) use super::parallel_slot::{
    EcParallelWorkerSlotRuntimeSnapshot, EcParallelWorkerSlotSnapshot,
    EC_PARALLEL_WORKER_PHASE_EXHAUSTED, EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL,
    EC_PARALLEL_WORKER_PHASE_IDLE, EC_PARALLEL_WORKER_PHASE_LINEAR_FALLBACK,
    EC_PARALLEL_WORKER_SLOT_CLAIMED, EC_PARALLEL_WORKER_SLOT_FREE,
};

const EC_PARALLEL_SCAN_STATE_MAGIC: u32 = u32::from_le_bytes(*b"ECPR");
const EC_PARALLEL_SCAN_STATE_VERSION: u16 = 2;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct EcParallelScanState {
    magic: u32,
    version: u16,
    flags: u16,
    descriptor_bytes: pg_sys::Size,
    coordinator_bytes: pg_sys::Size,
    worker_slot_bytes: pg_sys::Size,
    worker_slot_count: u32,
    reserved_worker_slots: u32,
    rescan_epoch: u32,
    reserved0: u32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct EcParallelCoordinatorState {
    pub(crate) flags: AtomicU32,
    pub(crate) claimed_worker_slots: AtomicU32,
    reserved0: u32,
    reserved1: u32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct EcParallelWorkerSlot {
    pub(crate) flags: AtomicU32,
    pub(crate) slot_index: u32,
    pub(crate) observed_rescan_epoch: AtomicU32,
    execution_phase: AtomicU32,
    scan_dimensions: AtomicU32,
    bootstrap_frontier_limit: AtomicU32,
    visible_frontier_len: AtomicU32,
    scheduler_frontier_len: AtomicU32,
    visited_count: AtomicU32,
    emitted_count: AtomicU32,
    active_result_pending_count: AtomicU32,
    active_result_has_current: AtomicU32,
    reserved0: u32,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ParallelScanAttachment {
    pub(crate) state: *mut EcParallelScanState,
    pub(crate) coordinator: *mut EcParallelCoordinatorState,
    worker_slots: *mut EcParallelWorkerSlot,
    pub(crate) descriptor_bytes: pg_sys::Size,
    pub(crate) worker_slot_count: u32,
    worker_slot_bytes: pg_sys::Size,
    pub(crate) rescan_epoch: u32,
}

impl ParallelScanAttachment {
    pub(crate) fn worker_slot(
        &self,
        slot_index: u32,
    ) -> Result<*mut EcParallelWorkerSlot, &'static str> {
        if slot_index >= self.worker_slot_count {
            return Err("parallel worker slot index was outside the descriptor capacity");
        }
        let offset = checked_mul_size(
            self.worker_slot_bytes,
            slot_index as pg_sys::Size,
            "parallel worker slot offset",
        );
        // SAFETY: validated attachments seed `worker_slots` from a descriptor
        // whose slot count and stride were bounds-checked above.
        Ok(unsafe { self.worker_slots.cast::<u8>().add(offset) }.cast())
    }
}

fn maxalign(size: pg_sys::Size) -> pg_sys::Size {
    let align = size_of::<usize>();
    debug_assert!(align.is_power_of_two());
    (size + align - 1) & !(align - 1)
}

fn checked_add_size(lhs: pg_sys::Size, rhs: pg_sys::Size, context: &str) -> pg_sys::Size {
    lhs.checked_add(rhs)
        .unwrap_or_else(|| panic!("{context} overflowed pg_sys::Size"))
}

fn checked_mul_size(lhs: pg_sys::Size, rhs: pg_sys::Size, context: &str) -> pg_sys::Size {
    lhs.checked_mul(rhs)
        .unwrap_or_else(|| panic!("{context} overflowed pg_sys::Size"))
}

pub(crate) fn ec_parallel_scan_state_size() -> pg_sys::Size {
    maxalign(size_of::<EcParallelScanState>())
}

pub(crate) fn ec_parallel_scan_coordinator_size() -> pg_sys::Size {
    maxalign(size_of::<EcParallelCoordinatorState>())
}

pub(crate) fn ec_parallel_scan_worker_slot_size() -> pg_sys::Size {
    maxalign(size_of::<EcParallelWorkerSlot>())
}

fn ec_parallel_scan_descriptor_size_for(worker_slot_count: u32) -> pg_sys::Size {
    let worker_slot_bytes = checked_mul_size(
        ec_parallel_scan_worker_slot_size(),
        worker_slot_count as pg_sys::Size,
        "parallel worker slot descriptor size",
    );
    maxalign(checked_add_size(
        checked_add_size(
            ec_parallel_scan_state_size(),
            ec_parallel_scan_coordinator_size(),
            "parallel scan state plus coordinator size",
        ),
        worker_slot_bytes,
        "parallel scan descriptor size",
    ))
}

pub(crate) fn ec_parallel_scan_worker_slot_capacity() -> u32 {
    // SAFETY: PostgreSQL exposes this process-wide GUC as a backend global;
    // reading it is valid while executing inside a PostgreSQL backend.
    let max_workers = unsafe { pg_sys::max_parallel_workers_per_gather }.max(0) as u32;
    max_workers.saturating_add(1)
}

pub(crate) fn ec_parallel_scan_descriptor_size() -> pg_sys::Size {
    ec_parallel_scan_descriptor_size_for(ec_parallel_scan_worker_slot_capacity())
}

fn coordinator_ptr(state: *mut EcParallelScanState) -> *mut EcParallelCoordinatorState {
    // SAFETY: callers pass the start of the AM-private descriptor; the
    // coordinator immediately follows the MAXALIGN-sized state header.
    unsafe { state.cast::<u8>().add(ec_parallel_scan_state_size()) }.cast()
}

fn worker_slots_ptr(state: *mut EcParallelScanState) -> *mut EcParallelWorkerSlot {
    let coordinator_offset = checked_add_size(
        ec_parallel_scan_state_size(),
        // SAFETY: callers only use initialized/validated descriptors, so the
        // header field is readable before deriving the slot-array offset.
        unsafe { (*state).coordinator_bytes },
        "parallel worker slot base offset",
    );
    // SAFETY: the worker slot array starts after the state header plus the
    // recorded coordinator span, both checked for overflow above.
    unsafe { state.cast::<u8>().add(coordinator_offset) }.cast()
}

fn reset_parallel_scan_layout(state: *mut EcParallelScanState) {
    // SAFETY: callers pass an initialized descriptor header; use raw reads so
    // resetting the following shared-memory layout does not move header fields.
    let (worker_slot_count, worker_slot_bytes, rescan_epoch) = unsafe {
        (
            addr_of!((*state).worker_slot_count).read(),
            addr_of!((*state).worker_slot_bytes).read(),
            addr_of!((*state).rescan_epoch).read(),
        )
    };
    // SAFETY: `state` points at the writable AM-private descriptor header.
    unsafe { addr_of_mut!((*state).reserved_worker_slots).write(0) };

    // SAFETY: the coordinator region is within the descriptor immediately
    // after the initialized state header.
    unsafe {
        *coordinator_ptr(state) = EcParallelCoordinatorState {
            flags: AtomicU32::new(0),
            claimed_worker_slots: AtomicU32::new(0),
            reserved0: 0,
            reserved1: 0,
        };
    }

    for slot_index in 0..worker_slot_count {
        // SAFETY: slot offsets are derived from the descriptor stride and the
        // in-range loop index, so each write targets one allocated slot.
        let slot = unsafe {
            worker_slots_ptr(state)
                .cast::<u8>()
                .add(checked_mul_size(
                    worker_slot_bytes,
                    slot_index as pg_sys::Size,
                    "parallel worker slot reset offset",
                ))
                .cast::<EcParallelWorkerSlot>()
        };
        // SAFETY: `slot` points at a writable slot in the AM-private shared
        // descriptor and is initialized exactly once during this reset pass.
        unsafe {
            *slot = EcParallelWorkerSlot {
                flags: AtomicU32::new(EC_PARALLEL_WORKER_SLOT_FREE),
                slot_index,
                observed_rescan_epoch: AtomicU32::new(rescan_epoch),
                execution_phase: AtomicU32::new(EC_PARALLEL_WORKER_PHASE_IDLE),
                scan_dimensions: AtomicU32::new(0),
                bootstrap_frontier_limit: AtomicU32::new(0),
                visible_frontier_len: AtomicU32::new(0),
                scheduler_frontier_len: AtomicU32::new(0),
                visited_count: AtomicU32::new(0),
                emitted_count: AtomicU32::new(0),
                active_result_pending_count: AtomicU32::new(0),
                active_result_has_current: AtomicU32::new(0),
                reserved0: 0,
            };
        }
    }
}

fn worker_slot_fields(slot: &EcParallelWorkerSlot) -> EcParallelWorkerSlotFields<'_, AtomicU32> {
    EcParallelWorkerSlotFields {
        flags: &slot.flags,
        slot_index: slot.slot_index,
        observed_rescan_epoch: &slot.observed_rescan_epoch,
        execution_phase: &slot.execution_phase,
        scan_dimensions: &slot.scan_dimensions,
        bootstrap_frontier_limit: &slot.bootstrap_frontier_limit,
        visible_frontier_len: &slot.visible_frontier_len,
        scheduler_frontier_len: &slot.scheduler_frontier_len,
        visited_count: &slot.visited_count,
        emitted_count: &slot.emitted_count,
        active_result_pending_count: &slot.active_result_pending_count,
        active_result_has_current: &slot.active_result_has_current,
    }
}

fn initialize_parallel_scan_state(state: *mut EcParallelScanState, worker_slot_count: u32) {
    // SAFETY: PostgreSQL gave the AM a writable descriptor buffer of at least
    // `ec_parallel_scan_descriptor_size_for(worker_slot_count)` bytes.
    unsafe {
        state.write(EcParallelScanState {
            magic: EC_PARALLEL_SCAN_STATE_MAGIC,
            version: EC_PARALLEL_SCAN_STATE_VERSION,
            flags: 0,
            descriptor_bytes: ec_parallel_scan_descriptor_size_for(worker_slot_count),
            coordinator_bytes: ec_parallel_scan_coordinator_size(),
            worker_slot_bytes: ec_parallel_scan_worker_slot_size(),
            worker_slot_count,
            reserved_worker_slots: 0,
            rescan_epoch: 0,
            reserved0: 0,
        });
        reset_parallel_scan_layout(state);
    }
}

fn validate_parallel_scan_state(
    state: *mut EcParallelScanState,
) -> Result<ParallelScanAttachment, &'static str> {
    if state.is_null() {
        return Err("AM-private parallel scan state pointer was null");
    }

    // SAFETY: non-null state pointers are expected to reference the AM-private
    // shared descriptor header; magic/version checks below reject stale memory.
    let state_ref = unsafe { &*state };
    if state_ref.magic != EC_PARALLEL_SCAN_STATE_MAGIC {
        return Err("AM-private parallel scan state magic was not initialized");
    }
    if state_ref.version != EC_PARALLEL_SCAN_STATE_VERSION {
        return Err("AM-private parallel scan state version was not initialized");
    }
    if state_ref.coordinator_bytes < ec_parallel_scan_coordinator_size() {
        return Err("AM-private parallel scan coordinator size was smaller than the shared header");
    }
    if state_ref.worker_slot_bytes < ec_parallel_scan_worker_slot_size() {
        return Err("AM-private parallel worker slot size was smaller than the shared header");
    }
    let minimum_descriptor_bytes =
        ec_parallel_scan_descriptor_size_for(state_ref.worker_slot_count);
    if state_ref.descriptor_bytes < minimum_descriptor_bytes {
        return Err("AM-private parallel scan descriptor size was smaller than the shared layout");
    }

    Ok(ParallelScanAttachment {
        state,
        coordinator: coordinator_ptr(state),
        worker_slots: worker_slots_ptr(state),
        descriptor_bytes: state_ref.descriptor_bytes,
        worker_slot_count: state_ref.worker_slot_count,
        worker_slot_bytes: state_ref.worker_slot_bytes,
        rescan_epoch: state_ref.rescan_epoch,
    })
}

#[cfg(feature = "pg17")]
fn parallel_scan_state_ptr(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<*mut EcParallelScanState>, &'static str> {
    if parallel_scan.is_null() {
        return Ok(None);
    }
    // SAFETY: PostgreSQL owns `parallel_scan`; pg17 stores `ps_offset` as a
    // byte offset from the descriptor base to the AM-private state.
    let state = unsafe {
        let offset = (*parallel_scan).ps_offset;
        if offset == 0 {
            return Ok(None);
        }
        parallel_scan
            .cast::<u8>()
            .add(offset)
            .cast::<EcParallelScanState>()
    };
    Ok(Some(state))
}

#[cfg(feature = "pg18")]
fn parallel_scan_state_ptr(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<*mut EcParallelScanState>, &'static str> {
    if parallel_scan.is_null() {
        return Ok(None);
    }
    // SAFETY: PostgreSQL owns `parallel_scan`; pg18 stores `ps_offset_am` as a
    // byte offset from the descriptor base to the AM-private state.
    let state = unsafe {
        let offset = (*parallel_scan).ps_offset_am;
        if offset == 0 {
            return Ok(None);
        }
        parallel_scan
            .cast::<u8>()
            .add(offset)
            .cast::<EcParallelScanState>()
    };
    Ok(Some(state))
}

pub(crate) unsafe fn parallel_scan_attachment(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<ParallelScanAttachment>, &'static str> {
    let Some(state) = parallel_scan_state_ptr(parallel_scan)? else {
        return Ok(None);
    };
    Ok(Some(validate_parallel_scan_state(state)?))
}

pub(crate) unsafe fn initialize_parallel_scan_target_with_worker_slots(
    target: *mut c_void,
    worker_slot_count: u32,
) -> Result<(), &'static str> {
    if target.is_null() {
        return Err("AM-private parallel scan target was null");
    }
    initialize_parallel_scan_state(target.cast::<EcParallelScanState>(), worker_slot_count);
    Ok(())
}

pub(crate) unsafe fn initialize_parallel_scan_target(
    target: *mut c_void,
) -> Result<(), &'static str> {
    // SAFETY: delegates to the checked initializer with the capacity derived
    // from PostgreSQL's configured worker limit.
    unsafe {
        initialize_parallel_scan_target_with_worker_slots(
            target,
            ec_parallel_scan_worker_slot_capacity(),
        )
    }
}

pub(crate) unsafe fn claim_parallel_scan_worker_slot(
    attachment: &ParallelScanAttachment,
) -> Result<u32, &'static str> {
    for slot_index in 0..attachment.worker_slot_count {
        let slot = attachment.worker_slot(slot_index)?;
        // SAFETY: `worker_slot` returns a slot pointer within the validated
        // descriptor for this bounded index.
        let slot_ref = unsafe { &*slot };
        if try_claim_worker_slot(worker_slot_fields(slot_ref), attachment.rescan_epoch) {
            // SAFETY: a valid attachment always carries a coordinator pointer
            // derived from the same shared descriptor.
            unsafe { &*attachment.coordinator }
                .claimed_worker_slots
                .fetch_add(1, Ordering::AcqRel);
            return Ok(slot_index);
        }
    }

    Err("parallel worker slot capacity was exhausted")
}

pub(crate) unsafe fn release_parallel_scan_worker_slot(
    state: *mut EcParallelScanState,
    slot_index: u32,
    rescan_epoch: u32,
) -> Result<bool, &'static str> {
    let attachment = validate_parallel_scan_state(state)?;
    let slot = attachment.worker_slot(slot_index)?;
    // SAFETY: a successful slot lookup returns a pointer within the validated
    // shared descriptor.
    let slot_ref = unsafe { &*slot };
    if release_worker_slot(worker_slot_fields(slot_ref), rescan_epoch) {
        // SAFETY: the coordinator pointer belongs to the same validated
        // descriptor as the released worker slot.
        unsafe { &*attachment.coordinator }
            .claimed_worker_slots
            .fetch_sub(1, Ordering::AcqRel);
        return Ok(true);
    }

    Ok(false)
}

pub(crate) unsafe fn publish_parallel_scan_worker_slot_runtime_snapshot(
    state: *mut EcParallelScanState,
    slot_index: u32,
    rescan_epoch: u32,
    snapshot: EcParallelWorkerSlotRuntimeSnapshot,
) -> Result<bool, &'static str> {
    let attachment = validate_parallel_scan_state(state)?;
    let slot = attachment.worker_slot(slot_index)?;
    // SAFETY: a successful slot lookup returns a pointer within the validated
    // shared descriptor.
    let slot_ref = unsafe { &*slot };
    Ok(publish_worker_slot_runtime_snapshot(
        worker_slot_fields(slot_ref),
        rescan_epoch,
        snapshot,
    ))
}

pub(crate) unsafe fn read_parallel_scan_worker_slot_snapshot(
    state: *mut EcParallelScanState,
    slot_index: u32,
) -> Result<EcParallelWorkerSlotSnapshot, &'static str> {
    let attachment = validate_parallel_scan_state(state)?;
    let slot = attachment.worker_slot(slot_index)?;
    // SAFETY: a successful slot lookup returns a pointer within the validated
    // shared descriptor.
    Ok(load_worker_slot_snapshot(worker_slot_fields(unsafe {
        &*slot
    })))
}

pub(crate) unsafe fn reset_parallel_scan_state(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<u32>, &'static str> {
    let Some(state) = parallel_scan_state_ptr(parallel_scan)? else {
        return Ok(None);
    };
    let rescan_epoch = {
        // SAFETY: the pointer came from PostgreSQL's AM-private offset; the
        // header is checked before the rescan epoch is mutated.
        let state_ref = unsafe { &mut *state };
        if state_ref.magic != EC_PARALLEL_SCAN_STATE_MAGIC
            || state_ref.version != EC_PARALLEL_SCAN_STATE_VERSION
        {
            return Err("AM-private parallel scan state was not initialized before rescan");
        }
        state_ref.rescan_epoch = state_ref.rescan_epoch.wrapping_add(1);
        state_ref.rescan_epoch
    };
    reset_parallel_scan_layout(state);
    Ok(Some(rescan_epoch))
}

#[cfg(feature = "pg17")]
pub(crate) unsafe extern "C-unwind" fn ec_amestimateparallelscan(
    _nkeys: c_int,
    _norderbys: c_int,
) -> pg_sys::Size {
    // SAFETY: pgrx guard converts Rust panics into PostgreSQL errors at the C
    // callback boundary; no raw PostgreSQL pointers are dereferenced here.
    unsafe { pgrx::pgrx_extern_c_guard(ec_parallel_scan_descriptor_size) }
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_amestimateparallelscan(
    _index_relation: pg_sys::Relation,
    _nkeys: c_int,
    _norderbys: c_int,
) -> pg_sys::Size {
    // SAFETY: pgrx guard converts Rust panics into PostgreSQL errors at the C
    // callback boundary; no raw PostgreSQL pointers are dereferenced here.
    unsafe { pgrx::pgrx_extern_c_guard(ec_parallel_scan_descriptor_size) }
}

pub(crate) unsafe extern "C-unwind" fn ec_aminitparallelscan(target: *mut c_void) {
    // SAFETY: pgrx guard protects the PostgreSQL C callback boundary; the
    // initializer validates the target pointer before writing AM state.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            initialize_parallel_scan_target(target)
                .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel scan init failed: {err}"));
        })
    }
}

pub(crate) unsafe extern "C-unwind" fn ec_amparallelrescan(scan: pg_sys::IndexScanDesc) {
    // SAFETY: pgrx guard protects the PostgreSQL C callback boundary; `scan`
    // is checked for null before accessing PostgreSQL's parallel-scan pointer.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }
            reset_parallel_scan_state((*scan).parallel_scan)
                .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel scan rescan failed: {err}"));
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::ptr;
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;

    #[derive(Copy, Clone)]
    struct SharedParallelScanState(*mut EcParallelScanState);

    // SAFETY: tests create this pointer from stack storage that remains alive
    // for the entire scoped-thread join, and shared access goes through atomics.
    unsafe impl Send for SharedParallelScanState {}
    // SAFETY: the pointed-at descriptor is test-owned for the duration of the
    // scoped threads, and concurrent fields use atomic slot operations.
    unsafe impl Sync for SharedParallelScanState {}

    impl SharedParallelScanState {
        fn attachment(self) -> ParallelScanAttachment {
            validate_parallel_scan_state(self.0)
                .expect("shared parallel scan state should validate")
        }
    }

    fn worker_slot_header_snapshot(slot: &EcParallelWorkerSlot) -> (u32, u32, u32) {
        (
            slot.flags.load(Ordering::Acquire),
            slot.slot_index,
            slot.observed_rescan_epoch.load(Ordering::Acquire),
        )
    }

    #[repr(C, align(8))]
    struct TestParallelScanStorage {
        bytes: [u8; 1024],
    }

    impl Default for TestParallelScanStorage {
        fn default() -> Self {
            Self { bytes: [0; 1024] }
        }
    }

    const TEST_PARALLEL_SCAN_OFFSET: usize = 64;
    const TEST_WORKER_SLOT_COUNT: u32 = 3;

    fn test_parallel_scan_desc(
        storage: &mut TestParallelScanStorage,
    ) -> pg_sys::ParallelIndexScanDesc {
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        {
            // SAFETY: `parallel_scan` points into aligned test storage large
            // enough for PostgreSQL's parallel scan descriptor header.
            unsafe { (*parallel_scan).ps_offset = TEST_PARALLEL_SCAN_OFFSET };
        }
        #[cfg(feature = "pg18")]
        {
            // SAFETY: `parallel_scan` points into aligned test storage large
            // enough for PostgreSQL's parallel scan descriptor header.
            unsafe { (*parallel_scan).ps_offset_am = TEST_PARALLEL_SCAN_OFFSET };
        }
        parallel_scan
    }

    fn test_parallel_scan_desc_and_target(
        storage: &mut TestParallelScanStorage,
    ) -> (pg_sys::ParallelIndexScanDesc, *mut c_void) {
        let base = storage.bytes.as_mut_ptr();
        let parallel_scan = base.cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        {
            // SAFETY: `parallel_scan` points into aligned test storage large
            // enough for PostgreSQL's parallel scan descriptor header.
            unsafe { (*parallel_scan).ps_offset = TEST_PARALLEL_SCAN_OFFSET };
        }
        #[cfg(feature = "pg18")]
        {
            // SAFETY: `parallel_scan` points into aligned test storage large
            // enough for PostgreSQL's parallel scan descriptor header.
            unsafe { (*parallel_scan).ps_offset_am = TEST_PARALLEL_SCAN_OFFSET };
        }
        // SAFETY: the fixed test offset stays within `TestParallelScanStorage`
        // and leaves enough space for the AM-private descriptor under test.
        let target = unsafe { base.add(TEST_PARALLEL_SCAN_OFFSET) }.cast::<c_void>();
        (parallel_scan, target)
    }

    fn test_parallel_scan_target(storage: &mut TestParallelScanStorage) -> *mut c_void {
        // SAFETY: the fixed test offset stays within `TestParallelScanStorage`
        // and leaves enough space for the AM-private descriptor under test.
        unsafe { storage.bytes.as_mut_ptr().add(TEST_PARALLEL_SCAN_OFFSET) }.cast::<c_void>()
    }

    fn initialize_test_parallel_scan_target(target: *mut c_void) {
        // SAFETY: test targets come from `TestParallelScanStorage`, whose fixed
        // AM-private offset reserves enough bytes for `TEST_WORKER_SLOT_COUNT`.
        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
    }

    fn test_parallel_scan_attachment(
        parallel_scan: pg_sys::ParallelIndexScanDesc,
    ) -> ParallelScanAttachment {
        // SAFETY: test descriptors are initialized by `test_parallel_scan_desc`
        // to point at the AM-private region inside `TestParallelScanStorage`.
        unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state")
    }

    fn test_parallel_scan_attachment_error(
        parallel_scan: pg_sys::ParallelIndexScanDesc,
    ) -> &'static str {
        // SAFETY: this negative test intentionally passes an uninitialized
        // descriptor region and expects validation to reject it.
        unsafe { parallel_scan_attachment(parallel_scan) }
            .expect_err("attachment should reject uninitialized AM-private state")
    }

    fn claim_test_worker_slot(attachment: &ParallelScanAttachment) -> u32 {
        // SAFETY: attachments used by tests come from validated test
        // descriptors and remain live for the duration of each test.
        unsafe { claim_parallel_scan_worker_slot(attachment) }.expect("claim should succeed")
    }

    fn try_claim_test_worker_slot(
        attachment: &ParallelScanAttachment,
    ) -> Result<u32, &'static str> {
        // SAFETY: attachments used by tests come from validated test
        // descriptors and remain live for the duration of each test.
        unsafe { claim_parallel_scan_worker_slot(attachment) }
    }

    fn release_test_worker_slot(attachment: &ParallelScanAttachment, slot_index: u32) -> bool {
        // SAFETY: the attachment state and slot index are produced by the same
        // initialized test descriptor.
        unsafe {
            release_parallel_scan_worker_slot(attachment.state, slot_index, attachment.rescan_epoch)
        }
        .expect("release should succeed")
    }

    fn publish_test_worker_slot_runtime_snapshot(
        attachment: &ParallelScanAttachment,
        slot_index: u32,
        runtime: EcParallelWorkerSlotRuntimeSnapshot,
    ) -> bool {
        // SAFETY: the attachment state and slot index are produced by the same
        // initialized test descriptor.
        unsafe {
            publish_parallel_scan_worker_slot_runtime_snapshot(
                attachment.state,
                slot_index,
                attachment.rescan_epoch,
                runtime,
            )
        }
        .expect("publish should succeed")
    }

    fn read_test_worker_slot_snapshot(
        attachment: &ParallelScanAttachment,
        slot_index: u32,
    ) -> EcParallelWorkerSlotSnapshot {
        // SAFETY: the attachment state and slot index are produced by the same
        // initialized test descriptor.
        unsafe { read_parallel_scan_worker_slot_snapshot(attachment.state, slot_index) }
            .expect("worker slot snapshot should read back")
    }

    fn reset_test_parallel_scan_state(parallel_scan: pg_sys::ParallelIndexScanDesc) -> u32 {
        // SAFETY: test descriptors are initialized by `test_parallel_scan_desc`
        // to point at the AM-private region inside `TestParallelScanStorage`.
        unsafe { reset_parallel_scan_state(parallel_scan) }
            .expect("parallel rescan should succeed")
            .expect("parallel rescan should see initialized state")
    }

    fn worker_slot_for_test(
        attachment: &ParallelScanAttachment,
        slot_index: u32,
    ) -> *mut EcParallelWorkerSlot {
        attachment
            .worker_slot(slot_index)
            .expect("slot index should stay within the configured capacity")
    }

    fn worker_slot_error_for_test(
        attachment: &ParallelScanAttachment,
        slot_index: u32,
    ) -> &'static str {
        attachment
            .worker_slot(slot_index)
            .expect_err("slot lookup should reject indices outside the descriptor capacity")
    }

    fn worker_slot_header_snapshot_for_test(
        attachment: &ParallelScanAttachment,
        slot_index: u32,
    ) -> (u32, u32, u32) {
        let slot = worker_slot_for_test(attachment, slot_index);
        // SAFETY: `worker_slot_for_test` returned a pointer inside the
        // validated AM-private test descriptor.
        worker_slot_header_snapshot(unsafe { &*slot })
    }

    fn coordinator_claimed_worker_slots(attachment: &ParallelScanAttachment) -> u32 {
        // SAFETY: validated attachments carry a coordinator pointer derived
        // from the same AM-private test descriptor.
        unsafe { &*attachment.coordinator }
            .claimed_worker_slots
            .load(Ordering::Acquire)
    }

    fn stage_claimed_state_for_rescan_test(attachment: &ParallelScanAttachment) {
        // SAFETY: the staged coordinator and slot pointers are derived from the
        // same validated descriptor and are reset by the test immediately after.
        unsafe {
            (&*attachment.coordinator)
                .claimed_worker_slots
                .store(2, Ordering::Release);
            (&*attachment
                .worker_slot(1)
                .expect("slot index should stay in bounds"))
                .flags
                .store(EC_PARALLEL_WORKER_SLOT_CLAIMED, Ordering::Release);
        }
    }

    #[test]
    fn ec_parallel_scan_layout_sizes_are_maxaligned() {
        assert_eq!(
            ec_parallel_scan_state_size() % size_of::<usize>(),
            0,
            "parallel scan state header should stay MAXALIGN-sized"
        );
        assert_eq!(
            ec_parallel_scan_coordinator_size() % size_of::<usize>(),
            0,
            "parallel scan coordinator header should stay MAXALIGN-sized"
        );
        assert_eq!(
            ec_parallel_scan_worker_slot_size() % size_of::<usize>(),
            0,
            "parallel worker slot header should stay MAXALIGN-sized"
        );
    }

    #[test]
    fn descriptor_size_covers_state_coordinator_and_slots() {
        let descriptor_bytes = ec_parallel_scan_descriptor_size_for(TEST_WORKER_SLOT_COUNT);
        let minimum = ec_parallel_scan_state_size()
            + ec_parallel_scan_coordinator_size()
            + ec_parallel_scan_worker_slot_size() * TEST_WORKER_SLOT_COUNT as pg_sys::Size;

        assert!(
            descriptor_bytes >= minimum,
            "descriptor size should cover the shared state, coordinator, and worker slots"
        );
        assert_eq!(
            descriptor_bytes % size_of::<usize>(),
            0,
            "descriptor size should stay MAXALIGN-sized"
        );
    }

    #[test]
    fn initialize_parallel_scan_target_round_trips_through_attachment() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = test_parallel_scan_desc(&mut storage);
        let target = test_parallel_scan_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let attachment = test_parallel_scan_attachment(parallel_scan);

        assert!(
            ptr::eq(attachment.state.cast::<c_void>(), target),
            "attachment should point at the AM-private target that init populated"
        );
        assert_eq!(
            attachment.descriptor_bytes,
            ec_parallel_scan_descriptor_size_for(TEST_WORKER_SLOT_COUNT),
            "attachment should report the initialized descriptor size"
        );
        assert_eq!(
            attachment.worker_slot_count, TEST_WORKER_SLOT_COUNT,
            "attachment should report the configured worker slot capacity"
        );
        assert_eq!(
            attachment.rescan_epoch, 0,
            "freshly initialized parallel scan state should start at epoch zero"
        );
        assert_eq!(
            coordinator_claimed_worker_slots(&attachment),
            0,
            "freshly initialized coordinator state should start with no claimed worker slots"
        );
    }

    #[test]
    fn initialize_parallel_scan_target_seeds_slot_headers() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = test_parallel_scan_desc(&mut storage);
        let target = test_parallel_scan_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let attachment = test_parallel_scan_attachment(parallel_scan);

        for slot_index in 0..TEST_WORKER_SLOT_COUNT {
            assert_eq!(
                worker_slot_header_snapshot_for_test(&attachment, slot_index),
                (EC_PARALLEL_WORKER_SLOT_FREE, slot_index, 0),
                "worker slot headers should be initialized deterministically"
            );
            assert_eq!(
                read_test_worker_slot_snapshot(&attachment, slot_index),
                EcParallelWorkerSlotSnapshot {
                    flags: EC_PARALLEL_WORKER_SLOT_FREE,
                    slot_index,
                    observed_rescan_epoch: 0,
                    runtime: EcParallelWorkerSlotRuntimeSnapshot::idle(),
                },
                "worker slot runtime should start at the idle zero snapshot"
            );
        }
    }

    #[test]
    fn claim_parallel_scan_worker_slot_claims_first_free_slots_in_order() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = test_parallel_scan_desc(&mut storage);
        let target = test_parallel_scan_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let attachment = test_parallel_scan_attachment(parallel_scan);

        assert_eq!(
            claim_test_worker_slot(&attachment),
            0,
            "first claim should reserve slot zero"
        );
        assert_eq!(
            claim_test_worker_slot(&attachment),
            1,
            "second claim should reserve the next free slot"
        );
        assert_eq!(
            coordinator_claimed_worker_slots(&attachment),
            2,
            "coordinator claim count should track the number of live claims"
        );
    }

    #[test]
    fn publish_parallel_scan_worker_slot_runtime_snapshot_records_live_state() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = test_parallel_scan_desc(&mut storage);
        let target = test_parallel_scan_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let attachment = test_parallel_scan_attachment(parallel_scan);
        let slot_index = claim_test_worker_slot(&attachment);
        let runtime = EcParallelWorkerSlotRuntimeSnapshot {
            execution_phase: EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL,
            scan_dimensions: 1536,
            bootstrap_frontier_limit: 64,
            visible_frontier_len: 5,
            scheduler_frontier_len: 8,
            visited_count: 13,
            emitted_count: 3,
            active_result_pending_count: 2,
            active_result_has_current: true,
        };

        assert!(
            publish_test_worker_slot_runtime_snapshot(&attachment, slot_index, runtime),
            "publishing should update the claimed slot for the active epoch"
        );
        assert_eq!(
            read_test_worker_slot_snapshot(&attachment, slot_index),
            EcParallelWorkerSlotSnapshot {
                flags: EC_PARALLEL_WORKER_SLOT_CLAIMED,
                slot_index,
                observed_rescan_epoch: attachment.rescan_epoch,
                runtime,
            },
            "published runtime state should round-trip through the shared slot"
        );
    }

    #[test]
    fn release_parallel_scan_worker_slot_drops_live_claims_only_once() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = test_parallel_scan_desc(&mut storage);
        let target = test_parallel_scan_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let attachment = test_parallel_scan_attachment(parallel_scan);
        let slot_index = claim_test_worker_slot(&attachment);

        assert!(
            release_test_worker_slot(&attachment, slot_index),
            "release should report that it dropped a live claim"
        );
        assert!(
            !release_test_worker_slot(&attachment, slot_index),
            "releasing the same slot twice should not underflow the claim count"
        );
        assert_eq!(
            coordinator_claimed_worker_slots(&attachment),
            0,
            "coordinator claim count should return to zero after release"
        );
        assert_eq!(
            read_test_worker_slot_snapshot(&attachment, slot_index),
            EcParallelWorkerSlotSnapshot {
                flags: EC_PARALLEL_WORKER_SLOT_FREE,
                slot_index,
                observed_rescan_epoch: attachment.rescan_epoch,
                runtime: EcParallelWorkerSlotRuntimeSnapshot::idle(),
            },
            "release should reset the slot runtime back to idle before making it free again"
        );
    }

    #[test]
    fn miri_parallel_worker_slots_are_unique_under_threaded_contention() {
        let mut storage = TestParallelScanStorage::default();
        let target = test_parallel_scan_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let shared_state = SharedParallelScanState(target.cast::<EcParallelScanState>());
        let worker_count = TEST_WORKER_SLOT_COUNT as usize + 2;
        let start = Arc::new(Barrier::new(worker_count));
        let claimed = Arc::new(Mutex::new(Vec::new()));
        let attempted = Arc::new(Barrier::new(worker_count));

        thread::scope(|scope| {
            for worker_id in 0..worker_count {
                let start = Arc::clone(&start);
                let claimed = Arc::clone(&claimed);
                let attempted = Arc::clone(&attempted);
                scope.spawn(move || {
                    start.wait();
                    let attachment = shared_state.attachment();
                    let claim = try_claim_test_worker_slot(&attachment);
                    match claim {
                        Ok(slot_index) => {
                            let runtime = EcParallelWorkerSlotRuntimeSnapshot {
                                execution_phase: EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL,
                                scan_dimensions: 768 + worker_id as u32,
                                bootstrap_frontier_limit: 32,
                                visible_frontier_len: worker_id as u32,
                                scheduler_frontier_len: worker_id as u32 + 1,
                                visited_count: worker_id as u32 + 2,
                                emitted_count: worker_id as u32,
                                active_result_pending_count: 1,
                                active_result_has_current: worker_id % 2 == 0,
                            };
                            assert!(
                                publish_test_worker_slot_runtime_snapshot(
                                    &attachment,
                                    slot_index,
                                    runtime
                                ),
                                "claimed worker should publish into its active epoch slot"
                            );
                            claimed.lock().expect("claim log lock").push((
                                worker_id as u32,
                                slot_index,
                                runtime,
                            ));
                            attempted.wait();
                            assert!(
                                release_test_worker_slot(&attachment, slot_index),
                                "claimed worker should release exactly once"
                            );
                        }
                        Err(err) => {
                            assert!(
                                err.contains("capacity"),
                                "unclaimed workers should fail only because slot capacity is exhausted: {err}"
                            );
                            attempted.wait();
                        }
                    }
                });
            }
        });

        let attachment = shared_state.attachment();
        let claimed = claimed.lock().expect("claim log lock");
        assert_eq!(
            claimed.len(),
            TEST_WORKER_SLOT_COUNT as usize,
            "contention should allow exactly one live claim per worker slot"
        );
        let claimed_slots = claimed
            .iter()
            .map(|(_, slot_index, _)| *slot_index)
            .collect::<HashSet<_>>();
        assert_eq!(
            claimed_slots.len(),
            TEST_WORKER_SLOT_COUNT as usize,
            "concurrent claims must not duplicate a worker slot"
        );
        for slot_index in 0..TEST_WORKER_SLOT_COUNT {
            assert!(
                claimed_slots.contains(&slot_index),
                "slot {slot_index} should be claimed exactly once under contention"
            );
            assert_eq!(
                read_test_worker_slot_snapshot(&attachment, slot_index),
                EcParallelWorkerSlotSnapshot {
                    flags: EC_PARALLEL_WORKER_SLOT_FREE,
                    slot_index,
                    observed_rescan_epoch: attachment.rescan_epoch,
                    runtime: EcParallelWorkerSlotRuntimeSnapshot::idle(),
                },
                "threaded release should reset slot {slot_index} before making it free"
            );
        }
        assert_eq!(
            coordinator_claimed_worker_slots(&attachment),
            0,
            "all threaded releases should return the coordinator claim count to zero"
        );
    }

    #[test]
    fn miri_publish_parallel_scan_worker_slot_runtime_snapshot_rejects_stale_epoch() {
        let mut storage = TestParallelScanStorage::default();
        let (parallel_scan, target) = test_parallel_scan_desc_and_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let attachment = test_parallel_scan_attachment(parallel_scan);
        let slot_index = claim_test_worker_slot(&attachment);

        assert_eq!(
            reset_test_parallel_scan_state(parallel_scan),
            1,
            "rescan should advance the shared epoch before the stale publish check"
        );
        assert!(
            !publish_test_worker_slot_runtime_snapshot(
                &attachment,
                slot_index,
                EcParallelWorkerSlotRuntimeSnapshot {
                    execution_phase: EC_PARALLEL_WORKER_PHASE_LINEAR_FALLBACK,
                    scan_dimensions: 96,
                    bootstrap_frontier_limit: 12,
                    visible_frontier_len: 2,
                    scheduler_frontier_len: 4,
                    visited_count: 7,
                    emitted_count: 1,
                    active_result_pending_count: 1,
                    active_result_has_current: true,
                },
            ),
            "publishing with a stale epoch should not mutate the reset slot"
        );
        assert_eq!(
            read_test_worker_slot_snapshot(&attachment, slot_index),
            EcParallelWorkerSlotSnapshot {
                flags: EC_PARALLEL_WORKER_SLOT_FREE,
                slot_index,
                observed_rescan_epoch: 1,
                runtime: EcParallelWorkerSlotRuntimeSnapshot::idle(),
            },
            "stale publish attempts should leave the reset slot at its fresh-epoch idle snapshot"
        );
    }

    #[test]
    fn reset_parallel_scan_state_advances_epoch_and_reinitializes_layout() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = test_parallel_scan_desc(&mut storage);
        let target = test_parallel_scan_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let attachment = test_parallel_scan_attachment(parallel_scan);
        stage_claimed_state_for_rescan_test(&attachment);

        assert_eq!(
            reset_test_parallel_scan_state(parallel_scan),
            1,
            "first rescan should advance the shared epoch to one"
        );

        let attachment = test_parallel_scan_attachment(parallel_scan);
        assert_eq!(
            coordinator_claimed_worker_slots(&attachment),
            0,
            "rescan should clear coordinator-side worker slot claims"
        );
        assert_eq!(
            worker_slot_header_snapshot_for_test(&attachment, 1),
            (EC_PARALLEL_WORKER_SLOT_FREE, 1, 1),
            "rescan should stamp worker slots with the new shared epoch"
        );
        assert_eq!(
            read_test_worker_slot_snapshot(&attachment, 1),
            EcParallelWorkerSlotSnapshot {
                flags: EC_PARALLEL_WORKER_SLOT_FREE,
                slot_index: 1,
                observed_rescan_epoch: 1,
                runtime: EcParallelWorkerSlotRuntimeSnapshot::idle(),
            },
            "rescan should also clear any staged worker-runtime snapshot state"
        );
    }

    #[test]
    fn worker_slot_lookup_rejects_out_of_bounds_indices() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = test_parallel_scan_desc(&mut storage);
        let target = test_parallel_scan_target(&mut storage);

        initialize_test_parallel_scan_target(target);
        let attachment = test_parallel_scan_attachment(parallel_scan);

        let err = worker_slot_error_for_test(&attachment, TEST_WORKER_SLOT_COUNT);
        assert!(
            err.contains("outside"),
            "out-of-bounds slot lookup should fail with the capacity check"
        );
    }

    #[test]
    fn parallel_scan_attachment_rejects_uninitialized_state() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = test_parallel_scan_desc(&mut storage);

        let err = test_parallel_scan_attachment_error(parallel_scan);
        assert!(
            err.contains("magic"),
            "uninitialized shared state should fail the magic check first"
        );
    }
}
