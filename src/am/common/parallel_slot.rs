use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) const EC_PARALLEL_WORKER_SLOT_FREE: u32 = 0;
pub(crate) const EC_PARALLEL_WORKER_SLOT_CLAIMED: u32 = 1;
pub(crate) const EC_PARALLEL_WORKER_SLOT_RELEASING: u32 = 2;
pub(crate) const EC_PARALLEL_WORKER_SLOT_PUBLISHING: u32 = 3;

pub(crate) const EC_PARALLEL_WORKER_PHASE_IDLE: u32 = 0;
pub(crate) const EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL: u32 = 1;
pub(crate) const EC_PARALLEL_WORKER_PHASE_LINEAR_FALLBACK: u32 = 2;
pub(crate) const EC_PARALLEL_WORKER_PHASE_EXHAUSTED: u32 = 3;

pub(crate) trait ParallelSlotAtomic {
    fn load_acquire(&self) -> u32;
    fn store_release(&self, value: u32);
    fn compare_exchange_acqrel_acquire(&self, current: u32, new: u32) -> bool;
    fn spin_wait(&self) {
        std::hint::spin_loop();
    }
}

impl ParallelSlotAtomic for AtomicU32 {
    fn load_acquire(&self) -> u32 {
        self.load(Ordering::Acquire)
    }

    fn store_release(&self, value: u32) {
        self.store(value, Ordering::Release);
    }

    fn compare_exchange_acqrel_acquire(&self, current: u32, new: u32) -> bool {
        self.compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
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
    pub(crate) const fn idle() -> Self {
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

#[derive(Debug)]
pub(crate) struct EcParallelWorkerSlotFields<'a, A: ParallelSlotAtomic> {
    pub(crate) flags: &'a A,
    pub(crate) slot_index: u32,
    pub(crate) observed_rescan_epoch: &'a A,
    pub(crate) execution_phase: &'a A,
    pub(crate) scan_dimensions: &'a A,
    pub(crate) bootstrap_frontier_limit: &'a A,
    pub(crate) visible_frontier_len: &'a A,
    pub(crate) scheduler_frontier_len: &'a A,
    pub(crate) visited_count: &'a A,
    pub(crate) emitted_count: &'a A,
    pub(crate) active_result_pending_count: &'a A,
    pub(crate) active_result_has_current: &'a A,
}

impl<'a, A: ParallelSlotAtomic> Copy for EcParallelWorkerSlotFields<'a, A> {}

impl<'a, A: ParallelSlotAtomic> Clone for EcParallelWorkerSlotFields<'a, A> {
    fn clone(&self) -> Self {
        *self
    }
}

pub(crate) fn reset_worker_slot_runtime<A: ParallelSlotAtomic>(
    slot: EcParallelWorkerSlotFields<'_, A>,
) {
    let runtime = EcParallelWorkerSlotRuntimeSnapshot::idle();
    slot.execution_phase.store_release(runtime.execution_phase);
    slot.scan_dimensions.store_release(runtime.scan_dimensions);
    slot.bootstrap_frontier_limit
        .store_release(runtime.bootstrap_frontier_limit);
    slot.visible_frontier_len
        .store_release(runtime.visible_frontier_len);
    slot.scheduler_frontier_len
        .store_release(runtime.scheduler_frontier_len);
    slot.visited_count.store_release(runtime.visited_count);
    slot.emitted_count.store_release(runtime.emitted_count);
    slot.active_result_pending_count
        .store_release(runtime.active_result_pending_count);
    slot.active_result_has_current
        .store_release(u32::from(runtime.active_result_has_current));
}

pub(crate) fn load_worker_slot_snapshot<A: ParallelSlotAtomic>(
    slot: EcParallelWorkerSlotFields<'_, A>,
) -> EcParallelWorkerSlotSnapshot {
    EcParallelWorkerSlotSnapshot {
        flags: slot.flags.load_acquire(),
        slot_index: slot.slot_index,
        observed_rescan_epoch: slot.observed_rescan_epoch.load_acquire(),
        runtime: EcParallelWorkerSlotRuntimeSnapshot {
            execution_phase: slot.execution_phase.load_acquire(),
            scan_dimensions: slot.scan_dimensions.load_acquire(),
            bootstrap_frontier_limit: slot.bootstrap_frontier_limit.load_acquire(),
            visible_frontier_len: slot.visible_frontier_len.load_acquire(),
            scheduler_frontier_len: slot.scheduler_frontier_len.load_acquire(),
            visited_count: slot.visited_count.load_acquire(),
            emitted_count: slot.emitted_count.load_acquire(),
            active_result_pending_count: slot.active_result_pending_count.load_acquire(),
            active_result_has_current: slot.active_result_has_current.load_acquire() != 0,
        },
    }
}

pub(crate) fn try_claim_worker_slot<A: ParallelSlotAtomic>(
    slot: EcParallelWorkerSlotFields<'_, A>,
    rescan_epoch: u32,
) -> bool {
    if slot.observed_rescan_epoch.load_acquire() != rescan_epoch {
        return false;
    }
    if slot.flags.compare_exchange_acqrel_acquire(
        EC_PARALLEL_WORKER_SLOT_FREE,
        EC_PARALLEL_WORKER_SLOT_CLAIMED,
    ) {
        reset_worker_slot_runtime(slot);
        return true;
    }
    false
}

pub(crate) fn publish_worker_slot_runtime_snapshot<A: ParallelSlotAtomic>(
    slot: EcParallelWorkerSlotFields<'_, A>,
    rescan_epoch: u32,
    snapshot: EcParallelWorkerSlotRuntimeSnapshot,
) -> bool {
    if slot.observed_rescan_epoch.load_acquire() != rescan_epoch {
        return false;
    }
    if !slot.flags.compare_exchange_acqrel_acquire(
        EC_PARALLEL_WORKER_SLOT_CLAIMED,
        EC_PARALLEL_WORKER_SLOT_PUBLISHING,
    ) {
        return false;
    }

    slot.execution_phase.store_release(snapshot.execution_phase);
    slot.scan_dimensions.store_release(snapshot.scan_dimensions);
    slot.bootstrap_frontier_limit
        .store_release(snapshot.bootstrap_frontier_limit);
    slot.visible_frontier_len
        .store_release(snapshot.visible_frontier_len);
    slot.scheduler_frontier_len
        .store_release(snapshot.scheduler_frontier_len);
    slot.visited_count.store_release(snapshot.visited_count);
    slot.emitted_count.store_release(snapshot.emitted_count);
    slot.active_result_pending_count
        .store_release(snapshot.active_result_pending_count);
    slot.active_result_has_current
        .store_release(u32::from(snapshot.active_result_has_current));

    let still_current = slot.observed_rescan_epoch.load_acquire() == rescan_epoch;
    if still_current {
        slot.flags.store_release(EC_PARALLEL_WORKER_SLOT_CLAIMED);
    } else {
        reset_worker_slot_runtime(slot);
        slot.flags.store_release(EC_PARALLEL_WORKER_SLOT_FREE);
    }
    still_current
}

pub(crate) fn release_worker_slot<A: ParallelSlotAtomic>(
    slot: EcParallelWorkerSlotFields<'_, A>,
    rescan_epoch: u32,
) -> bool {
    if slot.observed_rescan_epoch.load_acquire() != rescan_epoch {
        return false;
    }

    loop {
        match slot.flags.load_acquire() {
            EC_PARALLEL_WORKER_SLOT_CLAIMED => {
                if slot.flags.compare_exchange_acqrel_acquire(
                    EC_PARALLEL_WORKER_SLOT_CLAIMED,
                    EC_PARALLEL_WORKER_SLOT_RELEASING,
                ) {
                    break;
                }
            }
            EC_PARALLEL_WORKER_SLOT_PUBLISHING => {
                slot.flags.spin_wait();
            }
            _ => return false,
        }
    }

    reset_worker_slot_runtime(slot);
    slot.flags.store_release(EC_PARALLEL_WORKER_SLOT_FREE);
    true
}
