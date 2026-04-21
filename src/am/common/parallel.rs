use std::ffi::{c_int, c_void};
use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};

use pgrx::pg_sys;

const EC_PARALLEL_SCAN_STATE_MAGIC: u32 = u32::from_le_bytes(*b"ECPR");
const EC_PARALLEL_SCAN_STATE_VERSION: u16 = 6;
const EC_PARALLEL_HEAP_ENTRY_INVALID: u32 = u32::MAX;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct EcParallelScanState {
    magic: u32,
    version: u16,
    flags: u16,
    descriptor_bytes: pg_sys::Size,
    coordinator_bytes: pg_sys::Size,
    heap_bytes: pg_sys::Size,
    heap_entry_bytes: pg_sys::Size,
    result_slot_bytes: pg_sys::Size,
    worker_slot_bytes: pg_sys::Size,
    heap_entry_count: u32,
    result_slot_count: u32,
    worker_slot_count: u32,
    reserved_worker_slots: u32,
    reserved0: u32,
    rescan_epoch: u32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct EcParallelCoordinatorState {
    pub(crate) flags: AtomicU32,
    pub(crate) claimed_worker_slots: AtomicU32,
    pub(crate) published_result_slots: AtomicU32,
    pub(crate) result_publish_generation: AtomicU32,
    pub(crate) selected_result_slot_index: AtomicU32,
    pub(crate) selected_result_score_bits: AtomicU32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct EcParallelCoordinatorHeapState {
    pub(crate) mutex: AtomicU32,
    pub(crate) live_entry_count: AtomicU32,
    pub(crate) heap_generation: AtomicU32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct EcParallelCoordinatorResultSlot {
    pub(crate) flags: AtomicU32,
    pub(crate) slot_index: u32,
    pub(crate) observed_rescan_epoch: AtomicU32,
    heap_index: AtomicU32,
    element_block_number: AtomicU32,
    element_offset_number: AtomicU32,
    heap_block_number: AtomicU32,
    heap_offset_number: AtomicU32,
    score_bits: AtomicU32,
    approx_score_bits: AtomicU32,
    comparison_score_bits: AtomicU32,
    approx_rank_base_bits: AtomicU32,
    pending_count: AtomicU32,
    pending_index: AtomicU32,
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
const EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID: u32 = 1 << 0;
const EC_PARALLEL_RESULT_SLOT_PUBLISHED: u32 = 1 << 0;
const EC_PARALLEL_RESULT_SLOT_SCORE_VALID: u32 = 1 << 1;
const EC_PARALLEL_RESULT_SLOT_APPROX_SCORE_VALID: u32 = 1 << 2;
const EC_PARALLEL_RESULT_SLOT_COMPARISON_SCORE_VALID: u32 = 1 << 3;
const EC_PARALLEL_RESULT_SLOT_APPROX_RANK_VALID: u32 = 1 << 4;
const EC_PARALLEL_RESULT_SLOT_HEAP_TID_VALID: u32 = 1 << 5;
pub(crate) const EC_PARALLEL_WORKER_PHASE_IDLE: u32 = 0;
pub(crate) const EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL: u32 = 1;
pub(crate) const EC_PARALLEL_WORKER_PHASE_LINEAR_FALLBACK: u32 = 2;
pub(crate) const EC_PARALLEL_WORKER_PHASE_EXHAUSTED: u32 = 3;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct EcParallelItemPointer {
    pub(crate) block_number: u32,
    pub(crate) offset_number: u16,
}

impl EcParallelItemPointer {
    pub(crate) const INVALID: Self = Self {
        block_number: u32::MAX,
        offset_number: u16::MAX,
    };

    pub(crate) const fn is_valid(self) -> bool {
        self.block_number != Self::INVALID.block_number
            || self.offset_number != Self::INVALID.offset_number
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct EcParallelCoordinatorResultSlotRuntimeSnapshot {
    pub(crate) element_tid: EcParallelItemPointer,
    pub(crate) heap_tid: EcParallelItemPointer,
    pub(crate) score: f32,
    pub(crate) approx_score: Option<f32>,
    pub(crate) comparison_score: Option<f32>,
    pub(crate) approx_rank_base: Option<i32>,
    pub(crate) pending_count: u32,
    pub(crate) pending_index: u32,
}

impl EcParallelCoordinatorResultSlotRuntimeSnapshot {
    const fn idle() -> Self {
        Self {
            element_tid: EcParallelItemPointer::INVALID,
            heap_tid: EcParallelItemPointer::INVALID,
            score: 0.0,
            approx_score: None,
            comparison_score: None,
            approx_rank_base: None,
            pending_count: 0,
            pending_index: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct EcParallelCoordinatorResultSlotSnapshot {
    pub(crate) flags: u32,
    pub(crate) slot_index: u32,
    pub(crate) observed_rescan_epoch: u32,
    pub(crate) runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct EcParallelCoordinatorSnapshot {
    pub(crate) flags: u32,
    pub(crate) claimed_worker_slots: u32,
    pub(crate) published_result_slots: u32,
    pub(crate) result_publish_generation: u32,
    pub(crate) selected_result_slot_index: Option<u32>,
    pub(crate) selected_result_score: Option<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct EcParallelCoordinatorHeapSnapshot {
    pub(crate) live_entry_count: u32,
    pub(crate) entry_capacity: u32,
    pub(crate) heap_generation: u32,
    pub(crate) root_slot_index: Option<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct EcParallelCoordinatorResultSelection {
    pub(crate) coordinator: EcParallelCoordinatorSnapshot,
    pub(crate) selected_result_slot: EcParallelCoordinatorResultSlotSnapshot,
}

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
    heap_state: *mut EcParallelCoordinatorHeapState,
    result_slots: *mut EcParallelCoordinatorResultSlot,
    heap_entries: *mut u32,
    worker_slots: *mut EcParallelWorkerSlot,
    pub(crate) descriptor_bytes: pg_sys::Size,
    pub(crate) heap_entry_count: u32,
    pub(crate) result_slot_count: u32,
    pub(crate) worker_slot_count: u32,
    heap_entry_bytes: pg_sys::Size,
    result_slot_bytes: pg_sys::Size,
    worker_slot_bytes: pg_sys::Size,
    pub(crate) rescan_epoch: u32,
}

struct ParallelScanHeapLockGuard {
    lock: *const AtomicU32,
}

impl Drop for ParallelScanHeapLockGuard {
    fn drop(&mut self) {
        unsafe { &*self.lock }.store(0, Ordering::Release);
    }
}

impl ParallelScanAttachment {
    pub(crate) unsafe fn result_slot(
        &self,
        slot_index: u32,
    ) -> Result<*mut EcParallelCoordinatorResultSlot, &'static str> {
        if slot_index >= self.result_slot_count {
            return Err("parallel result slot index was outside the descriptor capacity");
        }
        let offset = checked_mul_size(
            self.result_slot_bytes,
            slot_index as pg_sys::Size,
            "parallel result slot offset",
        );
        Ok(unsafe { self.result_slots.cast::<u8>().add(offset) }.cast())
    }

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

    pub(crate) unsafe fn heap_entry(&self, entry_index: u32) -> Result<*mut u32, &'static str> {
        if entry_index >= self.heap_entry_count {
            return Err("parallel heap entry index was outside the descriptor capacity");
        }
        let offset = checked_mul_size(
            self.heap_entry_bytes,
            entry_index as pg_sys::Size,
            "parallel heap entry offset",
        );
        Ok(unsafe { self.heap_entries.cast::<u8>().add(offset) }.cast())
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

pub(crate) fn ec_parallel_scan_heap_size() -> pg_sys::Size {
    maxalign(size_of::<EcParallelCoordinatorHeapState>())
}

pub(crate) fn ec_parallel_scan_heap_entry_size() -> pg_sys::Size {
    size_of::<u32>()
}

pub(crate) fn ec_parallel_scan_result_slot_size() -> pg_sys::Size {
    maxalign(size_of::<EcParallelCoordinatorResultSlot>())
}

pub(crate) fn ec_parallel_scan_worker_slot_size() -> pg_sys::Size {
    maxalign(size_of::<EcParallelWorkerSlot>())
}

fn ec_parallel_scan_descriptor_size_for(worker_slot_count: u32) -> pg_sys::Size {
    let heap_entry_count = ec_parallel_scan_heap_entry_capacity_for(worker_slot_count);
    let heap_entry_bytes = checked_mul_size(
        ec_parallel_scan_heap_entry_size(),
        heap_entry_count as pg_sys::Size,
        "parallel heap entry descriptor size",
    );
    let result_slot_count = ec_parallel_scan_result_slot_capacity_for(worker_slot_count);
    let result_slot_bytes = checked_mul_size(
        ec_parallel_scan_result_slot_size(),
        result_slot_count as pg_sys::Size,
        "parallel result slot descriptor size",
    );
    let worker_slot_bytes = checked_mul_size(
        ec_parallel_scan_worker_slot_size(),
        worker_slot_count as pg_sys::Size,
        "parallel worker slot descriptor size",
    );
    let shared_header_bytes = checked_add_size(
        checked_add_size(
            ec_parallel_scan_state_size(),
            ec_parallel_scan_coordinator_size(),
            "parallel scan state plus coordinator size",
        ),
        checked_add_size(
            ec_parallel_scan_heap_size(),
            heap_entry_bytes,
            "parallel heap header plus entry array size",
        ),
        "parallel scan state plus coordinator and heap size",
    );
    let shared_header_bytes = checked_add_size(
        shared_header_bytes,
        result_slot_bytes,
        "parallel scan state plus coordinator and result-slot size",
    );
    maxalign(checked_add_size(
        shared_header_bytes,
        worker_slot_bytes,
        "parallel scan descriptor size",
    ))
}

pub(crate) fn ec_parallel_scan_worker_slot_capacity() -> u32 {
    let max_workers = unsafe { pg_sys::max_parallel_workers_per_gather }.max(0) as u32;
    max_workers.saturating_add(1)
}

fn ec_parallel_scan_heap_entry_capacity_for(worker_slot_count: u32) -> u32 {
    worker_slot_count
}

fn ec_parallel_scan_result_slot_capacity_for(worker_slot_count: u32) -> u32 {
    worker_slot_count
}

pub(crate) fn ec_parallel_scan_result_slot_capacity() -> u32 {
    ec_parallel_scan_result_slot_capacity_for(ec_parallel_scan_worker_slot_capacity())
}

pub(crate) fn ec_parallel_scan_descriptor_size() -> pg_sys::Size {
    ec_parallel_scan_descriptor_size_for(ec_parallel_scan_worker_slot_capacity())
}

unsafe fn coordinator_ptr(state: *mut EcParallelScanState) -> *mut EcParallelCoordinatorState {
    unsafe { state.cast::<u8>().add(ec_parallel_scan_state_size()) }.cast()
}

unsafe fn result_slots_ptr(
    state: *mut EcParallelScanState,
) -> *mut EcParallelCoordinatorResultSlot {
    let result_slot_offset = checked_add_size(
        ec_parallel_scan_state_size(),
        unsafe { (*state).coordinator_bytes },
        "parallel result slot base offset",
    );
    let result_slot_offset = checked_add_size(
        result_slot_offset,
        checked_add_size(
            unsafe { (*state).heap_bytes },
            checked_mul_size(
                unsafe { (*state).heap_entry_bytes },
                unsafe { (*state).heap_entry_count as pg_sys::Size },
                "parallel heap entry bytes span",
            ),
            "parallel heap header and entry span",
        ),
        "parallel result slot base offset after heap state",
    );
    unsafe { state.cast::<u8>().add(result_slot_offset) }.cast()
}

unsafe fn heap_state_ptr(state: *mut EcParallelScanState) -> *mut EcParallelCoordinatorHeapState {
    let heap_offset = checked_add_size(
        ec_parallel_scan_state_size(),
        unsafe { (*state).coordinator_bytes },
        "parallel heap state base offset",
    );
    unsafe { state.cast::<u8>().add(heap_offset) }.cast()
}

unsafe fn heap_entries_ptr(state: *mut EcParallelScanState) -> *mut u32 {
    let heap_entries_offset = checked_add_size(
        ec_parallel_scan_state_size(),
        checked_add_size(
            unsafe { (*state).coordinator_bytes },
            unsafe { (*state).heap_bytes },
            "parallel heap state plus header size",
        ),
        "parallel heap entry base offset",
    );
    unsafe { state.cast::<u8>().add(heap_entries_offset) }.cast()
}

unsafe fn worker_slots_ptr(state: *mut EcParallelScanState) -> *mut EcParallelWorkerSlot {
    let coordinator_heap_results_offset = checked_add_size(
        ec_parallel_scan_state_size(),
        checked_add_size(
            unsafe { (*state).coordinator_bytes },
            checked_add_size(
                checked_add_size(
                    unsafe { (*state).heap_bytes },
                    checked_mul_size(
                        unsafe { (*state).heap_entry_bytes },
                        unsafe { (*state).heap_entry_count as pg_sys::Size },
                        "parallel heap entry bytes span",
                    ),
                    "parallel heap entry span offset",
                ),
                checked_mul_size(
                    unsafe { (*state).result_slot_bytes },
                    unsafe { (*state).result_slot_count as pg_sys::Size },
                    "parallel result slot bytes span",
                ),
                "parallel result slot span offset",
            ),
            "parallel heap and result slot span offset",
        ),
        "parallel worker slot base offset",
    );
    unsafe { state.cast::<u8>().add(coordinator_heap_results_offset) }.cast()
}

unsafe fn reset_parallel_scan_layout(state: *mut EcParallelScanState) {
    let state_ref = unsafe { &mut *state };
    state_ref.reserved_worker_slots = 0;

    unsafe {
        *coordinator_ptr(state) = EcParallelCoordinatorState {
            flags: AtomicU32::new(0),
            claimed_worker_slots: AtomicU32::new(0),
            published_result_slots: AtomicU32::new(0),
            result_publish_generation: AtomicU32::new(0),
            selected_result_slot_index: AtomicU32::new(u32::MAX),
            selected_result_score_bits: AtomicU32::new(0),
        };
    }

    unsafe {
        *heap_state_ptr(state) = EcParallelCoordinatorHeapState {
            mutex: AtomicU32::new(0),
            live_entry_count: AtomicU32::new(0),
            heap_generation: AtomicU32::new(0),
        };
    }

    for entry_index in 0..state_ref.heap_entry_count {
        let entry = unsafe {
            heap_entries_ptr(state).cast::<u8>().add(checked_mul_size(
                state_ref.heap_entry_bytes,
                entry_index as pg_sys::Size,
                "parallel heap entry reset offset",
            ))
        }
        .cast::<u32>();
        unsafe { *entry = EC_PARALLEL_HEAP_ENTRY_INVALID };
    }

    for slot_index in 0..state_ref.result_slot_count {
        let slot = unsafe {
            result_slots_ptr(state)
                .cast::<u8>()
                .add(checked_mul_size(
                    state_ref.result_slot_bytes,
                    slot_index as pg_sys::Size,
                    "parallel result slot reset offset",
                ))
                .cast::<EcParallelCoordinatorResultSlot>()
        };
        unsafe {
            *slot = EcParallelCoordinatorResultSlot {
                flags: AtomicU32::new(0),
                slot_index,
                observed_rescan_epoch: AtomicU32::new(state_ref.rescan_epoch),
                heap_index: AtomicU32::new(EC_PARALLEL_HEAP_ENTRY_INVALID),
                element_block_number: AtomicU32::new(EcParallelItemPointer::INVALID.block_number),
                element_offset_number: AtomicU32::new(
                    EcParallelItemPointer::INVALID.offset_number as u32,
                ),
                heap_block_number: AtomicU32::new(EcParallelItemPointer::INVALID.block_number),
                heap_offset_number: AtomicU32::new(
                    EcParallelItemPointer::INVALID.offset_number as u32,
                ),
                score_bits: AtomicU32::new(0),
                approx_score_bits: AtomicU32::new(0),
                comparison_score_bits: AtomicU32::new(0),
                approx_rank_base_bits: AtomicU32::new(0),
                pending_count: AtomicU32::new(0),
                pending_index: AtomicU32::new(0),
            };
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

fn store_parallel_item_pointer(
    block_number: &AtomicU32,
    offset_number: &AtomicU32,
    tid: EcParallelItemPointer,
) {
    block_number.store(tid.block_number, Ordering::Release);
    offset_number.store(u32::from(tid.offset_number), Ordering::Release);
}

fn load_parallel_item_pointer(
    block_number: &AtomicU32,
    offset_number: &AtomicU32,
) -> EcParallelItemPointer {
    EcParallelItemPointer {
        block_number: block_number.load(Ordering::Acquire),
        offset_number: u16::try_from(offset_number.load(Ordering::Acquire))
            .expect("parallel item-pointer offsets should fit in u16"),
    }
}

fn reset_result_slot_runtime(slot: &EcParallelCoordinatorResultSlot) {
    let runtime = EcParallelCoordinatorResultSlotRuntimeSnapshot::idle();
    slot.heap_index
        .store(EC_PARALLEL_HEAP_ENTRY_INVALID, Ordering::Release);
    store_parallel_item_pointer(
        &slot.element_block_number,
        &slot.element_offset_number,
        runtime.element_tid,
    );
    store_parallel_item_pointer(
        &slot.heap_block_number,
        &slot.heap_offset_number,
        runtime.heap_tid,
    );
    slot.score_bits
        .store(runtime.score.to_bits(), Ordering::Release);
    slot.approx_score_bits.store(0, Ordering::Release);
    slot.comparison_score_bits.store(0, Ordering::Release);
    slot.approx_rank_base_bits.store(0, Ordering::Release);
    slot.pending_count.store(0, Ordering::Release);
    slot.pending_index.store(0, Ordering::Release);
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

fn load_coordinator_snapshot(
    coordinator: &EcParallelCoordinatorState,
) -> EcParallelCoordinatorSnapshot {
    let flags = coordinator.flags.load(Ordering::Acquire);
    EcParallelCoordinatorSnapshot {
        flags,
        claimed_worker_slots: coordinator.claimed_worker_slots.load(Ordering::Acquire),
        published_result_slots: coordinator.published_result_slots.load(Ordering::Acquire),
        result_publish_generation: coordinator
            .result_publish_generation
            .load(Ordering::Acquire),
        selected_result_slot_index: (flags & EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID != 0)
            .then(|| {
                coordinator
                    .selected_result_slot_index
                    .load(Ordering::Acquire)
            }),
        selected_result_score: (flags & EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID != 0).then(
            || {
                f32::from_bits(
                    coordinator
                        .selected_result_score_bits
                        .load(Ordering::Acquire),
                )
            },
        ),
    }
}

fn load_coordinator_heap_snapshot(
    attachment: &ParallelScanAttachment,
) -> EcParallelCoordinatorHeapSnapshot {
    let heap_state = unsafe { &*attachment.heap_state };
    let live_entry_count = heap_state.live_entry_count.load(Ordering::Acquire);
    let root_slot_index = if live_entry_count == 0 {
        None
    } else {
        let root = unsafe { *attachment.heap_entries };
        (root != EC_PARALLEL_HEAP_ENTRY_INVALID).then_some(root)
    };

    EcParallelCoordinatorHeapSnapshot {
        live_entry_count,
        entry_capacity: attachment.heap_entry_count,
        heap_generation: heap_state.heap_generation.load(Ordering::Acquire),
        root_slot_index,
    }
}

fn result_slot_orders_before(
    lhs: &EcParallelCoordinatorResultSlotSnapshot,
    rhs: &EcParallelCoordinatorResultSlotSnapshot,
) -> bool {
    lhs.runtime
        .score
        .total_cmp(&rhs.runtime.score)
        .then_with(|| lhs.slot_index.cmp(&rhs.slot_index))
        .is_lt()
}

fn coordinator_result_slot_snapshot_is_live(
    snapshot: &EcParallelCoordinatorResultSlotSnapshot,
    rescan_epoch: u32,
) -> bool {
    snapshot.observed_rescan_epoch == rescan_epoch
        && snapshot.flags & EC_PARALLEL_RESULT_SLOT_PUBLISHED != 0
        && snapshot.flags & EC_PARALLEL_RESULT_SLOT_SCORE_VALID != 0
        && snapshot.runtime.element_tid.is_valid()
}

unsafe fn coordinator_result_slot_worker_claim_is_live(
    attachment: &ParallelScanAttachment,
    slot_index: u32,
) -> bool {
    let Ok(worker_slot) = (unsafe { attachment.worker_slot(slot_index) }) else {
        return false;
    };
    let worker_slot_ref = unsafe { &*worker_slot };
    worker_slot_ref
        .observed_rescan_epoch
        .load(Ordering::Acquire)
        == attachment.rescan_epoch
        && worker_slot_ref.flags.load(Ordering::Acquire) == EC_PARALLEL_WORKER_SLOT_CLAIMED
}

unsafe fn coordinator_result_slot_snapshot_is_live_with_attachment(
    attachment: &ParallelScanAttachment,
    snapshot: &EcParallelCoordinatorResultSlotSnapshot,
) -> bool {
    coordinator_result_slot_snapshot_is_live(snapshot, attachment.rescan_epoch)
        && unsafe { coordinator_result_slot_worker_claim_is_live(attachment, snapshot.slot_index) }
}

unsafe fn select_best_parallel_scan_coordinator_result_slot_with_attachment(
    attachment: &ParallelScanAttachment,
) -> Result<Option<EcParallelCoordinatorResultSlotSnapshot>, &'static str> {
    let heap_snapshot = load_coordinator_heap_snapshot(attachment);
    let Some(slot_index) = heap_snapshot.root_slot_index else {
        return Ok(None);
    };
    let slot = unsafe { attachment.result_slot(slot_index) }?;
    let snapshot = load_coordinator_result_slot_snapshot(unsafe { &*slot });
    if !unsafe { coordinator_result_slot_snapshot_is_live_with_attachment(attachment, &snapshot) } {
        return Ok(None);
    }
    Ok(Some(snapshot))
}

unsafe fn heap_entry_slot_index(
    attachment: &ParallelScanAttachment,
    heap_index: u32,
) -> Result<u32, &'static str> {
    Ok(unsafe { *attachment.heap_entry(heap_index)? })
}

unsafe fn heap_entry_snapshot(
    attachment: &ParallelScanAttachment,
    heap_index: u32,
) -> Result<EcParallelCoordinatorResultSlotSnapshot, &'static str> {
    let slot_index = unsafe { heap_entry_slot_index(attachment, heap_index) }?;
    let slot = unsafe { attachment.result_slot(slot_index) }?;
    Ok(load_coordinator_result_slot_snapshot(unsafe { &*slot }))
}

unsafe fn slot_heap_index(
    attachment: &ParallelScanAttachment,
    slot_index: u32,
) -> Result<Option<u32>, &'static str> {
    let slot = unsafe { attachment.result_slot(slot_index) }?;
    let heap_index = unsafe { &*slot }.heap_index.load(Ordering::Acquire);
    Ok((heap_index != EC_PARALLEL_HEAP_ENTRY_INVALID).then_some(heap_index))
}

unsafe fn store_slot_heap_index(
    attachment: &ParallelScanAttachment,
    slot_index: u32,
    heap_index: u32,
) -> Result<(), &'static str> {
    let slot = unsafe { attachment.result_slot(slot_index) }?;
    unsafe { &*slot }
        .heap_index
        .store(heap_index, Ordering::Release);
    Ok(())
}

unsafe fn swap_heap_entries(
    attachment: &ParallelScanAttachment,
    lhs_index: u32,
    rhs_index: u32,
) -> Result<(), &'static str> {
    let lhs = unsafe { attachment.heap_entry(lhs_index) }?;
    let rhs = unsafe { attachment.heap_entry(rhs_index) }?;
    let lhs_slot_index = unsafe { *lhs };
    let rhs_slot_index = unsafe { *rhs };
    unsafe { std::ptr::swap(lhs, rhs) };
    unsafe { store_slot_heap_index(attachment, lhs_slot_index, rhs_index) }?;
    unsafe { store_slot_heap_index(attachment, rhs_slot_index, lhs_index) }?;
    Ok(())
}

unsafe fn sift_up_heap_entry(
    attachment: &ParallelScanAttachment,
    mut heap_index: u32,
) -> Result<(), &'static str> {
    while heap_index > 0 {
        let parent_index = (heap_index - 1) / 2;
        let child_snapshot = unsafe { heap_entry_snapshot(attachment, heap_index) }?;
        let parent_snapshot = unsafe { heap_entry_snapshot(attachment, parent_index) }?;
        if !result_slot_orders_before(&child_snapshot, &parent_snapshot) {
            break;
        }
        unsafe { swap_heap_entries(attachment, heap_index, parent_index) }?;
        heap_index = parent_index;
    }
    Ok(())
}

unsafe fn sift_down_heap_entry(
    attachment: &ParallelScanAttachment,
    mut heap_index: u32,
    live_entry_count: u32,
) -> Result<(), &'static str> {
    loop {
        let left_child = heap_index
            .checked_mul(2)
            .and_then(|index| index.checked_add(1))
            .expect("parallel heap child index should not overflow u32");
        if left_child >= live_entry_count {
            return Ok(());
        }

        let right_child = left_child + 1;
        let mut best_child = left_child;
        if right_child < live_entry_count {
            let left_snapshot = unsafe { heap_entry_snapshot(attachment, left_child) }?;
            let right_snapshot = unsafe { heap_entry_snapshot(attachment, right_child) }?;
            if result_slot_orders_before(&right_snapshot, &left_snapshot) {
                best_child = right_child;
            }
        }

        let current_snapshot = unsafe { heap_entry_snapshot(attachment, heap_index) }?;
        let child_snapshot = unsafe { heap_entry_snapshot(attachment, best_child) }?;
        if !result_slot_orders_before(&child_snapshot, &current_snapshot) {
            return Ok(());
        }

        unsafe { swap_heap_entries(attachment, heap_index, best_child) }?;
        heap_index = best_child;
    }
}

unsafe fn rebuild_parallel_scan_heap_with_attachment(
    attachment: &ParallelScanAttachment,
) -> Result<u32, &'static str> {
    for slot_index in 0..attachment.result_slot_count {
        unsafe {
            store_slot_heap_index(attachment, slot_index, EC_PARALLEL_HEAP_ENTRY_INVALID)?;
        }
    }
    for entry_index in 0..attachment.heap_entry_count {
        let entry = unsafe { attachment.heap_entry(entry_index) }?;
        unsafe {
            *entry = EC_PARALLEL_HEAP_ENTRY_INVALID;
        }
    }

    let mut live_entry_count = 0;
    for slot_index in 0..attachment.result_slot_count {
        let slot = unsafe { attachment.result_slot(slot_index) }?;
        let snapshot = load_coordinator_result_slot_snapshot(unsafe { &*slot });
        if !unsafe {
            coordinator_result_slot_snapshot_is_live_with_attachment(attachment, &snapshot)
        } {
            continue;
        }
        let entry = unsafe { attachment.heap_entry(live_entry_count) }?;
        unsafe {
            *entry = slot_index;
        }
        unsafe { store_slot_heap_index(attachment, slot_index, live_entry_count) }?;
        unsafe { sift_up_heap_entry(attachment, live_entry_count) }?;
        live_entry_count += 1;
    }

    let heap_state = unsafe { &*attachment.heap_state };
    heap_state
        .live_entry_count
        .store(live_entry_count, Ordering::Release);
    heap_state.heap_generation.fetch_add(1, Ordering::AcqRel);
    Ok(live_entry_count)
}

unsafe fn acquire_parallel_scan_heap_lock(
    attachment: &ParallelScanAttachment,
) -> ParallelScanHeapLockGuard {
    let lock = unsafe { &(*attachment.heap_state).mutex };
    while lock
        .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        std::hint::spin_loop();
    }
    ParallelScanHeapLockGuard { lock }
}

unsafe fn detach_parallel_scan_heap_entry_with_attachment(
    attachment: &ParallelScanAttachment,
    slot_index: u32,
) -> Result<(), &'static str> {
    let Some(mut heap_index) = (unsafe { slot_heap_index(attachment, slot_index) })? else {
        return Ok(());
    };
    let heap_state = unsafe { &*attachment.heap_state };
    let live_entry_count = heap_state.live_entry_count.load(Ordering::Acquire);
    if heap_index >= live_entry_count {
        unsafe { store_slot_heap_index(attachment, slot_index, EC_PARALLEL_HEAP_ENTRY_INVALID) }?;
        return Ok(());
    }

    let last_entry_index = live_entry_count - 1;
    let mut replacement_slot_index = None;
    if heap_index != last_entry_index {
        let moved_slot_index = unsafe { heap_entry_slot_index(attachment, last_entry_index) }?;
        let heap_entry = unsafe { attachment.heap_entry(heap_index) }?;
        unsafe {
            *heap_entry = moved_slot_index;
        }
        unsafe { store_slot_heap_index(attachment, moved_slot_index, heap_index) }?;
        replacement_slot_index = Some(moved_slot_index);
    }

    let last_entry = unsafe { attachment.heap_entry(last_entry_index) }?;
    unsafe {
        *last_entry = EC_PARALLEL_HEAP_ENTRY_INVALID;
    }
    unsafe { store_slot_heap_index(attachment, slot_index, EC_PARALLEL_HEAP_ENTRY_INVALID) }?;

    let new_live_entry_count = live_entry_count - 1;
    heap_state
        .live_entry_count
        .store(new_live_entry_count, Ordering::Release);
    heap_state.heap_generation.fetch_add(1, Ordering::AcqRel);

    if heap_index != last_entry_index && new_live_entry_count != 0 {
        unsafe { sift_up_heap_entry(attachment, heap_index) }?;
        if let Some(moved_slot_index) = replacement_slot_index {
            heap_index = (unsafe { slot_heap_index(attachment, moved_slot_index) })?
                .expect("replacement slot should stay present after heap detach");
            unsafe { sift_down_heap_entry(attachment, heap_index, new_live_entry_count) }?;
        }
    }

    Ok(())
}

unsafe fn upsert_parallel_scan_heap_entry_with_attachment(
    attachment: &ParallelScanAttachment,
    slot_index: u32,
) -> Result<(), &'static str> {
    let slot = unsafe { attachment.result_slot(slot_index) }?;
    let snapshot = load_coordinator_result_slot_snapshot(unsafe { &*slot });
    if !unsafe { coordinator_result_slot_snapshot_is_live_with_attachment(attachment, &snapshot) } {
        unsafe { detach_parallel_scan_heap_entry_with_attachment(attachment, slot_index) }?;
        return Ok(());
    }

    let heap_state = unsafe { &*attachment.heap_state };
    if let Some(mut heap_index) = (unsafe { slot_heap_index(attachment, slot_index) })? {
        unsafe { sift_up_heap_entry(attachment, heap_index) }?;
        heap_index = (unsafe { slot_heap_index(attachment, slot_index) })?
            .expect("slot should stay present after heap update");
        let live_entry_count = heap_state.live_entry_count.load(Ordering::Acquire);
        unsafe { sift_down_heap_entry(attachment, heap_index, live_entry_count) }?;
        heap_state.heap_generation.fetch_add(1, Ordering::AcqRel);
        return Ok(());
    }

    let live_entry_count = heap_state.live_entry_count.load(Ordering::Acquire);
    let entry = unsafe { attachment.heap_entry(live_entry_count) }?;
    unsafe {
        *entry = slot_index;
    }
    unsafe { store_slot_heap_index(attachment, slot_index, live_entry_count) }?;
    heap_state
        .live_entry_count
        .store(live_entry_count + 1, Ordering::Release);
    unsafe { sift_up_heap_entry(attachment, live_entry_count) }?;
    heap_state.heap_generation.fetch_add(1, Ordering::AcqRel);
    Ok(())
}

unsafe fn reap_parallel_scan_dead_root_slots_with_attachment(
    attachment: &ParallelScanAttachment,
) -> Result<(), &'static str> {
    loop {
        let heap_snapshot = load_coordinator_heap_snapshot(attachment);
        let Some(slot_index) = heap_snapshot.root_slot_index else {
            return Ok(());
        };
        let slot = unsafe { attachment.result_slot(slot_index) }?;
        let slot_ref = unsafe { &*slot };
        let snapshot = load_coordinator_result_slot_snapshot(slot_ref);
        if unsafe {
            coordinator_result_slot_snapshot_is_live_with_attachment(attachment, &snapshot)
        } {
            return Ok(());
        }

        unsafe { detach_parallel_scan_heap_entry_with_attachment(attachment, slot_index) }?;
        if snapshot.observed_rescan_epoch == attachment.rescan_epoch
            && snapshot.flags & EC_PARALLEL_RESULT_SLOT_PUBLISHED != 0
            && !unsafe { coordinator_result_slot_worker_claim_is_live(attachment, slot_index) }
        {
            reset_result_slot_runtime(slot_ref);
            slot_ref.flags.store(0, Ordering::Release);
            let coordinator = unsafe { &*attachment.coordinator };
            coordinator
                .published_result_slots
                .fetch_sub(1, Ordering::AcqRel);
            coordinator
                .result_publish_generation
                .fetch_add(1, Ordering::AcqRel);
        }
    }
}

fn refresh_coordinator_selected_result_fast_path_locked(
    attachment: &ParallelScanAttachment,
) -> Result<(), &'static str> {
    let coordinator = unsafe { &*attachment.coordinator };
    match unsafe { select_best_parallel_scan_coordinator_result_slot_with_attachment(attachment) }?
    {
        Some(selected) => {
            coordinator
                .selected_result_slot_index
                .store(selected.slot_index, Ordering::Release);
            coordinator
                .selected_result_score_bits
                .store(selected.runtime.score.to_bits(), Ordering::Release);
            coordinator.flags.fetch_or(
                EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                Ordering::AcqRel,
            );
        }
        None => {
            coordinator
                .selected_result_slot_index
                .store(u32::MAX, Ordering::Release);
            coordinator
                .selected_result_score_bits
                .store(0, Ordering::Release);
            coordinator.flags.fetch_and(
                !EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                Ordering::AcqRel,
            );
        }
    }
    Ok(())
}

unsafe fn reap_dead_parallel_scan_result_slots_with_attachment(
    attachment: &ParallelScanAttachment,
) -> Result<u32, &'static str> {
    let mut reaped = 0;

    for slot_index in 0..attachment.result_slot_count {
        let slot = unsafe { attachment.result_slot(slot_index) }?;
        let slot_ref = unsafe { &*slot };
        let snapshot = load_coordinator_result_slot_snapshot(slot_ref);
        if snapshot.observed_rescan_epoch != attachment.rescan_epoch
            || snapshot.flags & EC_PARALLEL_RESULT_SLOT_PUBLISHED == 0
        {
            continue;
        }
        if unsafe { coordinator_result_slot_worker_claim_is_live(attachment, slot_index) } {
            continue;
        }

        reset_result_slot_runtime(slot_ref);
        slot_ref.flags.store(0, Ordering::Release);
        reaped += 1;
    }

    if reaped != 0 {
        let coordinator = unsafe { &*attachment.coordinator };
        coordinator
            .published_result_slots
            .fetch_sub(reaped, Ordering::AcqRel);
        coordinator
            .result_publish_generation
            .fetch_add(reaped, Ordering::AcqRel);
    }

    Ok(reaped)
}

fn refresh_coordinator_selection_snapshot_locked(
    attachment: &ParallelScanAttachment,
) -> Result<(), &'static str> {
    unsafe { reap_dead_parallel_scan_result_slots_with_attachment(attachment) }?;
    unsafe { rebuild_parallel_scan_heap_with_attachment(attachment) }?;
    unsafe { reap_parallel_scan_dead_root_slots_with_attachment(attachment) }?;
    refresh_coordinator_selected_result_fast_path_locked(attachment)
}

fn refresh_coordinator_selection_snapshot(
    attachment: &ParallelScanAttachment,
) -> Result<(), &'static str> {
    let _heap_lock = unsafe { acquire_parallel_scan_heap_lock(attachment) };
    refresh_coordinator_selection_snapshot_locked(attachment)
}

fn load_coordinator_result_slot_snapshot(
    slot: &EcParallelCoordinatorResultSlot,
) -> EcParallelCoordinatorResultSlotSnapshot {
    let flags = slot.flags.load(Ordering::Acquire);
    EcParallelCoordinatorResultSlotSnapshot {
        flags,
        slot_index: slot.slot_index,
        observed_rescan_epoch: slot.observed_rescan_epoch.load(Ordering::Acquire),
        runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot {
            element_tid: load_parallel_item_pointer(
                &slot.element_block_number,
                &slot.element_offset_number,
            ),
            heap_tid: load_parallel_item_pointer(&slot.heap_block_number, &slot.heap_offset_number),
            score: f32::from_bits(slot.score_bits.load(Ordering::Acquire)),
            approx_score: (flags & EC_PARALLEL_RESULT_SLOT_APPROX_SCORE_VALID != 0)
                .then(|| f32::from_bits(slot.approx_score_bits.load(Ordering::Acquire))),
            comparison_score: (flags & EC_PARALLEL_RESULT_SLOT_COMPARISON_SCORE_VALID != 0)
                .then(|| f32::from_bits(slot.comparison_score_bits.load(Ordering::Acquire))),
            approx_rank_base: (flags & EC_PARALLEL_RESULT_SLOT_APPROX_RANK_VALID != 0).then(|| {
                i32::from_ne_bytes(
                    slot.approx_rank_base_bits
                        .load(Ordering::Acquire)
                        .to_ne_bytes(),
                )
            }),
            pending_count: slot.pending_count.load(Ordering::Acquire),
            pending_index: slot.pending_index.load(Ordering::Acquire),
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
        heap_bytes: ec_parallel_scan_heap_size(),
        heap_entry_bytes: ec_parallel_scan_heap_entry_size(),
        result_slot_bytes: ec_parallel_scan_result_slot_size(),
        worker_slot_bytes: ec_parallel_scan_worker_slot_size(),
        heap_entry_count: ec_parallel_scan_heap_entry_capacity_for(worker_slot_count),
        result_slot_count: ec_parallel_scan_result_slot_capacity_for(worker_slot_count),
        worker_slot_count,
        reserved_worker_slots: 0,
        reserved0: 0,
        rescan_epoch: 0,
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
    if state_ref.heap_bytes < ec_parallel_scan_heap_size() {
        return Err("AM-private parallel scan heap size was smaller than the shared header");
    }
    if state_ref.heap_entry_bytes < ec_parallel_scan_heap_entry_size() {
        return Err("AM-private parallel scan heap entry size was smaller than the shared entry");
    }
    if state_ref.result_slot_bytes < ec_parallel_scan_result_slot_size() {
        return Err("AM-private parallel scan result-slot size was smaller than the shared header");
    }
    if state_ref.worker_slot_bytes < ec_parallel_scan_worker_slot_size() {
        return Err("AM-private parallel worker slot size was smaller than the shared header");
    }
    if state_ref.result_slot_count < state_ref.worker_slot_count {
        return Err(
            "AM-private parallel scan result-slot capacity was smaller than the worker-slot count",
        );
    }
    let minimum_descriptor_bytes =
        ec_parallel_scan_descriptor_size_for(state_ref.worker_slot_count);
    if state_ref.descriptor_bytes < minimum_descriptor_bytes {
        return Err("AM-private parallel scan descriptor size was smaller than the shared layout");
    }

    Ok(ParallelScanAttachment {
        state,
        coordinator: unsafe { coordinator_ptr(state) },
        heap_state: unsafe { heap_state_ptr(state) },
        result_slots: unsafe { result_slots_ptr(state) },
        heap_entries: unsafe { heap_entries_ptr(state) },
        worker_slots: unsafe { worker_slots_ptr(state) },
        descriptor_bytes: state_ref.descriptor_bytes,
        heap_entry_count: state_ref.heap_entry_count,
        result_slot_count: state_ref.result_slot_count,
        worker_slot_count: state_ref.worker_slot_count,
        heap_entry_bytes: state_ref.heap_entry_bytes,
        result_slot_bytes: state_ref.result_slot_bytes,
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

unsafe fn clear_parallel_scan_result_slot_with_attachment(
    attachment: &ParallelScanAttachment,
    slot_index: u32,
    rescan_epoch: u32,
    refresh_selection_snapshot: bool,
) -> Result<bool, &'static str> {
    let _heap_lock = unsafe { acquire_parallel_scan_heap_lock(attachment) };
    clear_parallel_scan_result_slot_locked(
        attachment,
        slot_index,
        rescan_epoch,
        refresh_selection_snapshot,
    )
}

unsafe fn clear_parallel_scan_result_slot_locked(
    attachment: &ParallelScanAttachment,
    slot_index: u32,
    rescan_epoch: u32,
    refresh_selection_snapshot: bool,
) -> Result<bool, &'static str> {
    let worker_slot = unsafe { attachment.worker_slot(slot_index) }?;
    let worker_slot_ref = unsafe { &*worker_slot };
    if worker_slot_ref
        .observed_rescan_epoch
        .load(Ordering::Acquire)
        != rescan_epoch
    {
        return Ok(false);
    }
    if worker_slot_ref.flags.load(Ordering::Acquire) != EC_PARALLEL_WORKER_SLOT_CLAIMED {
        return Ok(false);
    }

    let result_slot = unsafe { attachment.result_slot(slot_index) }?;
    let result_slot_ref = unsafe { &*result_slot };
    if result_slot_ref
        .observed_rescan_epoch
        .load(Ordering::Acquire)
        != rescan_epoch
    {
        return Ok(false);
    }

    let flags = result_slot_ref.flags.load(Ordering::Acquire);
    if flags & EC_PARALLEL_RESULT_SLOT_PUBLISHED == 0 {
        reset_result_slot_runtime(result_slot_ref);
        return Ok(false);
    }

    unsafe { detach_parallel_scan_heap_entry_with_attachment(attachment, slot_index) }?;
    reset_result_slot_runtime(result_slot_ref);
    result_slot_ref.flags.store(0, Ordering::Release);
    let coordinator = unsafe { &*attachment.coordinator };
    coordinator
        .published_result_slots
        .fetch_sub(1, Ordering::AcqRel);
    coordinator
        .result_publish_generation
        .fetch_add(1, Ordering::AcqRel);
    if refresh_selection_snapshot {
        refresh_coordinator_selection_snapshot_locked(attachment)?;
    }
    Ok(true)
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

    unsafe {
        clear_parallel_scan_result_slot_with_attachment(&attachment, slot_index, rescan_epoch, true)
    }?;
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

pub(crate) unsafe fn publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
    state: *mut EcParallelScanState,
    slot_index: u32,
    rescan_epoch: u32,
    snapshot: EcParallelCoordinatorResultSlotRuntimeSnapshot,
) -> Result<bool, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    let _heap_lock = unsafe { acquire_parallel_scan_heap_lock(&attachment) };
    let worker_slot = unsafe { attachment.worker_slot(slot_index) }?;
    let worker_slot_ref = unsafe { &*worker_slot };
    if worker_slot_ref
        .observed_rescan_epoch
        .load(Ordering::Acquire)
        != rescan_epoch
    {
        return Ok(false);
    }
    if worker_slot_ref.flags.load(Ordering::Acquire) != EC_PARALLEL_WORKER_SLOT_CLAIMED {
        return Ok(false);
    }

    let result_slot = unsafe { attachment.result_slot(slot_index) }?;
    let result_slot_ref = unsafe { &*result_slot };
    if result_slot_ref
        .observed_rescan_epoch
        .load(Ordering::Acquire)
        != rescan_epoch
    {
        return Ok(false);
    }

    let prior_flags = result_slot_ref.flags.load(Ordering::Acquire);
    store_parallel_item_pointer(
        &result_slot_ref.element_block_number,
        &result_slot_ref.element_offset_number,
        snapshot.element_tid,
    );
    store_parallel_item_pointer(
        &result_slot_ref.heap_block_number,
        &result_slot_ref.heap_offset_number,
        snapshot.heap_tid,
    );
    result_slot_ref
        .score_bits
        .store(snapshot.score.to_bits(), Ordering::Release);
    result_slot_ref.approx_score_bits.store(
        snapshot.approx_score.unwrap_or_default().to_bits(),
        Ordering::Release,
    );
    result_slot_ref.comparison_score_bits.store(
        snapshot.comparison_score.unwrap_or_default().to_bits(),
        Ordering::Release,
    );
    result_slot_ref.approx_rank_base_bits.store(
        u32::from_ne_bytes(snapshot.approx_rank_base.unwrap_or_default().to_ne_bytes()),
        Ordering::Release,
    );
    result_slot_ref
        .pending_count
        .store(snapshot.pending_count, Ordering::Release);
    result_slot_ref
        .pending_index
        .store(snapshot.pending_index, Ordering::Release);

    let mut flags = EC_PARALLEL_RESULT_SLOT_PUBLISHED;
    if snapshot.score != 0.0 || snapshot.element_tid.is_valid() {
        flags |= EC_PARALLEL_RESULT_SLOT_SCORE_VALID;
    }
    if snapshot.approx_score.is_some() {
        flags |= EC_PARALLEL_RESULT_SLOT_APPROX_SCORE_VALID;
    }
    if snapshot.comparison_score.is_some() {
        flags |= EC_PARALLEL_RESULT_SLOT_COMPARISON_SCORE_VALID;
    }
    if snapshot.approx_rank_base.is_some() {
        flags |= EC_PARALLEL_RESULT_SLOT_APPROX_RANK_VALID;
    }
    if snapshot.heap_tid.is_valid() {
        flags |= EC_PARALLEL_RESULT_SLOT_HEAP_TID_VALID;
    }
    result_slot_ref.flags.store(flags, Ordering::Release);

    let coordinator = unsafe { &*attachment.coordinator };
    if prior_flags & EC_PARALLEL_RESULT_SLOT_PUBLISHED == 0 {
        coordinator
            .published_result_slots
            .fetch_add(1, Ordering::AcqRel);
    }
    coordinator
        .result_publish_generation
        .fetch_add(1, Ordering::AcqRel);
    unsafe { upsert_parallel_scan_heap_entry_with_attachment(&attachment, slot_index) }?;
    unsafe { reap_parallel_scan_dead_root_slots_with_attachment(&attachment) }?;
    refresh_coordinator_selected_result_fast_path_locked(&attachment)?;
    Ok(true)
}

pub(crate) unsafe fn clear_parallel_scan_coordinator_result_slot_runtime_snapshot(
    state: *mut EcParallelScanState,
    slot_index: u32,
    rescan_epoch: u32,
) -> Result<bool, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    unsafe {
        clear_parallel_scan_result_slot_with_attachment(&attachment, slot_index, rescan_epoch, true)
    }
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

pub(crate) unsafe fn read_parallel_scan_coordinator_snapshot(
    state: *mut EcParallelScanState,
) -> Result<EcParallelCoordinatorSnapshot, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    Ok(load_coordinator_snapshot(unsafe {
        &*attachment.coordinator
    }))
}

pub(crate) unsafe fn read_parallel_scan_coordinator_heap_snapshot(
    state: *mut EcParallelScanState,
) -> Result<EcParallelCoordinatorHeapSnapshot, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    let _heap_lock = unsafe { acquire_parallel_scan_heap_lock(&attachment) };
    Ok(load_coordinator_heap_snapshot(&attachment))
}

pub(crate) unsafe fn read_parallel_scan_coordinator_result_slot_snapshot(
    state: *mut EcParallelScanState,
    slot_index: u32,
) -> Result<EcParallelCoordinatorResultSlotSnapshot, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    let slot = unsafe { attachment.result_slot(slot_index) }?;
    Ok(load_coordinator_result_slot_snapshot(unsafe { &*slot }))
}

pub(crate) unsafe fn read_parallel_scan_selected_result_slot_snapshot(
    state: *mut EcParallelScanState,
) -> Result<Option<EcParallelCoordinatorResultSelection>, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    for _ in 0..2 {
        let coordinator = load_coordinator_snapshot(unsafe { &*attachment.coordinator });
        let Some(slot_index) = coordinator.selected_result_slot_index else {
            return Ok(None);
        };
        let slot = unsafe { attachment.result_slot(slot_index) }?;
        let selected_result_slot = load_coordinator_result_slot_snapshot(unsafe { &*slot });
        if unsafe {
            coordinator_result_slot_snapshot_is_live_with_attachment(
                &attachment,
                &selected_result_slot,
            )
        } {
            return Ok(Some(EcParallelCoordinatorResultSelection {
                coordinator,
                selected_result_slot,
            }));
        }
        let _heap_lock = unsafe { acquire_parallel_scan_heap_lock(&attachment) };
        refresh_coordinator_selection_snapshot_locked(&attachment)?;
    }

    Ok(None)
}

pub(crate) unsafe fn take_parallel_scan_selected_result_slot_snapshot(
    state: *mut EcParallelScanState,
) -> Result<Option<EcParallelCoordinatorResultSelection>, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    for _ in 0..2 {
        let _heap_lock = unsafe { acquire_parallel_scan_heap_lock(&attachment) };
        let coordinator = load_coordinator_snapshot(unsafe { &*attachment.coordinator });
        let Some(slot_index) = coordinator.selected_result_slot_index else {
            return Ok(None);
        };
        let slot = unsafe { attachment.result_slot(slot_index) }?;
        let selected_result_slot = load_coordinator_result_slot_snapshot(unsafe { &*slot });
        if !unsafe {
            coordinator_result_slot_snapshot_is_live_with_attachment(
                &attachment,
                &selected_result_slot,
            )
        } {
            refresh_coordinator_selection_snapshot_locked(&attachment)?;
            continue;
        }
        if !unsafe {
            clear_parallel_scan_result_slot_locked(
                &attachment,
                slot_index,
                attachment.rescan_epoch,
                false,
            )
        }? {
            refresh_coordinator_selection_snapshot_locked(&attachment)?;
            continue;
        }
        unsafe { reap_parallel_scan_dead_root_slots_with_attachment(&attachment) }?;
        refresh_coordinator_selected_result_fast_path_locked(&attachment)?;

        return Ok(Some(EcParallelCoordinatorResultSelection {
            coordinator,
            selected_result_slot,
        }));
    }

    Ok(None)
}

pub(crate) unsafe fn select_best_parallel_scan_coordinator_result_slot(
    state: *mut EcParallelScanState,
) -> Result<Option<EcParallelCoordinatorResultSelection>, &'static str> {
    let attachment = unsafe { validate_parallel_scan_state(state) }?;
    let _heap_lock = unsafe { acquire_parallel_scan_heap_lock(&attachment) };
    let coordinator = load_coordinator_snapshot(unsafe { &*attachment.coordinator });
    let selected =
        unsafe { select_best_parallel_scan_coordinator_result_slot_with_attachment(&attachment) }?;

    Ok(selected.map(
        |selected_result_slot| EcParallelCoordinatorResultSelection {
            coordinator,
            selected_result_slot,
        },
    ))
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
            ec_parallel_scan_heap_size() % size_of::<usize>(),
            0,
            "parallel scan heap header should stay MAXALIGN-sized"
        );
        assert_eq!(
            ec_parallel_scan_result_slot_size() % size_of::<usize>(),
            0,
            "parallel scan result-slot header should stay MAXALIGN-sized"
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
            + ec_parallel_scan_heap_size()
            + ec_parallel_scan_heap_entry_size() * TEST_WORKER_SLOT_COUNT as pg_sys::Size
            + ec_parallel_scan_result_slot_size() * TEST_WORKER_SLOT_COUNT as pg_sys::Size
            + ec_parallel_scan_worker_slot_size() * TEST_WORKER_SLOT_COUNT as pg_sys::Size;

        assert!(
            descriptor_bytes >= minimum,
            "descriptor size should cover the shared state, coordinator, heap, result slots, and worker slots"
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
            attachment.heap_entry_count, TEST_WORKER_SLOT_COUNT,
            "attachment should reserve one shared heap entry per worker slot in this checkpoint"
        );
        assert_eq!(
            attachment.result_slot_count, TEST_WORKER_SLOT_COUNT,
            "attachment should reserve one staged result slot per worker slot in this checkpoint"
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
        assert_eq!(
            unsafe { &*attachment.coordinator }
                .published_result_slots
                .load(Ordering::Acquire),
            0,
            "freshly initialized coordinator state should start with no published result slots"
        );
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_heap_snapshot(attachment.state) }
                .expect("coordinator heap snapshot should read back"),
            EcParallelCoordinatorHeapSnapshot {
                live_entry_count: 0,
                entry_capacity: TEST_WORKER_SLOT_COUNT,
                heap_generation: 0,
                root_slot_index: None,
            },
            "freshly initialized heap state should start empty"
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
            assert_eq!(
                unsafe {
                    read_parallel_scan_coordinator_result_slot_snapshot(
                        attachment.state,
                        slot_index,
                    )
                }
                .expect("coordinator result-slot snapshot should read back"),
                EcParallelCoordinatorResultSlotSnapshot {
                    flags: 0,
                    slot_index,
                    observed_rescan_epoch: 0,
                    runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot::idle(),
                },
                "coordinator result slots should start empty for the active epoch"
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
    fn publish_parallel_scan_coordinator_result_slot_runtime_snapshot_records_live_state() {
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
        let runtime = EcParallelCoordinatorResultSlotRuntimeSnapshot {
            element_tid: EcParallelItemPointer {
                block_number: 42,
                offset_number: 7,
            },
            heap_tid: EcParallelItemPointer {
                block_number: 88,
                offset_number: 3,
            },
            score: -9.5,
            approx_score: Some(-8.0),
            comparison_score: Some(-9.25),
            approx_rank_base: Some(4),
            pending_count: 3,
            pending_index: 1,
        };

        assert!(
            unsafe {
                publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                    runtime,
                )
            }
            .expect("publish should succeed"),
            "publishing should update the coordinator-owned result slot for the active epoch"
        );
        assert_eq!(
            unsafe {
                read_parallel_scan_coordinator_result_slot_snapshot(attachment.state, slot_index)
            }
            .expect("coordinator result-slot snapshot should read back"),
            EcParallelCoordinatorResultSlotSnapshot {
                flags: EC_PARALLEL_RESULT_SLOT_PUBLISHED
                    | EC_PARALLEL_RESULT_SLOT_SCORE_VALID
                    | EC_PARALLEL_RESULT_SLOT_APPROX_SCORE_VALID
                    | EC_PARALLEL_RESULT_SLOT_COMPARISON_SCORE_VALID
                    | EC_PARALLEL_RESULT_SLOT_APPROX_RANK_VALID
                    | EC_PARALLEL_RESULT_SLOT_HEAP_TID_VALID,
                slot_index,
                observed_rescan_epoch: attachment.rescan_epoch,
                runtime,
            },
            "published coordinator result state should round-trip through the shared slot"
        );
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should read back"),
            EcParallelCoordinatorSnapshot {
                flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                claimed_worker_slots: 1,
                published_result_slots: 1,
                result_publish_generation: 1,
                selected_result_slot_index: Some(slot_index),
                selected_result_score: Some(-9.5),
            },
            "publishing a first result slot should update the coordinator counters and selected-result snapshot"
        );
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_heap_snapshot(attachment.state) }
                .expect("coordinator heap snapshot should read back"),
            EcParallelCoordinatorHeapSnapshot {
                live_entry_count: 1,
                entry_capacity: TEST_WORKER_SLOT_COUNT,
                heap_generation: 1,
                root_slot_index: Some(slot_index),
            },
            "publishing a first result slot should seed the shared heap root from the owning slot"
        );
    }

    #[test]
    fn select_best_parallel_scan_coordinator_result_slot_returns_none_without_live_results() {
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
            unsafe { select_best_parallel_scan_coordinator_result_slot(attachment.state) }
                .expect("selection should succeed"),
            None,
            "coordinator selection should stay empty until at least one worker publishes a live result"
        );
    }

    #[test]
    fn select_best_parallel_scan_coordinator_result_slot_prefers_lowest_score() {
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
        let first_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("first claim should succeed");
        let second_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("second claim should succeed");

        unsafe {
            publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                attachment.state,
                first_slot,
                attachment.rescan_epoch,
                EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: EcParallelItemPointer {
                        block_number: 10,
                        offset_number: 1,
                    },
                    heap_tid: EcParallelItemPointer::INVALID,
                    score: -4.0,
                    approx_score: None,
                    comparison_score: None,
                    approx_rank_base: None,
                    pending_count: 1,
                    pending_index: 0,
                },
            )
        }
        .expect("first publish should succeed");
        unsafe {
            publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                attachment.state,
                second_slot,
                attachment.rescan_epoch,
                EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: EcParallelItemPointer {
                        block_number: 11,
                        offset_number: 1,
                    },
                    heap_tid: EcParallelItemPointer::INVALID,
                    score: -6.5,
                    approx_score: None,
                    comparison_score: None,
                    approx_rank_base: None,
                    pending_count: 1,
                    pending_index: 0,
                },
            )
        }
        .expect("second publish should succeed");

        let selection =
            unsafe { select_best_parallel_scan_coordinator_result_slot(attachment.state) }
                .expect("selection should succeed")
                .expect("selection should surface the best published slot");
        assert_eq!(
            selection.coordinator,
            EcParallelCoordinatorSnapshot {
                flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                claimed_worker_slots: 2,
                published_result_slots: 2,
                result_publish_generation: 2,
                selected_result_slot_index: Some(second_slot),
                selected_result_score: Some(-6.5),
            },
            "selection should carry the current coordinator counters and selected-result snapshot too"
        );
        assert_eq!(
            selection.selected_result_slot.slot_index, second_slot,
            "selection should pick the lowest-score published slot"
        );
        assert_eq!(
            selection.selected_result_slot.runtime.score, -6.5,
            "selection should surface the selected slot's score"
        );
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_heap_snapshot(attachment.state) }
                .expect("coordinator heap snapshot should read back"),
            EcParallelCoordinatorHeapSnapshot {
                live_entry_count: 2,
                entry_capacity: TEST_WORKER_SLOT_COUNT,
                heap_generation: 2,
                root_slot_index: Some(second_slot),
            },
            "the shared heap root should mirror the lowest-score published staged result"
        );
    }

    #[test]
    fn select_best_parallel_scan_coordinator_result_slot_breaks_ties_by_slot_index() {
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
        let first_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("first claim should succeed");
        let second_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("second claim should succeed");

        for slot_index in [second_slot, first_slot] {
            unsafe {
                publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                    EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number: 20 + slot_index,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score: -5.0,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                )
            }
            .expect("publish should succeed");
        }

        let selection =
            unsafe { select_best_parallel_scan_coordinator_result_slot(attachment.state) }
                .expect("selection should succeed")
                .expect("selection should surface a published slot");
        assert_eq!(
            selection.selected_result_slot.slot_index,
            first_slot,
            "score ties should break toward the lower slot index for deterministic coordinator selection"
        );
        assert_eq!(
            selection.coordinator.selected_result_slot_index,
            Some(first_slot),
            "coordinator snapshot should carry the chosen staged result slot too"
        );
        assert_eq!(
            selection.coordinator.selected_result_score,
            Some(-5.0),
            "coordinator snapshot should carry the chosen staged result score too"
        );
    }

    #[test]
    fn publish_parallel_scan_coordinator_result_slot_runtime_snapshot_reheapifies_existing_slot() {
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
        let first_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("first claim should succeed");
        let second_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("second claim should succeed");

        for (slot_index, block_number, score) in [(first_slot, 71, -4.0), (second_slot, 72, -7.0)] {
            unsafe {
                publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                    EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                )
            }
            .expect("publish should succeed");
        }

        unsafe {
            publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                attachment.state,
                first_slot,
                attachment.rescan_epoch,
                EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: EcParallelItemPointer {
                        block_number: 71,
                        offset_number: 1,
                    },
                    heap_tid: EcParallelItemPointer::INVALID,
                    score: -9.0,
                    approx_score: None,
                    comparison_score: None,
                    approx_rank_base: None,
                    pending_count: 1,
                    pending_index: 0,
                },
            )
        }
        .expect("republish should succeed");

        assert_eq!(
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should read back after republish"),
            EcParallelCoordinatorSnapshot {
                flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                claimed_worker_slots: 2,
                published_result_slots: 2,
                result_publish_generation: 3,
                selected_result_slot_index: Some(first_slot),
                selected_result_score: Some(-9.0),
            },
            "republishing an existing slot with a lower score should retarget the coordinator fast path to that slot"
        );
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_heap_snapshot(attachment.state) }
                .expect("coordinator heap snapshot should read back after republish"),
            EcParallelCoordinatorHeapSnapshot {
                live_entry_count: 2,
                entry_capacity: TEST_WORKER_SLOT_COUNT,
                heap_generation: 3,
                root_slot_index: Some(first_slot),
            },
            "republishing a staged result should reheapify the slot in place instead of rebuilding the shared heap"
        );
    }

    #[test]
    fn read_parallel_scan_selected_result_slot_snapshot_reads_coordinator_fast_path() {
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
        let first_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("first claim should succeed");
        let second_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("second claim should succeed");

        for (slot_index, block_number, score) in [(first_slot, 41, -4.0), (second_slot, 42, -8.0)] {
            unsafe {
                publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                    EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                )
            }
            .expect("publish should succeed");
        }

        let selection =
            unsafe { read_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("direct read should succeed")
                .expect("direct read should surface the coordinator-selected slot");
        assert_eq!(
            selection.coordinator.selected_result_slot_index,
            Some(second_slot),
            "coordinator fast path should point at the lowest-score staged result slot"
        );
        assert_eq!(
            selection.coordinator.selected_result_score,
            Some(-8.0),
            "coordinator fast path should carry the staged result score"
        );
        assert_eq!(
            selection.selected_result_slot.slot_index, second_slot,
            "direct read should return the slot named by the coordinator snapshot"
        );
        assert_eq!(
            selection.selected_result_slot.runtime.score, -8.0,
            "direct read should return the chosen staged result score"
        );
    }

    #[test]
    fn read_parallel_scan_selected_result_slot_snapshot_returns_none_without_selection() {
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
            unsafe { read_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("direct read should succeed"),
            None,
            "direct read should stay empty until the coordinator snapshot names a live staged result"
        );
    }

    #[test]
    fn read_parallel_scan_selected_result_slot_snapshot_refreshes_past_unclaimed_slot() {
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
        let first_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("first claim should succeed");
        let second_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("second claim should succeed");

        for (slot_index, block_number, score) in [(first_slot, 71, -4.0), (second_slot, 72, -8.0)] {
            unsafe {
                publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                    EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                )
            }
            .expect("publish should succeed");
        }

        let second_worker_slot = unsafe { attachment.worker_slot(second_slot) }
            .expect("second worker slot should resolve");
        unsafe { &*second_worker_slot }
            .flags
            .store(EC_PARALLEL_WORKER_SLOT_FREE, Ordering::Release);
        unsafe { &*attachment.coordinator }
            .claimed_worker_slots
            .fetch_sub(1, Ordering::AcqRel);

        assert_eq!(
            unsafe { read_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("direct read should succeed after claim drop"),
            Some(EcParallelCoordinatorResultSelection {
                coordinator: EcParallelCoordinatorSnapshot {
                    flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                    claimed_worker_slots: 1,
                    published_result_slots: 1,
                    result_publish_generation: 3,
                    selected_result_slot_index: Some(first_slot),
                    selected_result_score: Some(-4.0),
                },
                selected_result_slot: EcParallelCoordinatorResultSlotSnapshot {
                    flags: EC_PARALLEL_RESULT_SLOT_PUBLISHED | EC_PARALLEL_RESULT_SLOT_SCORE_VALID,
                    slot_index: first_slot,
                    observed_rescan_epoch: attachment.rescan_epoch,
                    runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number: 71,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score: -4.0,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                },
            }),
            "direct read should refresh past a staged result whose worker claim is no longer live"
        );
        assert_eq!(
            unsafe {
                read_parallel_scan_coordinator_result_slot_snapshot(attachment.state, second_slot)
            }
            .expect("stale result slot should stay readable after refresh"),
            EcParallelCoordinatorResultSlotSnapshot {
                flags: 0,
                slot_index: second_slot,
                observed_rescan_epoch: attachment.rescan_epoch,
                runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot::idle(),
            },
            "refresh should reap the unclaimed staged result slot from the shared descriptor"
        );
    }

    #[test]
    fn take_parallel_scan_selected_result_slot_snapshot_returns_none_without_selection() {
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
            unsafe { take_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("take should succeed"),
            None,
            "taking should stay empty until the coordinator snapshot names a live staged result"
        );
    }

    #[test]
    fn take_parallel_scan_selected_result_slot_snapshot_clears_fast_path_when_claim_drops() {
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
        let slot_index =
            unsafe { claim_parallel_scan_worker_slot(&attachment) }.expect("claim should succeed");

        unsafe {
            publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                attachment.state,
                slot_index,
                attachment.rescan_epoch,
                EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: EcParallelItemPointer {
                        block_number: 81,
                        offset_number: 1,
                    },
                    heap_tid: EcParallelItemPointer::INVALID,
                    score: -9.0,
                    approx_score: None,
                    comparison_score: None,
                    approx_rank_base: None,
                    pending_count: 1,
                    pending_index: 0,
                },
            )
        }
        .expect("publish should succeed");

        let worker_slot =
            unsafe { attachment.worker_slot(slot_index) }.expect("worker slot should resolve");
        unsafe { &*worker_slot }
            .flags
            .store(EC_PARALLEL_WORKER_SLOT_FREE, Ordering::Release);
        unsafe { &*attachment.coordinator }
            .claimed_worker_slots
            .fetch_sub(1, Ordering::AcqRel);

        assert_eq!(
            unsafe { take_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("take should succeed after claim drop"),
            None,
            "take should refuse a staged result once its worker claim is no longer live"
        );
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should read back after claim drop"),
            EcParallelCoordinatorSnapshot {
                flags: 0,
                claimed_worker_slots: 0,
                published_result_slots: 0,
                result_publish_generation: 2,
                selected_result_slot_index: None,
                selected_result_score: None,
            },
            "refreshing past an unclaimed staged result should clear the coordinator fast path"
        );
        assert_eq!(
            unsafe {
                read_parallel_scan_coordinator_result_slot_snapshot(attachment.state, slot_index)
            }
            .expect("stale result slot should stay readable after take-side reap"),
            EcParallelCoordinatorResultSlotSnapshot {
                flags: 0,
                slot_index,
                observed_rescan_epoch: attachment.rescan_epoch,
                runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot::idle(),
            },
            "take should reap an unclaimed staged result slot instead of leaving it published"
        );
    }

    #[test]
    fn take_parallel_scan_selected_result_slot_snapshot_clears_selected_slot() {
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
        let slot_index =
            unsafe { claim_parallel_scan_worker_slot(&attachment) }.expect("claim should succeed");

        unsafe {
            publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                attachment.state,
                slot_index,
                attachment.rescan_epoch,
                EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: EcParallelItemPointer {
                        block_number: 51,
                        offset_number: 1,
                    },
                    heap_tid: EcParallelItemPointer::INVALID,
                    score: -9.0,
                    approx_score: None,
                    comparison_score: None,
                    approx_rank_base: None,
                    pending_count: 1,
                    pending_index: 0,
                },
            )
        }
        .expect("publish should succeed");

        assert_eq!(
            unsafe { take_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("take should succeed"),
            Some(EcParallelCoordinatorResultSelection {
                coordinator: EcParallelCoordinatorSnapshot {
                    flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                    claimed_worker_slots: 1,
                    published_result_slots: 1,
                    result_publish_generation: 1,
                    selected_result_slot_index: Some(slot_index),
                    selected_result_score: Some(-9.0),
                },
                selected_result_slot: EcParallelCoordinatorResultSlotSnapshot {
                    flags: EC_PARALLEL_RESULT_SLOT_PUBLISHED | EC_PARALLEL_RESULT_SLOT_SCORE_VALID,
                    slot_index,
                    observed_rescan_epoch: attachment.rescan_epoch,
                    runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number: 51,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score: -9.0,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                },
            }),
            "take should return the coordinator-selected staged result before clearing it"
        );
        assert_eq!(
            unsafe { read_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("direct read should succeed after take"),
            None,
            "taking the only staged result should clear the coordinator fast path"
        );
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should read back after take"),
            EcParallelCoordinatorSnapshot {
                flags: 0,
                claimed_worker_slots: 1,
                published_result_slots: 0,
                result_publish_generation: 2,
                selected_result_slot_index: None,
                selected_result_score: None,
            },
            "taking the selected staged result should clear the coordinator snapshot"
        );
    }

    #[test]
    fn take_parallel_scan_selected_result_slot_snapshot_refreshes_next_best_slot() {
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
        let first_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("first claim should succeed");
        let second_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("second claim should succeed");

        for (slot_index, block_number, score) in [(first_slot, 61, -4.0), (second_slot, 62, -10.0)]
        {
            unsafe {
                publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                    EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                )
            }
            .expect("publish should succeed");
        }

        let taken = unsafe { take_parallel_scan_selected_result_slot_snapshot(attachment.state) }
            .expect("take should succeed")
            .expect("take should return the current selected staged result");
        assert_eq!(
            taken.selected_result_slot.slot_index, second_slot,
            "take should consume the current lowest-score staged result"
        );
        assert_eq!(
            unsafe { read_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("direct read should succeed after take"),
            Some(EcParallelCoordinatorResultSelection {
                coordinator: EcParallelCoordinatorSnapshot {
                    flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                    claimed_worker_slots: 2,
                    published_result_slots: 1,
                    result_publish_generation: 3,
                    selected_result_slot_index: Some(first_slot),
                    selected_result_score: Some(-4.0),
                },
                selected_result_slot: EcParallelCoordinatorResultSlotSnapshot {
                    flags: EC_PARALLEL_RESULT_SLOT_PUBLISHED
                        | EC_PARALLEL_RESULT_SLOT_SCORE_VALID,
                    slot_index: first_slot,
                    observed_rescan_epoch: attachment.rescan_epoch,
                    runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number: 61,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score: -4.0,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                },
            }),
            "taking the current selected staged result should refresh the coordinator fast path to the next best slot"
        );
    }

    #[test]
    fn clear_parallel_scan_coordinator_result_slot_runtime_snapshot_refreshes_selected_result() {
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
        let first_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("first claim should succeed");
        let second_slot = unsafe { claim_parallel_scan_worker_slot(&attachment) }
            .expect("second claim should succeed");

        for (slot_index, block_number, score) in [(first_slot, 31, -4.0), (second_slot, 32, -7.0)] {
            unsafe {
                publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                    EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                )
            }
            .expect("publish should succeed");
        }

        assert_eq!(
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should read back after publish"),
            EcParallelCoordinatorSnapshot {
                flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                claimed_worker_slots: 2,
                published_result_slots: 2,
                result_publish_generation: 2,
                selected_result_slot_index: Some(second_slot),
                selected_result_score: Some(-7.0),
            },
            "coordinator snapshot should point at the lowest-score staged result before clear"
        );

        assert!(
            unsafe {
                clear_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    second_slot,
                    attachment.rescan_epoch,
                )
            }
            .expect("clear should succeed"),
            "clearing the currently selected slot should report the mutation"
        );

        assert_eq!(
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should read back after clear"),
            EcParallelCoordinatorSnapshot {
                flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                claimed_worker_slots: 2,
                published_result_slots: 1,
                result_publish_generation: 3,
                selected_result_slot_index: Some(first_slot),
                selected_result_score: Some(-4.0),
            },
            "clearing the selected slot should refresh the coordinator snapshot to the remaining best staged result"
        );
        assert_eq!(
            unsafe { read_parallel_scan_selected_result_slot_snapshot(attachment.state) }
                .expect("direct read should succeed after clear")
                .expect("direct read should still see the remaining staged result"),
            EcParallelCoordinatorResultSelection {
                coordinator: EcParallelCoordinatorSnapshot {
                    flags: EC_PARALLEL_COORDINATOR_SELECTED_RESULT_VALID,
                    claimed_worker_slots: 2,
                    published_result_slots: 1,
                    result_publish_generation: 3,
                    selected_result_slot_index: Some(first_slot),
                    selected_result_score: Some(-4.0),
                },
                selected_result_slot: EcParallelCoordinatorResultSlotSnapshot {
                    flags: EC_PARALLEL_RESULT_SLOT_PUBLISHED
                        | EC_PARALLEL_RESULT_SLOT_SCORE_VALID,
                    slot_index: first_slot,
                    observed_rescan_epoch: attachment.rescan_epoch,
                    runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot {
                        element_tid: EcParallelItemPointer {
                            block_number: 31,
                            offset_number: 1,
                        },
                        heap_tid: EcParallelItemPointer::INVALID,
                        score: -4.0,
                        approx_score: None,
                        comparison_score: None,
                        approx_rank_base: None,
                        pending_count: 1,
                        pending_index: 0,
                    },
                },
            },
            "direct read should track the refreshed coordinator snapshot after clearing the selected slot"
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
    fn clear_parallel_scan_coordinator_result_slot_runtime_snapshot_resets_live_results() {
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
            .expect("claim should succeed before publish/clear");
        unsafe {
            publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                attachment.state,
                slot_index,
                attachment.rescan_epoch,
                EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: EcParallelItemPointer {
                        block_number: 17,
                        offset_number: 2,
                    },
                    heap_tid: EcParallelItemPointer::INVALID,
                    score: -3.25,
                    approx_score: None,
                    comparison_score: None,
                    approx_rank_base: None,
                    pending_count: 2,
                    pending_index: 0,
                },
            )
        }
        .expect("publish should succeed");

        assert!(
            unsafe {
                clear_parallel_scan_coordinator_result_slot_runtime_snapshot(
                    attachment.state,
                    slot_index,
                    attachment.rescan_epoch,
                )
            }
            .expect("clear should succeed"),
            "clearing a published coordinator result slot should report the mutation"
        );
        assert_eq!(
            unsafe {
                read_parallel_scan_coordinator_result_slot_snapshot(attachment.state, slot_index)
            }
            .expect("coordinator result-slot snapshot should stay readable"),
            EcParallelCoordinatorResultSlotSnapshot {
                flags: 0,
                slot_index,
                observed_rescan_epoch: attachment.rescan_epoch,
                runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot::idle(),
            },
            "clearing should return the staged coordinator result slot to its idle state"
        );
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should stay readable"),
            EcParallelCoordinatorSnapshot {
                flags: 0,
                claimed_worker_slots: 1,
                published_result_slots: 0,
                result_publish_generation: 2,
                selected_result_slot_index: None,
                selected_result_score: None,
            },
            "publishing then clearing should leave the coordinator with no staged results"
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
        unsafe {
            publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                attachment.state,
                slot_index,
                attachment.rescan_epoch,
                EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: EcParallelItemPointer {
                        block_number: 70,
                        offset_number: 11,
                    },
                    heap_tid: EcParallelItemPointer {
                        block_number: 71,
                        offset_number: 1,
                    },
                    score: -11.0,
                    approx_score: Some(-10.5),
                    comparison_score: None,
                    approx_rank_base: Some(0),
                    pending_count: 1,
                    pending_index: 0,
                },
            )
        }
        .expect("publish should succeed before release");

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
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should stay readable after release"),
            EcParallelCoordinatorSnapshot {
                flags: 0,
                claimed_worker_slots: 0,
                published_result_slots: 0,
                result_publish_generation: 2,
                selected_result_slot_index: None,
                selected_result_score: None,
            },
            "release should also clear the coordinator-owned result slot for the worker"
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
        assert_eq!(
            unsafe {
                read_parallel_scan_coordinator_result_slot_snapshot(attachment.state, slot_index)
            }
            .expect("coordinator result-slot snapshot should stay readable"),
            EcParallelCoordinatorResultSlotSnapshot {
                flags: 0,
                slot_index,
                observed_rescan_epoch: attachment.rescan_epoch,
                runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot::idle(),
            },
            "release should reset the coordinator-owned staged result slot too"
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
            unsafe { read_parallel_scan_coordinator_snapshot(attachment.state) }
                .expect("coordinator snapshot should read back after rescan"),
            EcParallelCoordinatorSnapshot {
                flags: 0,
                claimed_worker_slots: 0,
                published_result_slots: 0,
                result_publish_generation: 0,
                selected_result_slot_index: None,
                selected_result_score: None,
            },
            "rescan should also clear the staged coordinator-result counters"
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
        assert_eq!(
            unsafe { read_parallel_scan_coordinator_result_slot_snapshot(attachment.state, 1) }
                .expect("coordinator result-slot snapshot should read back after rescan"),
            EcParallelCoordinatorResultSlotSnapshot {
                flags: 0,
                slot_index: 1,
                observed_rescan_epoch: 1,
                runtime: EcParallelCoordinatorResultSlotRuntimeSnapshot::idle(),
            },
            "rescan should reset staged coordinator result slots to the fresh-epoch idle state"
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
