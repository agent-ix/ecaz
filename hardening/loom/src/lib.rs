#![allow(dead_code)]

#[path = "../../../src/am/ec_hnsw/concurrent_dsm_state.rs"]
mod hnsw_concurrent_dsm_state;
#[path = "../../../src/am/common/parallel_slot.rs"]
mod parallel_slot;

#[cfg(test)]
mod tests {
    use super::hnsw_concurrent_dsm_state::{
        begin_concurrent_dsm_node_insert_state, complete_concurrent_dsm_node_insert_state,
        wait_until_concurrent_dsm_node_ready, EcHnswConcurrentDsmInsertBegin,
        EcHnswConcurrentDsmInsertError, EcHnswConcurrentDsmInsertStateCell,
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING, EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY,
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED,
    };
    use super::parallel_slot::{
        load_worker_slot_snapshot, publish_worker_slot_runtime_snapshot, release_worker_slot,
        try_claim_worker_slot, EcParallelWorkerSlotFields, EcParallelWorkerSlotRuntimeSnapshot,
        ParallelSlotAtomic, EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL,
        EC_PARALLEL_WORKER_SLOT_CLAIMED, EC_PARALLEL_WORKER_SLOT_FREE,
    };
    use loom::sync::atomic::{AtomicU32, Ordering};
    use loom::sync::Arc;
    use loom::thread;

    impl EcHnswConcurrentDsmInsertStateCell for AtomicU32 {
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

        fn spin_wait(&self) {
            thread::yield_now();
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

        fn spin_wait(&self) {
            thread::yield_now();
        }
    }

    struct LoomParallelWorkerSlot {
        flags: AtomicU32,
        slot_index: u32,
        observed_rescan_epoch: AtomicU32,
        execution_phase: AtomicU32,
        scan_dimensions: AtomicU32,
        bootstrap_frontier_limit: AtomicU32,
        visible_frontier_len: AtomicU32,
        scheduler_frontier_len: AtomicU32,
        visited_count: AtomicU32,
        emitted_count: AtomicU32,
        active_result_pending_count: AtomicU32,
        active_result_has_current: AtomicU32,
    }

    impl LoomParallelWorkerSlot {
        fn new(slot_index: u32, flags: u32, rescan_epoch: u32) -> Self {
            Self {
                flags: AtomicU32::new(flags),
                slot_index,
                observed_rescan_epoch: AtomicU32::new(rescan_epoch),
                execution_phase: AtomicU32::new(0),
                scan_dimensions: AtomicU32::new(0),
                bootstrap_frontier_limit: AtomicU32::new(0),
                visible_frontier_len: AtomicU32::new(0),
                scheduler_frontier_len: AtomicU32::new(0),
                visited_count: AtomicU32::new(0),
                emitted_count: AtomicU32::new(0),
                active_result_pending_count: AtomicU32::new(0),
                active_result_has_current: AtomicU32::new(0),
            }
        }

        fn fields(&self) -> EcParallelWorkerSlotFields<'_, AtomicU32> {
            EcParallelWorkerSlotFields {
                flags: &self.flags,
                slot_index: self.slot_index,
                observed_rescan_epoch: &self.observed_rescan_epoch,
                execution_phase: &self.execution_phase,
                scan_dimensions: &self.scan_dimensions,
                bootstrap_frontier_limit: &self.bootstrap_frontier_limit,
                visible_frontier_len: &self.visible_frontier_len,
                scheduler_frontier_len: &self.scheduler_frontier_len,
                visited_count: &self.visited_count,
                emitted_count: &self.emitted_count,
                active_result_pending_count: &self.active_result_pending_count,
                active_result_has_current: &self.active_result_has_current,
            }
        }
    }

    fn non_idle_snapshot() -> EcParallelWorkerSlotRuntimeSnapshot {
        EcParallelWorkerSlotRuntimeSnapshot {
            execution_phase: EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL,
            scan_dimensions: 128,
            bootstrap_frontier_limit: 8,
            visible_frontier_len: 5,
            scheduler_frontier_len: 3,
            visited_count: 21,
            emitted_count: 2,
            active_result_pending_count: 1,
            active_result_has_current: true,
        }
    }

    #[test]
    fn worker_slot_claim_is_exclusive_for_two_workers() {
        loom::model(|| {
            let slot = Arc::new(LoomParallelWorkerSlot::new(
                0,
                EC_PARALLEL_WORKER_SLOT_FREE,
                0,
            ));
            let claimed_count = Arc::new(AtomicU32::new(0));

            let left = {
                let slot = Arc::clone(&slot);
                let claimed_count = Arc::clone(&claimed_count);
                thread::spawn(move || {
                    if try_claim_worker_slot(slot.fields(), 0) {
                        claimed_count.fetch_add(1, Ordering::AcqRel);
                    }
                })
            };
            let right = {
                let slot = Arc::clone(&slot);
                let claimed_count = Arc::clone(&claimed_count);
                thread::spawn(move || {
                    if try_claim_worker_slot(slot.fields(), 0) {
                        claimed_count.fetch_add(1, Ordering::AcqRel);
                    }
                })
            };

            left.join().unwrap();
            right.join().unwrap();

            let claimed = claimed_count.load(Ordering::Acquire);
            assert!(claimed <= 1);
            let snapshot = load_worker_slot_snapshot(slot.fields());
            assert_eq!(snapshot.flags, EC_PARALLEL_WORKER_SLOT_CLAIMED);
            assert_eq!(claimed, 1);
        });
    }

    #[test]
    fn worker_slot_claim_count_matches_live_claimed_slots() {
        let mut builder = loom::model::Builder::new();
        builder.max_permutations = Some(10_000);
        builder.check(|| {
            let slots = Arc::new(vec![
                LoomParallelWorkerSlot::new(0, EC_PARALLEL_WORKER_SLOT_FREE, 0),
                LoomParallelWorkerSlot::new(1, EC_PARALLEL_WORKER_SLOT_FREE, 0),
            ]);
            let claimed_count = Arc::new(AtomicU32::new(0));
            let mut workers = Vec::new();

            for _ in 0..4 {
                let slots = Arc::clone(&slots);
                let claimed_count = Arc::clone(&claimed_count);
                workers.push(thread::spawn(move || {
                    for slot in slots.iter() {
                        if try_claim_worker_slot(slot.fields(), 0) {
                            claimed_count.fetch_add(1, Ordering::AcqRel);
                            break;
                        }
                    }
                }));
            }

            for worker in workers {
                worker.join().unwrap();
            }

            let live_claimed = slots
                .iter()
                .filter(|slot| {
                    load_worker_slot_snapshot(slot.fields()).flags
                        == EC_PARALLEL_WORKER_SLOT_CLAIMED
                })
                .count() as u32;
            assert_eq!(claimed_count.load(Ordering::Acquire), live_claimed);
            assert!(live_claimed <= 2);
        });
    }

    #[test]
    fn release_racing_publish_leaves_free_slot_idle() {
        loom::model(|| {
            let slot = Arc::new(LoomParallelWorkerSlot::new(
                0,
                EC_PARALLEL_WORKER_SLOT_CLAIMED,
                0,
            ));

            let releaser = {
                let slot = Arc::clone(&slot);
                thread::spawn(move || release_worker_slot(slot.fields(), 0))
            };
            let publisher = {
                let slot = Arc::clone(&slot);
                thread::spawn(move || {
                    publish_worker_slot_runtime_snapshot(slot.fields(), 0, non_idle_snapshot())
                })
            };

            let released = releaser.join().unwrap();
            let _published = publisher.join().unwrap();
            assert!(released);

            let snapshot = load_worker_slot_snapshot(slot.fields());
            assert_eq!(snapshot.flags, EC_PARALLEL_WORKER_SLOT_FREE);
            assert_eq!(
                snapshot.runtime,
                EcParallelWorkerSlotRuntimeSnapshot::idle()
            );
        });
    }

    #[test]
    fn stale_epoch_release_and_publish_do_not_mutate_slot() {
        loom::model(|| {
            let slot = LoomParallelWorkerSlot::new(0, EC_PARALLEL_WORKER_SLOT_CLAIMED, 0);
            assert!(!publish_worker_slot_runtime_snapshot(
                slot.fields(),
                1,
                non_idle_snapshot()
            ));
            assert!(!release_worker_slot(slot.fields(), 1));

            let snapshot = load_worker_slot_snapshot(slot.fields());
            assert_eq!(snapshot.flags, EC_PARALLEL_WORKER_SLOT_CLAIMED);
            assert_eq!(
                snapshot.runtime,
                EcParallelWorkerSlotRuntimeSnapshot::idle()
            );
        });
    }

    #[test]
    fn hnsw_concurrent_dsm_node_insert_is_exclusive() {
        loom::model(|| {
            let state = Arc::new(AtomicU32::new(
                EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED,
            ));
            let started_count = Arc::new(AtomicU32::new(0));
            let duplicate_count = Arc::new(AtomicU32::new(0));

            let left = {
                let state = Arc::clone(&state);
                let started_count = Arc::clone(&started_count);
                let duplicate_count = Arc::clone(&duplicate_count);
                thread::spawn(
                    move || match begin_concurrent_dsm_node_insert_state(&*state, 3) {
                        Ok(EcHnswConcurrentDsmInsertBegin::Started { level }) => {
                            assert_eq!(level, 3);
                            started_count.fetch_add(1, Ordering::AcqRel);
                        }
                        Ok(EcHnswConcurrentDsmInsertBegin::AlreadyReady) => {}
                        Err(EcHnswConcurrentDsmInsertError::DuplicateInProgress) => {
                            duplicate_count.fetch_add(1, Ordering::AcqRel);
                        }
                        Err(err) => panic!("unexpected HNSW insert-state error: {err:?}"),
                    },
                )
            };
            let right = {
                let state = Arc::clone(&state);
                let started_count = Arc::clone(&started_count);
                let duplicate_count = Arc::clone(&duplicate_count);
                thread::spawn(
                    move || match begin_concurrent_dsm_node_insert_state(&*state, 3) {
                        Ok(EcHnswConcurrentDsmInsertBegin::Started { level }) => {
                            assert_eq!(level, 3);
                            started_count.fetch_add(1, Ordering::AcqRel);
                        }
                        Ok(EcHnswConcurrentDsmInsertBegin::AlreadyReady) => {}
                        Err(EcHnswConcurrentDsmInsertError::DuplicateInProgress) => {
                            duplicate_count.fetch_add(1, Ordering::AcqRel);
                        }
                        Err(err) => panic!("unexpected HNSW insert-state error: {err:?}"),
                    },
                )
            };

            left.join().unwrap();
            right.join().unwrap();

            assert_eq!(started_count.load(Ordering::Acquire), 1);
            assert!(duplicate_count.load(Ordering::Acquire) <= 1);
            assert_eq!(
                state.load(Ordering::Acquire),
                EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING
            );
        });
    }

    #[test]
    fn hnsw_concurrent_dsm_ready_is_published_after_neighbor_slots() {
        loom::model(|| {
            let state = Arc::new(AtomicU32::new(
                EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED,
            ));
            let first_neighbor_slot = Arc::new(AtomicU32::new(u32::MAX));
            let observed_ready = Arc::new(AtomicU32::new(0));

            let writer = {
                let state = Arc::clone(&state);
                let first_neighbor_slot = Arc::clone(&first_neighbor_slot);
                thread::spawn(move || {
                    assert_eq!(
                        begin_concurrent_dsm_node_insert_state(&*state, 1),
                        Ok(EcHnswConcurrentDsmInsertBegin::Started { level: 1 })
                    );
                    first_neighbor_slot.store(42, Ordering::Release);
                    complete_concurrent_dsm_node_insert_state(&*state).unwrap();
                })
            };
            let reader = {
                let state = Arc::clone(&state);
                let first_neighbor_slot = Arc::clone(&first_neighbor_slot);
                let observed_ready = Arc::clone(&observed_ready);
                thread::spawn(move || {
                    if wait_until_concurrent_dsm_node_ready(&*state) {
                        observed_ready.store(1, Ordering::Release);
                        assert_eq!(first_neighbor_slot.load(Ordering::Acquire), 42);
                    }
                })
            };

            writer.join().unwrap();
            reader.join().unwrap();

            assert_eq!(
                state.load(Ordering::Acquire),
                EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY
            );
            assert_eq!(first_neighbor_slot.load(Ordering::Acquire), 42);
            assert!(observed_ready.load(Ordering::Acquire) <= 1);
        });
    }
}
