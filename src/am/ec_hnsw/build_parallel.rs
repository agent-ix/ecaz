use std::sync::atomic::{AtomicU64, Ordering};

use pgrx::pg_sys;

const EC_HNSW_PARALLEL_BUILD_MAGIC: u32 = u32::from_le_bytes(*b"ECBP");
const EC_HNSW_PARALLEL_BUILD_VERSION: u16 = 1;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum EcHnswBuildCoordinatorKind {
    LeaderLocal,
    DedicatedParallelBuild,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum EcHnswBuildHeapIngest {
    SerialTableIndexBuildScan,
    ParallelTableScan,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum EcHnswBuildTupleSink {
    LeaderBuildStateVec,
    SharedTupleStream,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum EcHnswBuildGraphAssembly {
    SerialLeader,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct EcHnswParallelBuildPlan {
    pub(super) requested_workers: i32,
    pub(super) leader_participates: bool,
    pub(super) participant_count: i32,
    pub(super) coordinator: EcHnswBuildCoordinatorKind,
    pub(super) heap_ingest: EcHnswBuildHeapIngest,
    pub(super) tuple_sink: EcHnswBuildTupleSink,
    pub(super) graph_assembly: EcHnswBuildGraphAssembly,
}

impl EcHnswParallelBuildPlan {
    pub(super) fn from_index_info(index_info: *mut pg_sys::IndexInfo) -> Self {
        let requested_workers = if index_info.is_null() {
            0
        } else {
            unsafe { (*index_info).ii_ParallelWorkers }
        };
        Self::for_requested_workers(requested_workers)
    }

    pub(super) fn for_requested_workers(requested_workers: i32) -> Self {
        if requested_workers <= 0 {
            return Self {
                requested_workers: 0,
                leader_participates: false,
                participant_count: 1,
                coordinator: EcHnswBuildCoordinatorKind::LeaderLocal,
                heap_ingest: EcHnswBuildHeapIngest::SerialTableIndexBuildScan,
                tuple_sink: EcHnswBuildTupleSink::LeaderBuildStateVec,
                graph_assembly: EcHnswBuildGraphAssembly::SerialLeader,
            };
        }

        let leader_participates = true;
        Self {
            requested_workers,
            leader_participates,
            participant_count: requested_workers + i32::from(leader_participates),
            coordinator: EcHnswBuildCoordinatorKind::DedicatedParallelBuild,
            heap_ingest: EcHnswBuildHeapIngest::ParallelTableScan,
            tuple_sink: EcHnswBuildTupleSink::SharedTupleStream,
            graph_assembly: EcHnswBuildGraphAssembly::SerialLeader,
        }
    }

    pub(super) fn uses_serial_build_path(self) -> bool {
        self.coordinator == EcHnswBuildCoordinatorKind::LeaderLocal
    }
}

#[repr(C)]
#[derive(Debug)]
pub(super) struct EcHnswParallelBuildSharedHeader {
    magic: u32,
    version: u16,
    requested_workers: u16,
    participant_count: u16,
    flags: u16,
    scanned_heap_tuples: AtomicU64,
    encoded_index_tuples: AtomicU64,
}

impl EcHnswParallelBuildSharedHeader {
    pub(super) fn new(plan: EcHnswParallelBuildPlan) -> Self {
        Self {
            magic: EC_HNSW_PARALLEL_BUILD_MAGIC,
            version: EC_HNSW_PARALLEL_BUILD_VERSION,
            requested_workers: checked_u16(plan.requested_workers, "requested workers"),
            participant_count: checked_u16(plan.participant_count, "participant count"),
            flags: 0,
            scanned_heap_tuples: AtomicU64::new(0),
            encoded_index_tuples: AtomicU64::new(0),
        }
    }

    pub(super) fn record_worker_counts(&self, heap_tuples: u64, index_tuples: u64) {
        self.scanned_heap_tuples
            .fetch_add(heap_tuples, Ordering::AcqRel);
        self.encoded_index_tuples
            .fetch_add(index_tuples, Ordering::AcqRel);
    }

    pub(super) fn scanned_heap_tuples(&self) -> u64 {
        self.scanned_heap_tuples.load(Ordering::Acquire)
    }

    pub(super) fn encoded_index_tuples(&self) -> u64 {
        self.encoded_index_tuples.load(Ordering::Acquire)
    }
}

fn checked_u16(value: i32, field: &str) -> u16 {
    u16::try_from(value).unwrap_or_else(|_| panic!("parallel build {field} should fit in u16"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_build_plan_stays_serial_without_requested_workers() {
        let plan = EcHnswParallelBuildPlan::for_requested_workers(0);

        assert_eq!(plan.requested_workers, 0);
        assert_eq!(plan.participant_count, 1);
        assert!(!plan.leader_participates);
        assert_eq!(plan.coordinator, EcHnswBuildCoordinatorKind::LeaderLocal);
        assert_eq!(
            plan.heap_ingest,
            EcHnswBuildHeapIngest::SerialTableIndexBuildScan
        );
        assert_eq!(plan.tuple_sink, EcHnswBuildTupleSink::LeaderBuildStateVec);
        assert_eq!(plan.graph_assembly, EcHnswBuildGraphAssembly::SerialLeader);
        assert!(plan.uses_serial_build_path());
    }

    #[test]
    fn parallel_build_plan_uses_dedicated_build_coordinator() {
        let plan = EcHnswParallelBuildPlan::for_requested_workers(3);

        assert_eq!(plan.requested_workers, 3);
        assert_eq!(plan.participant_count, 4);
        assert!(plan.leader_participates);
        assert_eq!(
            plan.coordinator,
            EcHnswBuildCoordinatorKind::DedicatedParallelBuild
        );
        assert_eq!(plan.heap_ingest, EcHnswBuildHeapIngest::ParallelTableScan);
        assert_eq!(plan.tuple_sink, EcHnswBuildTupleSink::SharedTupleStream);
        assert_eq!(plan.graph_assembly, EcHnswBuildGraphAssembly::SerialLeader);
        assert!(!plan.uses_serial_build_path());
    }

    #[test]
    fn parallel_build_shared_header_accumulates_counts() {
        let plan = EcHnswParallelBuildPlan::for_requested_workers(2);
        let shared = EcHnswParallelBuildSharedHeader::new(plan);

        shared.record_worker_counts(11, 7);
        shared.record_worker_counts(13, 5);

        assert_eq!(shared.scanned_heap_tuples(), 24);
        assert_eq!(shared.encoded_index_tuples(), 12);
    }
}
