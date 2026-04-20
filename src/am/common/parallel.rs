use std::ffi::{c_int, c_void};
use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};

use pgrx::pg_sys;

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

const EC_PARALLEL_WORKER_SLOT_FREE: u32 = 0;
const EC_PARALLEL_WORKER_SLOT_CLAIMED: u32 = 1;
pub(crate) const EC_PARALLEL_WORKER_PHASE_IDLE: u32 = 0;
pub(crate) const EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL: u32 = 1;
pub(crate) const EC_PARALLEL_WORKER_PHASE_LINEAR_FALLBACK: u32 = 2;
pub(crate) const EC_PARALLEL_WORKER_PHASE_EXHAUSTED: u32 = 3;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct EcParallelWorkerSlotRuntimeSnapshot {
    pub(crate) execution_phase: u32,
    pub(crate) scan_dimensions: u32,
    pub(crate) bootstrap_frontier_limit: u32,
    pub(crate) visible_frontier_len: u32,
    pub(crate) scheduler_frontier_len: u32,
    pub(crate) visited_count: u32,
    pub(crate) emitted_count: u32,
    pub(crate) active_result_pending_count: u32,
    pub(crate) active_result_has_current: bool,
}

impl EcParallelWorkerSlotRuntimeSnapshot {
    const fn idle() -> Self {
        Self {
            execution_phase: EC_PARALLEL_WORKER_PHASE_IDLE,
            scan_dimensions: 0,
            bootstrap_frontier_limit: 0,
            visible_frontier_len: 0,
            scheduler_frontier_len: 0,
            visited_count: 0,
            emitted_count: 0,
            active_result_pending_count: 0,
            active_result_has_current: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct EcParallelWorkerSlotSnapshot {
    pub(crate) flags: u32,
    pub(crate) slot_index: u32,
    pub(crate) observed_rescan_epoch: u32,
    pub(crate) runtime: EcParallelWorkerSlotRuntimeSnapshot,
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
    pub(crate) unsafe fn worker_slot(
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
    let max_workers = unsafe { pg_sys::max_parallel_workers_per_gather }.max(0) as u32;
    max_workers.saturating_add(1)
}

pub(crate) fn ec_parallel_scan_descriptor_size() -> pg_sys::Size {
    ec_parallel_scan_descriptor_size_for(ec_parallel_scan_worker_slot_capacity())
}

unsafe fn coordinator_ptr(state: *mut EcParallelScanState) -> *mut EcParallelCoordinatorState {
    unsafe { state.cast::<u8>().add(ec_parallel_scan_state_size()) }.cast()
}

unsafe fn worker_slots_ptr(state: *mut EcParallelScanState) -> *mut EcParallelWorkerSlot {
    let coordinator_offset = checked_add_size(
        ec_parallel_scan_state_size(),
        unsafe { (*state).coordinator_bytes },
        "parallel worker slot base offset",
    );
    unsafe { state.cast::<u8>().add(coordinator_offset) }.cast()
}

unsafe fn reset_parallel_scan_layout(state: *mut EcParallelScanState) {
    let state_ref = unsafe { &mut *state };
    state_ref.reserved_worker_slots = 0;

    unsafe {
        *coordinator_ptr(state) = EcParallelCoordinatorState {
            flags: AtomicU32::new(0),
            claimed_worker_slots: AtomicU32::new(0),
            reserved0: 0,
            reserved1: 0,
        };
    }

    for slot_index in 0..state_ref.worker_slot_count {
        let slot = unsafe {
            worker_slots_ptr(state)
                .cast::<u8>()
                .add(checked_mul_size(
                    state_ref.worker_slot_bytes,
                    slot_index as pg_sys::Size,
                    "parallel worker slot reset offset",
                ))
                .cast::<EcParallelWorkerSlot>()
        };
        unsafe {
            *slot = EcParallelWorkerSlot {
                flags: AtomicU32::new(EC_PARALLEL_WORKER_SLOT_FREE),
                slot_index,
                observed_rescan_epoch: AtomicU32::new(state_ref.rescan_epoch),
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

fn reset_worker_slot_runtime(slot: &EcParallelWorkerSlot) {
    let runtime = EcParallelWorkerSlotRuntimeSnapshot::idle();
    slot.execution_phase
        .store(runtime.execution_phase, Ordering::Release);
    slot.scan_dimensions
        .store(runtime.scan_dimensions, Ordering::Release);
    slot.bootstrap_frontier_limit
        .store(runtime.bootstrap_frontier_limit, Ordering::Release);
    slot.visible_frontier_len
        .store(runtime.visible_frontier_len, Ordering::Release);
    slot.scheduler_frontier_len
        .store(runtime.scheduler_frontier_len, Ordering::Release);
    slot.visited_count
        .store(runtime.visited_count, Ordering::Release);
    slot.emitted_count
        .store(runtime.emitted_count, Ordering::Release);
    slot.active_result_pending_count
        .store(runtime.active_result_pending_count, Ordering::Release);
    slot.active_result_has_current.store(
        u32::from(runtime.active_result_has_current),
        Ordering::Release,
    );
}

fn load_worker_slot_snapshot(slot: &EcParallelWorkerSlot) -> EcParallelWorkerSlotSnapshot {
    EcParallelWorkerSlotSnapshot {
        flags: slot.flags.load(Ordering::Acquire),
        slot_index: slot.slot_index,
        observed_rescan_epoch: slot.observed_rescan_epoch.load(Ordering::Acquire),
        runtime: EcParallelWorkerSlotRuntimeSnapshot {
            execution_phase: slot.execution_phase.load(Ordering::Acquire),
            scan_dimensions: slot.scan_dimensions.load(Ordering::Acquire),
            bootstrap_frontier_limit: slot.bootstrap_frontier_limit.load(Ordering::Acquire),
            visible_frontier_len: slot.visible_frontier_len.load(Ordering::Acquire),
            scheduler_frontier_len: slot.scheduler_frontier_len.load(Ordering::Acquire),
            visited_count: slot.visited_count.load(Ordering::Acquire),
            emitted_count: slot.emitted_count.load(Ordering::Acquire),
            active_result_pending_count: slot.active_result_pending_count.load(Ordering::Acquire),
            active_result_has_current: slot.active_result_has_current.load(Ordering::Acquire) != 0,
        },
    }
}

fn initialize_parallel_scan_state(state: &mut EcParallelScanState, worker_slot_count: u32) {
    *state = EcParallelScanState {
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
    };
    unsafe { reset_parallel_scan_layout(state) };
}

unsafe fn validate_parallel_scan_state(
    state: *mut EcParallelScanState,
) -> Result<ParallelScanAttachment, &'static str> {
    if state.is_null() {
        return Err("AM-private parallel scan state pointer was null");
    }

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
        coordinator: unsafe { coordinator_ptr(state) },
        worker_slots: unsafe { worker_slots_ptr(state) },
        descriptor_bytes: state_ref.descriptor_bytes,
        worker_slot_count: state_ref.worker_slot_count,
        worker_slot_bytes: state_ref.worker_slot_bytes,
        rescan_epoch: state_ref.rescan_epoch,
    })
}

#[cfg(feature = "pg17")]
unsafe fn parallel_scan_state_ptr(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<*mut EcParallelScanState>, &'static str> {
    if parallel_scan.is_null() {
        return Ok(None);
    }
    let offset = unsafe { (*parallel_scan).ps_offset };
    if offset == 0 {
        return Ok(None);
    }
    Ok(Some(
        unsafe { parallel_scan.cast::<u8>().add(offset) }.cast::<EcParallelScanState>(),
    ))
}

#[cfg(feature = "pg18")]
unsafe fn parallel_scan_state_ptr(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<*mut EcParallelScanState>, &'static str> {
    if parallel_scan.is_null() {
        return Ok(None);
    }
    let offset = unsafe { (*parallel_scan).ps_offset_am };
    if offset == 0 {
        return Ok(None);
    }
    Ok(Some(
        unsafe { parallel_scan.cast::<u8>().add(offset) }.cast::<EcParallelScanState>(),
    ))
}

pub(crate) unsafe fn parallel_scan_attachment(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<ParallelScanAttachment>, &'static str> {
    let Some(state) = (unsafe { parallel_scan_state_ptr(parallel_scan) })? else {
        return Ok(None);
    };
    Ok(Some(unsafe { validate_parallel_scan_state(state) }?))
}

pub(crate) unsafe fn initialize_parallel_scan_target_with_worker_slots(
    target: *mut c_void,
    worker_slot_count: u32,
) -> Result<(), &'static str> {
    if target.is_null() {
        return Err("AM-private parallel scan target was null");
    }
    unsafe {
        initialize_parallel_scan_state(
            &mut *target.cast::<EcParallelScanState>(),
            worker_slot_count,
        )
    };
    Ok(())
}

pub(crate) unsafe fn initialize_parallel_scan_target(
    target: *mut c_void,
) -> Result<(), &'static str> {
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
        let slot = unsafe { attachment.worker_slot(slot_index) }?;
        let slot_ref = unsafe { &*slot };
        let observed_rescan_epoch = slot_ref.observed_rescan_epoch.load(Ordering::Acquire);
        if observed_rescan_epoch != attachment.rescan_epoch {
            continue;
        }

        if slot_ref
            .flags
            .compare_exchange(
                EC_PARALLEL_WORKER_SLOT_FREE,
                EC_PARALLEL_WORKER_SLOT_CLAIMED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            reset_worker_slot_runtime(slot_ref);
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
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    let slot = unsafe { attachment.worker_slot(slot_index) }?;
    let slot_ref = unsafe { &*slot };
    if slot_ref.observed_rescan_epoch.load(Ordering::Acquire) != rescan_epoch {
        return Ok(false);
    }

    if slot_ref.flags.load(Ordering::Acquire) != EC_PARALLEL_WORKER_SLOT_CLAIMED {
        return Ok(false);
    }

    reset_worker_slot_runtime(slot_ref);
    if slot_ref
        .flags
        .compare_exchange(
            EC_PARALLEL_WORKER_SLOT_CLAIMED,
            EC_PARALLEL_WORKER_SLOT_FREE,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_ok()
    {
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
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    let slot = unsafe { attachment.worker_slot(slot_index) }?;
    let slot_ref = unsafe { &*slot };
    if slot_ref.observed_rescan_epoch.load(Ordering::Acquire) != rescan_epoch {
        return Ok(false);
    }
    if slot_ref.flags.load(Ordering::Acquire) != EC_PARALLEL_WORKER_SLOT_CLAIMED {
        return Ok(false);
    }

    slot_ref
        .execution_phase
        .store(snapshot.execution_phase, Ordering::Release);
    slot_ref
        .scan_dimensions
        .store(snapshot.scan_dimensions, Ordering::Release);
    slot_ref
        .bootstrap_frontier_limit
        .store(snapshot.bootstrap_frontier_limit, Ordering::Release);
    slot_ref
        .visible_frontier_len
        .store(snapshot.visible_frontier_len, Ordering::Release);
    slot_ref
        .scheduler_frontier_len
        .store(snapshot.scheduler_frontier_len, Ordering::Release);
    slot_ref
        .visited_count
        .store(snapshot.visited_count, Ordering::Release);
    slot_ref
        .emitted_count
        .store(snapshot.emitted_count, Ordering::Release);
    slot_ref
        .active_result_pending_count
        .store(snapshot.active_result_pending_count, Ordering::Release);
    slot_ref.active_result_has_current.store(
        u32::from(snapshot.active_result_has_current),
        Ordering::Release,
    );
    Ok(true)
}

pub(crate) unsafe fn read_parallel_scan_worker_slot_snapshot(
    state: *mut EcParallelScanState,
    slot_index: u32,
) -> Result<EcParallelWorkerSlotSnapshot, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    let slot = unsafe { attachment.worker_slot(slot_index) }?;
    Ok(load_worker_slot_snapshot(unsafe { &*slot }))
}

pub(crate) unsafe fn reset_parallel_scan_state(
    parallel_scan: pg_sys::ParallelIndexScanDesc,
) -> Result<Option<u32>, &'static str> {
    let Some(state) = (unsafe { parallel_scan_state_ptr(parallel_scan) })? else {
        return Ok(None);
    };
    let rescan_epoch = {
        let state_ref = unsafe { &mut *state };
        if state_ref.magic != EC_PARALLEL_SCAN_STATE_MAGIC
            || state_ref.version != EC_PARALLEL_SCAN_STATE_VERSION
        {
            return Err("AM-private parallel scan state was not initialized before rescan");
        }
        state_ref.rescan_epoch = state_ref.rescan_epoch.wrapping_add(1);
        state_ref.rescan_epoch
    };
    unsafe { reset_parallel_scan_layout(state) };
    Ok(Some(rescan_epoch))
}

#[cfg(feature = "pg17")]
pub(crate) unsafe extern "C-unwind" fn ec_amestimateparallelscan(
    _nkeys: c_int,
    _norderbys: c_int,
) -> pg_sys::Size {
    ec_parallel_scan_descriptor_size()
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_amestimateparallelscan(
    _index_relation: pg_sys::Relation,
    _nkeys: c_int,
    _norderbys: c_int,
) -> pg_sys::Size {
    ec_parallel_scan_descriptor_size()
}

pub(crate) unsafe extern "C-unwind" fn ec_aminitparallelscan(target: *mut c_void) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            initialize_parallel_scan_target(target)
                .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel scan init failed: {err}"));
        })
    }
}

pub(crate) unsafe extern "C-unwind" fn ec_amparallelrescan(scan: pg_sys::IndexScanDesc) {
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
    use std::ptr;

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

    unsafe fn test_parallel_scan_desc(
        storage: &mut TestParallelScanStorage,
    ) -> pg_sys::ParallelIndexScanDesc {
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        {
            unsafe { (*parallel_scan).ps_offset = TEST_PARALLEL_SCAN_OFFSET };
        }
        #[cfg(feature = "pg18")]
        {
            unsafe { (*parallel_scan).ps_offset_am = TEST_PARALLEL_SCAN_OFFSET };
        }
        parallel_scan
    }

    unsafe fn test_parallel_scan_target(storage: &mut TestParallelScanStorage) -> *mut c_void {
        unsafe { storage.bytes.as_mut_ptr().add(TEST_PARALLEL_SCAN_OFFSET) }.cast::<c_void>()
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
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");

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
            unsafe { &*attachment.coordinator }
                .claimed_worker_slots
                .load(Ordering::Acquire),
            0,
            "freshly initialized coordinator state should start with no claimed worker slots"
        );
    }

    #[test]
    fn initialize_parallel_scan_target_seeds_slot_headers() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");

        for slot_index in 0..TEST_WORKER_SLOT_COUNT {
            let slot = unsafe { attachment.worker_slot(slot_index) }
                .expect("slot index should stay within the configured capacity");
            assert_eq!(
                worker_slot_header_snapshot(unsafe { &*slot }),
                (EC_PARALLEL_WORKER_SLOT_FREE, slot_index, 0),
                "worker slot headers should be initialized deterministically"
            );
            assert_eq!(
                unsafe { read_parallel_scan_worker_slot_snapshot(attachment.state, slot_index) }
                    .expect("worker slot snapshot should read back"),
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
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");

        assert_eq!(
            unsafe { claim_parallel_scan_worker_slot(&attachment) }
                .expect("first claim should succeed"),
            0,
            "first claim should reserve slot zero"
        );
        assert_eq!(
            unsafe { claim_parallel_scan_worker_slot(&attachment) }
                .expect("second claim should succeed"),
            1,
            "second claim should reserve the next free slot"
        );
        assert_eq!(
            unsafe { &*attachment.coordinator }
                .claimed_worker_slots
                .load(Ordering::Acquire),
            2,
            "coordinator claim count should track the number of live claims"
        );
    }

    #[test]
    fn publish_parallel_scan_worker_slot_runtime_snapshot_records_live_state() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");
        let slot_index = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("claim should succeed before publishing");
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
            unsafe {
                publish_parallel_scan_worker_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                    runtime,
                )
            }
            .expect("publish should succeed"),
            "publishing should update the claimed slot for the active epoch"
        );
        assert_eq!(
            unsafe { read_parallel_scan_worker_slot_snapshot(attachment.state, slot_index) }
                .expect("worker slot snapshot should read back"),
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
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");
        let slot_index = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("claim should succeed before release");

        assert!(
            unsafe {
                release_parallel_scan_worker_slot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                )
            }
            .expect("release should succeed"),
            "release should report that it dropped a live claim"
        );
        assert!(
            !unsafe {
                release_parallel_scan_worker_slot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                )
            }
            .expect("double release should stay benign"),
            "releasing the same slot twice should not underflow the claim count"
        );
        assert_eq!(
            unsafe { &*attachment.coordinator }
                .claimed_worker_slots
                .load(Ordering::Acquire),
            0,
            "coordinator claim count should return to zero after release"
        );
        assert_eq!(
            unsafe { read_parallel_scan_worker_slot_snapshot(attachment.state, slot_index) }
                .expect("worker slot snapshot should stay readable"),
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
    fn publish_parallel_scan_worker_slot_runtime_snapshot_rejects_stale_epoch() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");
        let slot_index = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("claim should succeed before stale publish check");

        assert_eq!(
            unsafe { reset_parallel_scan_state(parallel_scan) }
                .expect("parallel rescan should succeed")
                .expect("parallel rescan should see initialized state"),
            1,
            "rescan should advance the shared epoch before the stale publish check"
        );
        assert!(
            !unsafe {
                publish_parallel_scan_worker_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
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
                )
            }
            .expect("stale publish should return a benign false"),
            "publishing with a stale epoch should not mutate the reset slot"
        );
        assert_eq!(
            unsafe { read_parallel_scan_worker_slot_snapshot(attachment.state, slot_index) }
                .expect("worker slot snapshot should remain readable after rescan"),
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
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");
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

        assert_eq!(
            unsafe { reset_parallel_scan_state(parallel_scan) }
                .expect("parallel rescan should succeed")
                .expect("parallel rescan should see the initialized state"),
            1,
            "first rescan should advance the shared epoch to one"
        );

        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should keep validating")
            .expect("parallel descriptor should keep exposing AM state");
        assert_eq!(
            unsafe { &*attachment.coordinator }
                .claimed_worker_slots
                .load(Ordering::Acquire),
            0,
            "rescan should clear coordinator-side worker slot claims"
        );
        assert_eq!(
            worker_slot_header_snapshot(unsafe {
                &*attachment
                    .worker_slot(1)
                    .expect("slot index should stay in bounds")
            }),
            (EC_PARALLEL_WORKER_SLOT_FREE, 1, 1),
            "rescan should stamp worker slots with the new shared epoch"
        );
        assert_eq!(
            unsafe { read_parallel_scan_worker_slot_snapshot(attachment.state, 1) }
                .expect("worker slot snapshot should read back after rescan"),
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
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };
        let target = unsafe { test_parallel_scan_target(&mut storage) };

        unsafe {
            initialize_parallel_scan_target_with_worker_slots(target, TEST_WORKER_SLOT_COUNT)
        }
        .expect("parallel target should init");
        let attachment = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect("parallel descriptor should validate")
            .expect("parallel descriptor should expose AM state");

        let err = unsafe { attachment.worker_slot(TEST_WORKER_SLOT_COUNT) }
            .expect_err("slot lookup should reject indices outside the descriptor capacity");
        assert!(
            err.contains("outside"),
            "out-of-bounds slot lookup should fail with the capacity check"
        );
    }

    #[test]
    fn parallel_scan_attachment_rejects_uninitialized_state() {
        let mut storage = TestParallelScanStorage::default();
        let parallel_scan = unsafe { test_parallel_scan_desc(&mut storage) };

        let err = unsafe { parallel_scan_attachment(parallel_scan) }
            .expect_err("attachment should reject uninitialized AM-private state");
        assert!(
            err.contains("magic"),
            "uninitialized shared state should fail the magic check first"
        );
    }
}
