use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;

use pgrx::pg_sys;

use super::{build, graph, insert, page, search, shared, source};

const EC_HNSW_PARALLEL_BUILD_MAGIC: u32 = u32::from_le_bytes(*b"ECBP");
const EC_HNSW_PARALLEL_BUILD_VERSION: u16 = 1;
const EC_HNSW_PARALLEL_BUILD_QUEUE_BYTES: pg_sys::Size = 1024 * 1024;

const PARALLEL_KEY_EC_HNSW_BUILD_SHARED: u64 = 0xECA0_0000_0000_0001;
const PARALLEL_KEY_EC_HNSW_WAL_USAGE: u64 = 0xECA0_0000_0000_0002;
const PARALLEL_KEY_EC_HNSW_BUFFER_USAGE: u64 = 0xECA0_0000_0000_0003;
const PARALLEL_KEY_EC_HNSW_QUEUE_BASE: u64 = 0xECA0_0000_0001_0000;

const EC_HNSW_PARALLEL_BUILD_LIBRARY: &[u8] = b"ecaz\0";
const EC_HNSW_PARALLEL_BUILD_ENTRYPOINT: &[u8] = b"ec_hnsw_parallel_build_main\0";

const BUILD_TUPLE_MESSAGE: u8 = 1;
const BUILD_DONE_MESSAGE: u8 = 2;

const EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX: u32 = u32::MAX;
const EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED: u32 = 0;
const EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING: u32 = 1;
const EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY: u32 = 2;

static LAST_PARALLEL_BUILD_WORKERS_LAUNCHED: AtomicI32 = AtomicI32::new(0);

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
    #[allow(dead_code)]
    ConcurrentDsm,
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

        /*
         * This first executable build coordinator keeps the leader dedicated to
         * draining worker message queues.  Leader participation can be added
         * once tuple transport moves to a shared sorter or another nonblocking
         * merge surface.
         */
        let leader_participates = false;
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
    heaprelid: pg_sys::Oid,
    indexrelid: pg_sys::Oid,
    is_concurrent: bool,
    reserved0: [u8; 3],
    workersdonecv: pg_sys::ConditionVariable,
    mutex: pg_sys::slock_t,
    nparticipantsdone: i32,
    scanned_heap_tuples: f64,
    encoded_index_tuples: f64,
}

#[allow(dead_code)]
#[repr(C)]
struct EcHnswConcurrentDsmGraphHeader {
    node_count: u32,
    entry_idx: u32,
    max_level: u8,
    reserved0: [u8; 3],
    total_neighbor_slots: u32,
    code_len: u32,
}

#[allow(dead_code)]
#[repr(C)]
struct EcHnswConcurrentDsmNode {
    lock: pg_sys::LWLock,
    level: u8,
    reserved0: [u8; 3],
    neighbor_slot_offset: u32,
    neighbor_slot_count: u32,
    insert_state: pg_sys::pg_atomic_uint32,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub(super) struct EcHnswConcurrentDsmGraphParts {
    header: *mut EcHnswConcurrentDsmGraphHeader,
    nodes: *mut EcHnswConcurrentDsmNode,
    neighbor_slots: *mut u32,
    codes: *mut u8,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct EcHnswConcurrentDsmNodeLayout {
    pub(super) level: u8,
    pub(super) neighbor_slot_offset: u32,
    pub(super) neighbor_slot_count: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EcHnswConcurrentDsmNodeLayoutPlan {
    pub(super) nodes: Vec<EcHnswConcurrentDsmNodeLayout>,
    pub(super) total_neighbor_slots: u32,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct EcHnswConcurrentDsmNodePartition {
    pub(super) participant_index: u16,
    pub(super) start_node_idx: u32,
    pub(super) end_node_idx: u32,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmNodePartition {
    pub(super) fn is_empty(self) -> bool {
        self.start_node_idx == self.end_node_idx
    }

    pub(super) fn contains(self, node_idx: u32) -> bool {
        self.start_node_idx <= node_idx && node_idx < self.end_node_idx
    }
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmNodeLayoutPlan {
    pub(super) fn for_levels(levels: &build::NativeBuildLevels, m: u16) -> Self {
        let mut nodes = Vec::with_capacity(levels.levels.len());
        let mut next_neighbor_slot_offset = 0_usize;

        for level in levels.levels.iter().copied() {
            let neighbor_slot_count = page::neighbor_slots(level, m);
            nodes.push(EcHnswConcurrentDsmNodeLayout {
                level,
                neighbor_slot_offset: checked_u32(
                    next_neighbor_slot_offset,
                    "concurrent DSM graph node neighbor slot offset",
                ),
                neighbor_slot_count: checked_u32(
                    neighbor_slot_count,
                    "concurrent DSM graph node neighbor slot count",
                ),
            });
            next_neighbor_slot_offset = next_neighbor_slot_offset
                .checked_add(neighbor_slot_count)
                .unwrap_or_else(|| {
                    pgrx::error!("concurrent DSM graph neighbor slot count overflow")
                });
        }

        Self {
            nodes,
            total_neighbor_slots: checked_graph_u32(
                next_neighbor_slot_offset,
                "concurrent DSM graph neighbor slot count",
            ),
        }
    }
}

#[allow(dead_code)]
pub(super) fn concurrent_dsm_node_partitions(
    node_count: u32,
    participant_count: u16,
) -> Vec<EcHnswConcurrentDsmNodePartition> {
    if participant_count == 0 {
        pgrx::error!("concurrent DSM graph insertion requires at least one participant");
    }

    let node_count = node_count as usize;
    let participant_count = participant_count as usize;
    let base_len = node_count / participant_count;
    let remainder = node_count % participant_count;
    let mut start_node_idx = 0_usize;
    let mut partitions = Vec::with_capacity(participant_count);

    for participant_index in 0..participant_count {
        let len = base_len + usize::from(participant_index < remainder);
        let end_node_idx = start_node_idx
            .checked_add(len)
            .unwrap_or_else(|| pgrx::error!("concurrent DSM node partition overflow"));
        partitions.push(EcHnswConcurrentDsmNodePartition {
            participant_index: checked_u16(
                participant_index as i32,
                "concurrent DSM participant index",
            ),
            start_node_idx: checked_graph_u32(
                start_node_idx,
                "concurrent DSM partition start node index",
            ),
            end_node_idx: checked_graph_u32(
                end_node_idx,
                "concurrent DSM partition end node index",
            ),
        });
        start_node_idx = end_node_idx;
    }

    partitions
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct EcHnswConcurrentDsmInsertConfig {
    pub(super) dimensions: usize,
    pub(super) bits: u8,
    pub(super) seed: u64,
    pub(super) m: usize,
    pub(super) ef_construction: usize,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmInsertConfig {
    pub(super) fn for_state(state: &build::BuildState) -> Self {
        Self {
            dimensions: state
                .dimensions
                .expect("non-empty concurrent DSM build should record dimensions")
                as usize,
            bits: state
                .bits
                .expect("non-empty concurrent DSM build should record bits"),
            seed: state
                .seed
                .expect("non-empty concurrent DSM build should record seed"),
            m: usize::try_from(state.options.m).expect("validated m should be non-negative"),
            ef_construction: usize::try_from(state.options.ef_construction)
                .expect("validated ef_construction should be non-negative")
                .max(1),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub(super) struct EcHnswConcurrentDsmInsertScratch {
    query_scores: EcHnswConcurrentDsmQueryScoreCache,
    layer_search: EcHnswConcurrentDsmLayerSearchScratch,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmInsertScratch {
    pub(super) fn new(node_count: usize, ef_construction: usize, m: usize) -> Self {
        Self {
            query_scores: EcHnswConcurrentDsmQueryScoreCache::new(node_count),
            layer_search: EcHnswConcurrentDsmLayerSearchScratch::new(
                node_count,
                ef_construction,
                m,
            ),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(super) struct EcHnswConcurrentDsmLockOps {
    acquire_shared: unsafe fn(*mut pg_sys::LWLock),
    acquire_exclusive: unsafe fn(*mut pg_sys::LWLock),
    release: unsafe fn(*mut pg_sys::LWLock),
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmLockOps {
    pub(super) fn postgres() -> Self {
        Self {
            acquire_shared: concurrent_dsm_lwlock_acquire_shared,
            acquire_exclusive: concurrent_dsm_lwlock_acquire_exclusive,
            release: concurrent_dsm_lwlock_release,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EcHnswConcurrentDsmForwardSelection {
    layer: u8,
    node_idx: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
struct EcHnswConcurrentDsmQueryScoreCache {
    scores: Vec<f32>,
    generations: Vec<u32>,
    current_generation: u32,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmQueryScoreCache {
    fn new(capacity: usize) -> Self {
        Self {
            scores: vec![0.0; capacity],
            generations: vec![0; capacity],
            current_generation: 0,
        }
    }

    fn begin_query(&mut self) {
        self.current_generation = self.current_generation.wrapping_add(1);
        if self.current_generation == 0 {
            self.generations.fill(0);
            self.current_generation = 1;
        }
    }

    fn get(&self, candidate_idx: u32) -> Option<f32> {
        let candidate_idx = candidate_idx as usize;
        if self.generations[candidate_idx] == self.current_generation {
            Some(self.scores[candidate_idx])
        } else {
            None
        }
    }

    fn insert(&mut self, candidate_idx: u32, score: f32) {
        let candidate_idx = candidate_idx as usize;
        self.scores[candidate_idx] = score;
        self.generations[candidate_idx] = self.current_generation;
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct EcHnswConcurrentDsmLayerSearchScratch {
    visited: EcHnswConcurrentDsmVisitedSet,
    candidate_points: BinaryHeap<Reverse<EcHnswConcurrentDsmLayerSearchCandidate>>,
    result_points: BinaryHeap<EcHnswConcurrentDsmLayerSearchCandidate>,
    successors: Vec<search::BeamCandidate<u32>>,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmLayerSearchScratch {
    fn new(node_count: usize, ef_construction: usize, m: usize) -> Self {
        Self {
            visited: EcHnswConcurrentDsmVisitedSet::new(node_count),
            candidate_points: BinaryHeap::with_capacity(ef_construction),
            result_points: BinaryHeap::with_capacity(ef_construction.saturating_add(1)),
            successors: Vec::with_capacity(m.saturating_mul(2)),
        }
    }

    fn clear(&mut self) {
        self.visited.begin_search();
        self.candidate_points.clear();
        self.result_points.clear();
        self.successors.clear();
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct EcHnswConcurrentDsmVisitedSet {
    generations: Vec<u32>,
    current_generation: u32,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmVisitedSet {
    fn new(capacity: usize) -> Self {
        Self {
            generations: vec![0; capacity],
            current_generation: 0,
        }
    }

    fn begin_search(&mut self) {
        self.current_generation = self.current_generation.wrapping_add(1);
        if self.current_generation == 0 {
            self.generations.fill(0);
            self.current_generation = 1;
        }
    }

    fn insert(&mut self, node_idx: u32) -> bool {
        let generation = self
            .generations
            .get_mut(node_idx as usize)
            .unwrap_or_else(|| pgrx::error!("concurrent DSM visited node index out of bounds"));
        if *generation == self.current_generation {
            return false;
        }
        *generation = self.current_generation;
        true
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct EcHnswConcurrentDsmLayerSearchCandidate {
    candidate: search::BeamCandidate<u32>,
    sequence: u64,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmLayerSearchCandidate {
    fn new(candidate: search::BeamCandidate<u32>, sequence: u64) -> Self {
        Self {
            candidate,
            sequence,
        }
    }
}

impl Eq for EcHnswConcurrentDsmLayerSearchCandidate {}

impl Ord for EcHnswConcurrentDsmLayerSearchCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.candidate
            .score
            .total_cmp(&other.candidate.score)
            .then_with(|| self.sequence.cmp(&other.sequence))
    }
}

impl PartialOrd for EcHnswConcurrentDsmLayerSearchCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EcHnswConcurrentDsmCodeCorpus {
    pub(super) node_count: u32,
    pub(super) code_len: u32,
    pub(super) bytes: Vec<u8>,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmCodeCorpus {
    pub(super) fn from_tuples(tuples: &[build::BuildTuple]) -> Self {
        let node_count = checked_graph_u32(tuples.len(), "concurrent DSM code corpus nodes");
        let code_len = tuples.first().map_or(0_usize, |tuple| tuple.code.len());
        let total_code_bytes = code_len
            .checked_mul(tuples.len())
            .unwrap_or_else(|| pgrx::error!("concurrent DSM code corpus byte count overflow"));
        let mut bytes = Vec::with_capacity(total_code_bytes);

        for tuple in tuples {
            if tuple.code.len() != code_len {
                pgrx::error!("concurrent DSM code corpus requires fixed-width codes");
            }
            bytes.extend_from_slice(&tuple.code);
        }

        Self {
            node_count,
            code_len: checked_graph_u32(code_len, "concurrent DSM code corpus code length"),
            bytes,
        }
    }

    pub(super) fn code_for_node(&self, node_idx: usize) -> &[u8] {
        if node_idx >= self.node_count as usize {
            pgrx::error!("concurrent DSM code corpus node index out of bounds");
        }
        let code_len = self.code_len as usize;
        let start = node_idx
            .checked_mul(code_len)
            .unwrap_or_else(|| pgrx::error!("concurrent DSM code corpus offset overflow"));
        let end = start
            .checked_add(code_len)
            .unwrap_or_else(|| pgrx::error!("concurrent DSM code corpus offset overflow"));
        self.bytes
            .get(start..end)
            .unwrap_or_else(|| pgrx::error!("concurrent DSM code corpus slice out of bounds"))
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct EcHnswConcurrentDsmGraphLayout {
    pub(super) node_count: u32,
    pub(super) entry_idx: Option<u32>,
    pub(super) max_level: u8,
    pub(super) total_neighbor_slots: u32,
    pub(super) code_len: u32,
    pub(super) header_offset: pg_sys::Size,
    pub(super) nodes_offset: pg_sys::Size,
    pub(super) neighbor_slots_offset: pg_sys::Size,
    pub(super) codes_offset: pg_sys::Size,
    pub(super) total_bytes: pg_sys::Size,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmGraphLayout {
    pub(super) fn for_levels(levels: &build::NativeBuildLevels, m: u16, code_len: usize) -> Self {
        let node_plan = EcHnswConcurrentDsmNodeLayoutPlan::for_levels(levels, m);
        let node_count = checked_graph_u32(levels.levels.len(), "concurrent DSM graph nodes");
        let entry_idx = levels
            .entry_idx
            .map(|idx| checked_graph_u32(idx, "concurrent DSM graph entry index"));
        let total_neighbor_slots = node_plan.total_neighbor_slots;
        let code_len = checked_graph_u32(code_len, "concurrent DSM graph code length");

        let header_offset = 0;
        let nodes_offset = bufferalign(size_of::<EcHnswConcurrentDsmGraphHeader>() as pg_sys::Size);
        let node_bytes = checked_mul_size(
            size_of::<EcHnswConcurrentDsmNode>() as pg_sys::Size,
            node_count as pg_sys::Size,
            "concurrent DSM graph node array",
        );
        let neighbor_slots_offset = checked_add_size(
            nodes_offset,
            bufferalign(node_bytes),
            "concurrent DSM graph neighbor slot offset",
        );
        let neighbor_slot_bytes = checked_mul_size(
            size_of::<u32>() as pg_sys::Size,
            total_neighbor_slots as pg_sys::Size,
            "concurrent DSM graph neighbor slot array",
        );
        let codes_offset = checked_add_size(
            neighbor_slots_offset,
            bufferalign(neighbor_slot_bytes),
            "concurrent DSM graph code offset",
        );
        let code_bytes = checked_mul_size(
            code_len as pg_sys::Size,
            node_count as pg_sys::Size,
            "concurrent DSM graph code corpus",
        );
        let total_bytes = checked_add_size(
            codes_offset,
            bufferalign(code_bytes),
            "concurrent DSM graph total bytes",
        );

        Self {
            node_count,
            entry_idx,
            max_level: levels.max_level,
            total_neighbor_slots,
            code_len,
            header_offset,
            nodes_offset,
            neighbor_slots_offset,
            codes_offset,
            total_bytes,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EcHnswConcurrentDsmPreassemblyPlan {
    pub(super) levels: build::NativeBuildLevels,
    pub(super) node_layout: EcHnswConcurrentDsmNodeLayoutPlan,
    pub(super) code_corpus: EcHnswConcurrentDsmCodeCorpus,
    pub(super) graph_layout: EcHnswConcurrentDsmGraphLayout,
}

#[allow(dead_code)]
impl EcHnswConcurrentDsmPreassemblyPlan {
    pub(super) fn for_state(state: &build::BuildState) -> Self {
        if state.options.build_source_column.is_some() {
            pgrx::error!("concurrent DSM graph assembly does not support source-scored builds yet");
        }

        let m = u16::try_from(state.options.m).expect("validated m should fit into u16");
        let levels = if state.heap_tuples.is_empty() {
            build::NativeBuildLevels::from_levels(Vec::new())
        } else {
            build::precompute_native_build_levels(state, m)
        };
        let node_layout = EcHnswConcurrentDsmNodeLayoutPlan::for_levels(&levels, m);
        let code_corpus = EcHnswConcurrentDsmCodeCorpus::from_tuples(&state.heap_tuples);
        let graph_layout =
            EcHnswConcurrentDsmGraphLayout::for_levels(&levels, m, code_corpus.code_len as usize);

        if graph_layout.node_count != code_corpus.node_count {
            pgrx::error!("concurrent DSM preassembly node counts do not match");
        }
        if graph_layout.total_neighbor_slots != node_layout.total_neighbor_slots {
            pgrx::error!("concurrent DSM preassembly neighbor slot counts do not match");
        }

        Self {
            levels,
            node_layout,
            code_corpus,
            graph_layout,
        }
    }
}

#[allow(dead_code)]
pub(super) unsafe fn concurrent_dsm_graph_parts(
    base: *mut c_void,
    layout: EcHnswConcurrentDsmGraphLayout,
) -> EcHnswConcurrentDsmGraphParts {
    if base.is_null() {
        pgrx::error!("concurrent DSM graph base pointer is null");
    }

    let base = base.cast::<u8>();
    EcHnswConcurrentDsmGraphParts {
        header: unsafe {
            base.add(layout.header_offset)
                .cast::<EcHnswConcurrentDsmGraphHeader>()
        },
        nodes: unsafe {
            base.add(layout.nodes_offset)
                .cast::<EcHnswConcurrentDsmNode>()
        },
        neighbor_slots: unsafe { base.add(layout.neighbor_slots_offset).cast::<u32>() },
        codes: unsafe { base.add(layout.codes_offset) },
    }
}

#[allow(dead_code)]
pub(super) unsafe fn initialize_concurrent_dsm_graph_image(
    base: *mut c_void,
    plan: &EcHnswConcurrentDsmPreassemblyPlan,
    initialize_node_lock: unsafe fn(*mut pg_sys::LWLock),
) -> EcHnswConcurrentDsmGraphParts {
    let layout = plan.graph_layout;
    let parts = unsafe { concurrent_dsm_graph_parts(base, layout) };

    unsafe {
        ptr::write(
            parts.header,
            EcHnswConcurrentDsmGraphHeader {
                node_count: layout.node_count,
                entry_idx: layout
                    .entry_idx
                    .unwrap_or(EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX),
                max_level: layout.max_level,
                reserved0: [0; 3],
                total_neighbor_slots: layout.total_neighbor_slots,
                code_len: layout.code_len,
            },
        );

        for (node_idx, node_layout) in plan.node_layout.nodes.iter().copied().enumerate() {
            let node = parts.nodes.add(node_idx);
            let node_idx = checked_graph_u32(node_idx, "concurrent DSM initialized node index");
            let insert_state = if Some(node_idx) == layout.entry_idx {
                EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY
            } else {
                EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED
            };
            ptr::write(
                node,
                EcHnswConcurrentDsmNode {
                    lock: pg_sys::LWLock::default(),
                    level: node_layout.level,
                    reserved0: [0; 3],
                    neighbor_slot_offset: node_layout.neighbor_slot_offset,
                    neighbor_slot_count: node_layout.neighbor_slot_count,
                    insert_state: pg_sys::pg_atomic_uint32 {
                        value: insert_state,
                    },
                },
            );
            initialize_node_lock(ptr::addr_of_mut!((*node).lock));
        }

        slice::from_raw_parts_mut(parts.neighbor_slots, layout.total_neighbor_slots as usize)
            .fill(EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX);

        let code_bytes = checked_mul_size(
            layout.code_len as pg_sys::Size,
            layout.node_count as pg_sys::Size,
            "concurrent DSM graph initialized code bytes",
        );
        slice::from_raw_parts_mut(parts.codes, code_bytes).copy_from_slice(&plan.code_corpus.bytes);
    }

    parts
}

#[allow(dead_code)]
pub(super) unsafe fn concurrent_dsm_graph_to_build_nodes(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    m: usize,
) -> Vec<build::HnswBuildNode> {
    let nodes = unsafe { slice::from_raw_parts(parts.nodes, layout.node_count as usize) };
    let mut build_nodes = Vec::with_capacity(nodes.len());

    for (node_idx, node) in nodes.iter().enumerate() {
        if node.insert_state.value != EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY {
            pgrx::error!("concurrent DSM graph readback saw an uninserted node");
        }

        let raw_neighbor_slots = unsafe {
            slice::from_raw_parts(
                parts.neighbor_slots.add(node.neighbor_slot_offset as usize),
                node.neighbor_slot_count as usize,
            )
        };
        let mut neighbor_slots = Vec::with_capacity(raw_neighbor_slots.len());
        for neighbor_idx in raw_neighbor_slots.iter().copied() {
            if neighbor_idx == EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX {
                neighbor_slots.push(None);
            } else if neighbor_idx >= layout.node_count {
                pgrx::error!("concurrent DSM graph readback saw out-of-range neighbor index");
            } else {
                neighbor_slots.push(Some(neighbor_idx as usize));
            }
        }

        let score_neighbors =
            build::flatten_native_neighbor_slots(node_idx, node.level, m, &neighbor_slots);
        build_nodes.push(build::HnswBuildNode {
            level: node.level,
            neighbor_slots,
            score_neighbors,
        });
    }

    build_nodes
}

#[allow(dead_code)]
pub(super) unsafe fn insert_concurrent_dsm_graph_node(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    node_idx: u32,
    scratch: &mut EcHnswConcurrentDsmInsertScratch,
    locks: EcHnswConcurrentDsmLockOps,
) -> bool {
    if node_idx >= layout.node_count {
        pgrx::error!("concurrent DSM graph insert node index out of bounds");
    }
    if config.m == 0 {
        pgrx::error!("concurrent DSM graph insert requires m > 0");
    }

    let Some(insert_level) =
        (unsafe { begin_concurrent_dsm_graph_node_insert(parts, node_idx, locks) })
    else {
        return false;
    };

    let entry_idx = unsafe { (*parts.header).entry_idx };
    if entry_idx == EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX {
        let selected_slots =
            vec![
                EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX;
                unsafe { (*parts.nodes.add(node_idx as usize)).neighbor_slot_count as usize }
            ];
        unsafe {
            complete_concurrent_dsm_graph_node_insert(parts, node_idx, &selected_slots, locks)
        };
        return true;
    }
    if entry_idx >= layout.node_count {
        pgrx::error!("concurrent DSM graph insert saw out-of-range entry index");
    }

    scratch.query_scores.begin_query();
    let entry_score =
        unsafe { score_concurrent_dsm_code(parts, layout, config, node_idx, entry_idx, scratch) };
    let entry_candidate = search::BeamCandidate::new(entry_idx, entry_score);
    let mut selected_slots =
        vec![
            EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX;
            unsafe { (*parts.nodes.add(node_idx as usize)).neighbor_slot_count as usize }
        ];
    let mut selections = Vec::new();

    let entry_level = unsafe { (*parts.nodes.add(entry_idx as usize)).level };
    let layer0_seeds = unsafe {
        populate_concurrent_dsm_upper_layer_forward_slots(
            parts,
            layout,
            config,
            node_idx,
            insert_level,
            entry_candidate,
            entry_level,
            &mut selected_slots,
            &mut selections,
            scratch,
            locks,
        )
    };
    let layer0_candidates = unsafe {
        search_concurrent_dsm_layer_result_candidates(
            config.ef_construction,
            layer0_seeds,
            parts,
            layout,
            config,
            node_idx,
            0,
            scratch,
            locks,
        )
    };
    write_concurrent_dsm_layer_forward_candidates(
        &mut selected_slots,
        &mut selections,
        0,
        config.m,
        layer0_candidates,
    );

    unsafe {
        complete_concurrent_dsm_graph_node_insert(parts, node_idx, &selected_slots, locks);
        add_concurrent_dsm_backlinks(parts, layout, config, node_idx, &selections, scratch, locks);
    }
    true
}

#[allow(dead_code)]
pub(super) unsafe fn insert_concurrent_dsm_graph_partition(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    partition: EcHnswConcurrentDsmNodePartition,
    scratch: &mut EcHnswConcurrentDsmInsertScratch,
    locks: EcHnswConcurrentDsmLockOps,
) -> u32 {
    if partition.start_node_idx > partition.end_node_idx
        || partition.end_node_idx > layout.node_count
    {
        pgrx::error!("concurrent DSM graph partition is out of bounds");
    }

    let mut inserted = 0_u32;
    for node_idx in partition.start_node_idx..partition.end_node_idx {
        if unsafe {
            insert_concurrent_dsm_graph_node(parts, layout, config, node_idx, scratch, locks)
        } {
            inserted = inserted
                .checked_add(1)
                .unwrap_or_else(|| pgrx::error!("concurrent DSM inserted count overflow"));
        }
    }
    inserted
}

#[allow(dead_code)]
pub(super) unsafe fn insert_concurrent_dsm_graph_participant(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    participant_count: u16,
    participant_index: u16,
    scratch: &mut EcHnswConcurrentDsmInsertScratch,
    locks: EcHnswConcurrentDsmLockOps,
) -> u32 {
    if participant_index >= participant_count {
        pgrx::error!("concurrent DSM graph participant index is out of bounds");
    }
    let partitions = concurrent_dsm_node_partitions(layout.node_count, participant_count);
    let partition = partitions[participant_index as usize];
    unsafe {
        insert_concurrent_dsm_graph_partition(parts, layout, config, partition, scratch, locks)
    }
}

unsafe fn begin_concurrent_dsm_graph_node_insert(
    parts: EcHnswConcurrentDsmGraphParts,
    node_idx: u32,
    locks: EcHnswConcurrentDsmLockOps,
) -> Option<u8> {
    let node = unsafe { parts.nodes.add(node_idx as usize) };
    let lock = unsafe { ptr::addr_of_mut!((*node).lock) };
    unsafe { (locks.acquire_exclusive)(lock) };
    let state = unsafe { (*node).insert_state.value };
    let level = unsafe { (*node).level };
    match state {
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY => {
            unsafe { (locks.release)(lock) };
            None
        }
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED => {
            unsafe {
                (*node).insert_state.value = EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING;
                (locks.release)(lock);
            }
            Some(level)
        }
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING => {
            unsafe { (locks.release)(lock) };
            pgrx::error!("concurrent DSM graph insert saw a duplicate in-progress node");
        }
        _ => {
            unsafe { (locks.release)(lock) };
            pgrx::error!("concurrent DSM graph insert saw an unknown node state");
        }
    }
}

unsafe fn complete_concurrent_dsm_graph_node_insert(
    parts: EcHnswConcurrentDsmGraphParts,
    node_idx: u32,
    selected_slots: &[u32],
    locks: EcHnswConcurrentDsmLockOps,
) {
    let node = unsafe { parts.nodes.add(node_idx as usize) };
    let lock = unsafe { ptr::addr_of_mut!((*node).lock) };
    unsafe { (locks.acquire_exclusive)(lock) };
    let slot_count = unsafe { (*node).neighbor_slot_count as usize };
    if selected_slots.len() != slot_count {
        unsafe { (locks.release)(lock) };
        pgrx::error!("concurrent DSM graph insert selected slot count mismatch");
    }
    let slots = unsafe { concurrent_dsm_node_slots_mut(parts, node) };
    slots.copy_from_slice(selected_slots);
    unsafe {
        (*node).insert_state.value = EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY;
        (locks.release)(lock);
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn populate_concurrent_dsm_upper_layer_forward_slots(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    query_idx: u32,
    insert_level: u8,
    entry_candidate: search::BeamCandidate<u32>,
    entry_level: u8,
    selected_slots: &mut [u32],
    selections: &mut Vec<EcHnswConcurrentDsmForwardSelection>,
    scratch: &mut EcHnswConcurrentDsmInsertScratch,
    locks: EcHnswConcurrentDsmLockOps,
) -> Vec<search::BeamCandidate<u32>> {
    if entry_level == 0 {
        return vec![entry_candidate];
    }

    let mut seeds = vec![entry_candidate];
    for current_layer in (1..=entry_level).rev() {
        seeds = unsafe {
            search_concurrent_dsm_layer_result_candidates(
                config.ef_construction,
                seeds,
                parts,
                layout,
                config,
                query_idx,
                current_layer,
                scratch,
                locks,
            )
        };
        if current_layer <= insert_level {
            write_concurrent_dsm_layer_forward_candidates(
                selected_slots,
                selections,
                current_layer,
                config.m,
                seeds.iter().copied(),
            );
        }
        if seeds.is_empty() {
            break;
        }
    }

    seeds
}

#[allow(clippy::too_many_arguments)]
unsafe fn search_concurrent_dsm_layer_result_candidates(
    ef_search: usize,
    seeds: impl IntoIterator<Item = search::BeamCandidate<u32>>,
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    query_idx: u32,
    layer: u8,
    scratch: &mut EcHnswConcurrentDsmInsertScratch,
    locks: EcHnswConcurrentDsmLockOps,
) -> Vec<search::BeamCandidate<u32>> {
    if ef_search == 0 {
        return Vec::new();
    }

    scratch.layer_search.clear();
    let mut sequence = 0_u64;

    for seed in seeds {
        if seed.node >= layout.node_count || !scratch.layer_search.visited.insert(seed.node) {
            continue;
        }

        let queued = EcHnswConcurrentDsmLayerSearchCandidate::new(seed, sequence);
        scratch.layer_search.candidate_points.push(Reverse(queued));
        scratch.layer_search.result_points.push(queued);
        sequence += 1;
    }

    while let Some(Reverse(candidate)) = scratch.layer_search.candidate_points.pop() {
        let Some(worst_result) = scratch.layer_search.result_points.peek() else {
            break;
        };

        if scratch.layer_search.result_points.len() >= ef_search
            && candidate.candidate.score > worst_result.candidate.score
        {
            break;
        }

        unsafe {
            load_concurrent_dsm_successor_candidates_into(
                parts,
                layout,
                config,
                query_idx,
                candidate.candidate.node,
                layer,
                &mut scratch.query_scores,
                &mut scratch.layer_search.successors,
                locks,
            );
        }
        for idx in 0..scratch.layer_search.successors.len() {
            let neighbor = scratch.layer_search.successors[idx];
            if !scratch.layer_search.visited.insert(neighbor.node) {
                continue;
            }

            let should_enqueue = scratch.layer_search.result_points.len() < ef_search
                || scratch
                    .layer_search
                    .result_points
                    .peek()
                    .map(|worst| neighbor.score < worst.candidate.score)
                    .unwrap_or(true);
            if !should_enqueue {
                continue;
            }

            let queued = EcHnswConcurrentDsmLayerSearchCandidate::new(neighbor, sequence);
            sequence += 1;
            scratch.layer_search.candidate_points.push(Reverse(queued));
            scratch.layer_search.result_points.push(queued);
            if scratch.layer_search.result_points.len() > ef_search {
                scratch.layer_search.result_points.pop();
            }
        }
    }

    let mut results = Vec::with_capacity(scratch.layer_search.result_points.len());
    while let Some(queued) = scratch.layer_search.result_points.pop() {
        results.push(queued.candidate);
    }
    results.sort_by(|left, right| left.score.total_cmp(&right.score));
    results
}

#[allow(clippy::too_many_arguments)]
unsafe fn load_concurrent_dsm_successor_candidates_into(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    query_idx: u32,
    source_idx: u32,
    layer: u8,
    query_scores: &mut EcHnswConcurrentDsmQueryScoreCache,
    out: &mut Vec<search::BeamCandidate<u32>>,
    locks: EcHnswConcurrentDsmLockOps,
) {
    out.clear();
    if source_idx >= layout.node_count {
        pgrx::error!("concurrent DSM graph search source index out of bounds");
    }

    let source = unsafe { parts.nodes.add(source_idx as usize) };
    let source_lock = unsafe { ptr::addr_of_mut!((*source).lock) };
    let mut neighbor_idxs = Vec::new();
    unsafe { (locks.acquire_shared)(source_lock) };
    let source_state = unsafe { (*source).insert_state.value };
    if source_state == EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY {
        let source_level = unsafe { (*source).level };
        if let Some((start, end)) = graph::layer_slot_bounds(source_level, config.m, layer) {
            let raw_slots = unsafe { concurrent_dsm_node_slots(parts, source) };
            for neighbor_idx in raw_slots[start.min(raw_slots.len())..end.min(raw_slots.len())]
                .iter()
                .copied()
            {
                if neighbor_idx != EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX {
                    neighbor_idxs.push(neighbor_idx);
                }
            }
        }
    }
    unsafe { (locks.release)(source_lock) };

    for neighbor_idx in neighbor_idxs {
        if neighbor_idx >= layout.node_count {
            pgrx::error!("concurrent DSM graph search saw out-of-range neighbor index");
        }
        if unsafe { (*parts.nodes.add(neighbor_idx as usize)).insert_state.value }
            != EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY
        {
            continue;
        }
        let score = unsafe {
            score_concurrent_dsm_code_with_cache(
                parts,
                layout,
                config,
                query_idx,
                neighbor_idx,
                query_scores,
            )
        };
        out.push(search::BeamCandidate::new(neighbor_idx, score));
    }
}

fn write_concurrent_dsm_layer_forward_candidates(
    slots: &mut [u32],
    selections: &mut Vec<EcHnswConcurrentDsmForwardSelection>,
    layer: u8,
    m: usize,
    candidates: impl IntoIterator<Item = search::BeamCandidate<u32>>,
) {
    let Some((start, end)) = insert::selected_forward_slot_bounds(m, slots.len(), layer) else {
        return;
    };

    for (slot, candidate) in slots[start..end]
        .iter_mut()
        .zip(candidates.into_iter().take(end.saturating_sub(start)))
    {
        *slot = candidate.node;
        selections.push(EcHnswConcurrentDsmForwardSelection {
            layer,
            node_idx: candidate.node,
        });
    }
}

unsafe fn add_concurrent_dsm_backlinks(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    new_node_idx: u32,
    selections: &[EcHnswConcurrentDsmForwardSelection],
    scratch: &mut EcHnswConcurrentDsmInsertScratch,
    locks: EcHnswConcurrentDsmLockOps,
) {
    let mut pending = selections
        .iter()
        .copied()
        .filter(|selection| selection.node_idx != new_node_idx)
        .collect::<Vec<_>>();
    pending.sort_unstable_by(|left, right| {
        left.node_idx
            .cmp(&right.node_idx)
            .then_with(|| left.layer.cmp(&right.layer))
    });
    pending.dedup();

    for selection in pending {
        if selection.node_idx >= layout.node_count {
            pgrx::error!("concurrent DSM backlink target index out of bounds");
        }
        let target = unsafe { parts.nodes.add(selection.node_idx as usize) };
        let target_lock = unsafe { ptr::addr_of_mut!((*target).lock) };
        unsafe { (locks.acquire_exclusive)(target_lock) };
        let target_state = unsafe { (*target).insert_state.value };
        if target_state != EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY {
            unsafe { (locks.release)(target_lock) };
            continue;
        }

        let Some((start, end)) = insert::backlink_slot_bounds(
            config.m,
            unsafe { (*target).neighbor_slot_count as usize },
            selection.layer,
        ) else {
            unsafe { (locks.release)(target_lock) };
            continue;
        };
        let layer_slice = unsafe { &mut concurrent_dsm_node_slots_mut(parts, target)[start..end] };
        if layer_slice.contains(&new_node_idx) {
            unsafe { (locks.release)(target_lock) };
            continue;
        }
        if let Some(slot) = layer_slice
            .iter_mut()
            .find(|slot| **slot == EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX)
        {
            *slot = new_node_idx;
            unsafe { (locks.release)(target_lock) };
            continue;
        }

        scratch.query_scores.begin_query();
        let mut candidates = layer_slice
            .iter()
            .copied()
            .filter(|neighbor_idx| *neighbor_idx != EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX)
            .map(|neighbor_idx| insert::ScoredBacklinkNode {
                node: neighbor_idx,
                score: unsafe {
                    score_concurrent_dsm_code(
                        parts,
                        layout,
                        config,
                        selection.node_idx,
                        neighbor_idx,
                        scratch,
                    )
                },
                is_new: false,
            })
            .collect::<Vec<_>>();
        candidates.push(insert::ScoredBacklinkNode {
            node: new_node_idx,
            score: unsafe {
                score_concurrent_dsm_code(
                    parts,
                    layout,
                    config,
                    selection.node_idx,
                    new_node_idx,
                    scratch,
                )
            },
            is_new: true,
        });
        let replacement =
            insert::select_best_backlink_candidates(candidates, layer_slice.len(), u32::cmp);
        if replacement.contains(&new_node_idx) {
            layer_slice.copy_from_slice(&replacement);
        }
        unsafe { (locks.release)(target_lock) };
    }
}

unsafe fn score_concurrent_dsm_code(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    query_idx: u32,
    candidate_idx: u32,
    scratch: &mut EcHnswConcurrentDsmInsertScratch,
) -> f32 {
    score_concurrent_dsm_code_with_cache(
        parts,
        layout,
        config,
        query_idx,
        candidate_idx,
        &mut scratch.query_scores,
    )
}

unsafe fn score_concurrent_dsm_code_with_cache(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    config: EcHnswConcurrentDsmInsertConfig,
    query_idx: u32,
    candidate_idx: u32,
    cache: &mut EcHnswConcurrentDsmQueryScoreCache,
) -> f32 {
    if query_idx >= layout.node_count || candidate_idx >= layout.node_count {
        pgrx::error!("concurrent DSM graph score node index out of bounds");
    }
    if let Some(score) = cache.get(candidate_idx) {
        return score;
    }
    let query_code = unsafe { concurrent_dsm_code_for_node(parts, layout, query_idx) };
    let candidate_code = unsafe { concurrent_dsm_code_for_node(parts, layout, candidate_idx) };
    let score = -crate::score_code_inner_product(
        config.dimensions,
        config.bits,
        config.seed,
        query_code,
        candidate_code,
    );
    cache.insert(candidate_idx, score);
    score
}

unsafe fn concurrent_dsm_code_for_node(
    parts: EcHnswConcurrentDsmGraphParts,
    layout: EcHnswConcurrentDsmGraphLayout,
    node_idx: u32,
) -> &'static [u8] {
    if node_idx >= layout.node_count {
        pgrx::error!("concurrent DSM code node index out of bounds");
    }
    let code_len = layout.code_len as usize;
    let start = (node_idx as usize)
        .checked_mul(code_len)
        .unwrap_or_else(|| pgrx::error!("concurrent DSM code offset overflow"));
    unsafe { slice::from_raw_parts(parts.codes.add(start), code_len) }
}

unsafe fn concurrent_dsm_node_slots<'a>(
    parts: EcHnswConcurrentDsmGraphParts,
    node: *const EcHnswConcurrentDsmNode,
) -> &'a [u32] {
    unsafe {
        slice::from_raw_parts(
            parts
                .neighbor_slots
                .add((*node).neighbor_slot_offset as usize),
            (*node).neighbor_slot_count as usize,
        )
    }
}

unsafe fn concurrent_dsm_node_slots_mut<'a>(
    parts: EcHnswConcurrentDsmGraphParts,
    node: *mut EcHnswConcurrentDsmNode,
) -> &'a mut [u32] {
    unsafe {
        slice::from_raw_parts_mut(
            parts
                .neighbor_slots
                .add((*node).neighbor_slot_offset as usize),
            (*node).neighbor_slot_count as usize,
        )
    }
}

unsafe fn concurrent_dsm_lwlock_acquire_shared(lock: *mut pg_sys::LWLock) {
    unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LWLockMode::LW_SHARED) };
}

unsafe fn concurrent_dsm_lwlock_acquire_exclusive(lock: *mut pg_sys::LWLock) {
    unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LWLockMode::LW_EXCLUSIVE) };
}

unsafe fn concurrent_dsm_lwlock_release(lock: *mut pg_sys::LWLock) {
    unsafe { pg_sys::LWLockRelease(lock) };
}

impl EcHnswParallelBuildSharedHeader {
    pub(super) fn new(
        plan: EcHnswParallelBuildPlan,
        heaprelid: pg_sys::Oid,
        indexrelid: pg_sys::Oid,
        is_concurrent: bool,
    ) -> Self {
        Self {
            magic: EC_HNSW_PARALLEL_BUILD_MAGIC,
            version: EC_HNSW_PARALLEL_BUILD_VERSION,
            requested_workers: checked_u16(plan.requested_workers, "requested workers"),
            participant_count: checked_u16(plan.participant_count, "participant count"),
            flags: 0,
            heaprelid,
            indexrelid,
            is_concurrent,
            reserved0: [0; 3],
            workersdonecv: pg_sys::ConditionVariable::default(),
            mutex: 0,
            nparticipantsdone: 0,
            scanned_heap_tuples: 0.0,
            encoded_index_tuples: 0.0,
        }
    }

    pub(super) fn record_worker_counts(&mut self, heap_tuples: f64, index_tuples: f64) {
        self.nparticipantsdone += 1;
        self.scanned_heap_tuples += heap_tuples;
        self.encoded_index_tuples += index_tuples;
    }

    pub(super) fn scanned_heap_tuples(&self) -> f64 {
        self.scanned_heap_tuples
    }

    pub(super) fn encoded_index_tuples(&self) -> f64 {
        self.encoded_index_tuples
    }

    fn validate(&self) {
        if self.magic != EC_HNSW_PARALLEL_BUILD_MAGIC
            || self.version != EC_HNSW_PARALLEL_BUILD_VERSION
        {
            pgrx::error!("ec_hnsw parallel build worker saw incompatible shared state");
        }
    }
}

fn checked_u16(value: i32, field: &str) -> u16 {
    u16::try_from(value).unwrap_or_else(|_| panic!("parallel build {field} should fit in u16"))
}

pub(super) fn reset_debug_last_parallel_build_workers_launched() {
    LAST_PARALLEL_BUILD_WORKERS_LAUNCHED.store(0, Ordering::Release);
}

pub(crate) fn debug_last_parallel_build_workers_launched() -> i32 {
    LAST_PARALLEL_BUILD_WORKERS_LAUNCHED.load(Ordering::Acquire)
}

pub(super) unsafe fn try_parallel_build(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    state: &mut build::BuildState,
    plan: EcHnswParallelBuildPlan,
) -> Option<EcHnswParallelBuildResult> {
    if plan.uses_serial_build_path() || state.options.build_source_column.is_some() {
        return None;
    }

    let begin_start = Instant::now();
    let mut leader = unsafe {
        EcHnswParallelBuildLeader::begin(heap_relation, index_relation, index_info, plan)
    }?;
    let begin_us = elapsed_us(begin_start);

    let drain_start = Instant::now();
    let mut worker_tuples = Vec::new();
    unsafe { leader.drain_worker_messages(&mut worker_tuples) };
    unsafe { leader.finish() };
    let drain_us = elapsed_us(drain_start);

    let sort_push_start = Instant::now();
    worker_tuples.sort_by_key(build_tuple_heap_tid_key);
    for tuple in worker_tuples {
        state.push(tuple);
    }
    let sort_push_us = elapsed_us(sort_push_start);

    Some(EcHnswParallelBuildResult {
        heap_tuples: state.scanned_tuples as f64,
        begin_us,
        drain_us,
        sort_push_us,
    })
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(super) struct EcHnswParallelBuildResult {
    pub(super) heap_tuples: f64,
    pub(super) begin_us: u64,
    pub(super) drain_us: u64,
    pub(super) sort_push_us: u64,
}

struct EcHnswParallelBuildLeader {
    pcxt: *mut pg_sys::ParallelContext,
    snapshot: pg_sys::Snapshot,
    unregister_snapshot: bool,
    queue_handles: Vec<*mut pg_sys::shm_mq_handle>,
    walusage: *mut pg_sys::WalUsage,
    bufferusage: *mut pg_sys::BufferUsage,
}

impl EcHnswParallelBuildLeader {
    unsafe fn begin(
        heap_relation: pg_sys::Relation,
        index_relation: pg_sys::Relation,
        index_info: *mut pg_sys::IndexInfo,
        plan: EcHnswParallelBuildPlan,
    ) -> Option<Self> {
        debug_assert!(plan.requested_workers > 0);
        unsafe { pg_sys::EnterParallelMode() };

        let pcxt = unsafe {
            pg_sys::CreateParallelContext(
                EC_HNSW_PARALLEL_BUILD_LIBRARY.as_ptr().cast(),
                EC_HNSW_PARALLEL_BUILD_ENTRYPOINT.as_ptr().cast(),
                plan.requested_workers,
            )
        };
        if pcxt.is_null() {
            unsafe { pg_sys::ExitParallelMode() };
            return None;
        }

        let is_concurrent = unsafe { !index_info.is_null() && (*index_info).ii_Concurrent };
        let snapshot = if is_concurrent {
            unsafe { pg_sys::RegisterSnapshot(pg_sys::GetTransactionSnapshot()) }
        } else {
            ptr::addr_of_mut!(pg_sys::SnapshotAnyData)
        };
        let unregister_snapshot = is_concurrent;

        let shared_bytes = unsafe { parallel_build_shared_workspace_size(heap_relation, snapshot) };
        unsafe {
            estimate_chunk(&mut (*pcxt).estimator, shared_bytes);
            estimate_keys(&mut (*pcxt).estimator, 1);
            for _ in 0..plan.requested_workers {
                estimate_chunk(&mut (*pcxt).estimator, EC_HNSW_PARALLEL_BUILD_QUEUE_BYTES);
                estimate_keys(&mut (*pcxt).estimator, 1);
            }
            estimate_chunk(
                &mut (*pcxt).estimator,
                checked_mul_size(
                    size_of::<pg_sys::WalUsage>() as pg_sys::Size,
                    plan.requested_workers as pg_sys::Size,
                    "parallel build WAL usage estimate",
                ),
            );
            estimate_keys(&mut (*pcxt).estimator, 1);
            estimate_chunk(
                &mut (*pcxt).estimator,
                checked_mul_size(
                    size_of::<pg_sys::BufferUsage>() as pg_sys::Size,
                    plan.requested_workers as pg_sys::Size,
                    "parallel build buffer usage estimate",
                ),
            );
            estimate_keys(&mut (*pcxt).estimator, 1);
        }

        unsafe { pg_sys::InitializeParallelDSM(pcxt) };
        if unsafe { (*pcxt).seg.is_null() } {
            if unregister_snapshot {
                unsafe { pg_sys::UnregisterSnapshot(snapshot) };
            }
            unsafe {
                pg_sys::DestroyParallelContext(pcxt);
                pg_sys::ExitParallelMode();
            }
            return None;
        }

        let shared = unsafe {
            pg_sys::shm_toc_allocate((*pcxt).toc, shared_bytes)
                .cast::<EcHnswParallelBuildSharedHeader>()
        };
        unsafe {
            ptr::write(
                shared,
                EcHnswParallelBuildSharedHeader::new(
                    plan,
                    (*heap_relation).rd_id,
                    (*index_relation).rd_id,
                    is_concurrent,
                ),
            );
            pg_sys::ConditionVariableInit(&mut (*shared).workersdonecv);
            pg_sys::SpinLockInit(&mut (*shared).mutex);
            pg_sys::table_parallelscan_initialize(
                heap_relation,
                parallel_table_scan_from_shared(shared),
                snapshot,
            );
            pg_sys::shm_toc_insert(
                (*pcxt).toc,
                PARALLEL_KEY_EC_HNSW_BUILD_SHARED,
                shared.cast(),
            );
        }

        unsafe {
            for worker_index in 0..plan.requested_workers {
                let mq = pg_sys::shm_mq_create(
                    pg_sys::shm_toc_allocate((*pcxt).toc, EC_HNSW_PARALLEL_BUILD_QUEUE_BYTES),
                    EC_HNSW_PARALLEL_BUILD_QUEUE_BYTES,
                );
                pg_sys::shm_mq_set_receiver(mq, pg_sys::MyProc);
                pg_sys::shm_toc_insert((*pcxt).toc, queue_key(worker_index), mq.cast::<c_void>());
            }
        }

        let walusage = unsafe {
            pg_sys::shm_toc_allocate(
                (*pcxt).toc,
                checked_mul_size(
                    size_of::<pg_sys::WalUsage>() as pg_sys::Size,
                    plan.requested_workers as pg_sys::Size,
                    "parallel build WAL usage allocation",
                ),
            )
            .cast::<pg_sys::WalUsage>()
        };
        let bufferusage = unsafe {
            pg_sys::shm_toc_allocate(
                (*pcxt).toc,
                checked_mul_size(
                    size_of::<pg_sys::BufferUsage>() as pg_sys::Size,
                    plan.requested_workers as pg_sys::Size,
                    "parallel build buffer usage allocation",
                ),
            )
            .cast::<pg_sys::BufferUsage>()
        };
        unsafe {
            pg_sys::shm_toc_insert((*pcxt).toc, PARALLEL_KEY_EC_HNSW_WAL_USAGE, walusage.cast());
            pg_sys::shm_toc_insert(
                (*pcxt).toc,
                PARALLEL_KEY_EC_HNSW_BUFFER_USAGE,
                bufferusage.cast(),
            );
        }

        unsafe { pg_sys::LaunchParallelWorkers(pcxt) };
        let workers_launched = unsafe { (*pcxt).nworkers_launched };
        LAST_PARALLEL_BUILD_WORKERS_LAUNCHED.store(workers_launched, Ordering::Release);

        let mut leader = Self {
            pcxt,
            snapshot,
            unregister_snapshot,
            queue_handles: Vec::with_capacity(workers_launched.max(0) as usize),
            walusage,
            bufferusage,
        };

        if workers_launched == 0 {
            unsafe { leader.finish() };
            return None;
        }

        unsafe {
            for worker_index in 0..workers_launched {
                let mq = pg_sys::shm_toc_lookup((*pcxt).toc, queue_key(worker_index), false)
                    .cast::<pg_sys::shm_mq>();
                let worker_info = (*pcxt).worker.add(worker_index as usize);
                let handle = pg_sys::shm_mq_attach(mq, (*pcxt).seg, (*worker_info).bgwhandle);
                leader.queue_handles.push(handle);
            }
            pg_sys::WaitForParallelWorkersToAttach(pcxt);
        }

        Some(leader)
    }

    unsafe fn drain_worker_messages(&mut self, tuples: &mut Vec<build::BuildTuple>) {
        let mut done = vec![false; self.queue_handles.len()];
        let mut done_count = 0_usize;

        while done_count < self.queue_handles.len() {
            let mut made_progress = false;

            for (queue_index, queue_handle) in self.queue_handles.iter().copied().enumerate() {
                if done[queue_index] {
                    continue;
                }

                loop {
                    let mut nbytes = 0_usize;
                    let mut data = ptr::null_mut::<c_void>();
                    let result = unsafe {
                        pg_sys::shm_mq_receive(queue_handle, &mut nbytes, &mut data, true)
                    };

                    match result {
                        pg_sys::shm_mq_result::SHM_MQ_SUCCESS => {
                            made_progress = true;
                            if data.is_null() || nbytes == 0 {
                                pgrx::error!("ec_hnsw parallel build worker sent an empty message");
                            }
                            let bytes = unsafe { slice::from_raw_parts(data.cast::<u8>(), nbytes) };
                            match decode_worker_message(bytes) {
                                EcHnswParallelBuildWorkerMessage::Tuple(tuple) => {
                                    tuples.push(tuple)
                                }
                                EcHnswParallelBuildWorkerMessage::Done => {
                                    done[queue_index] = true;
                                    done_count += 1;
                                    break;
                                }
                            }
                        }
                        pg_sys::shm_mq_result::SHM_MQ_WOULD_BLOCK => break,
                        pg_sys::shm_mq_result::SHM_MQ_DETACHED => {
                            done[queue_index] = true;
                            done_count += 1;
                            break;
                        }
                        _ => pgrx::error!("ec_hnsw parallel build saw unknown shm_mq result"),
                    }
                }
            }

            if done_count < self.queue_handles.len() && !made_progress {
                unsafe {
                    pg_sys::ProcessInterrupts();
                    pg_sys::pg_usleep(1000);
                }
            }
        }
    }

    unsafe fn finish(self) {
        unsafe { pg_sys::WaitForParallelWorkersToFinish(self.pcxt) };

        let launched = unsafe { (*self.pcxt).nworkers_launched.max(0) as usize };
        for worker_index in 0..launched {
            unsafe {
                pg_sys::InstrAccumParallelQuery(
                    self.bufferusage.add(worker_index),
                    self.walusage.add(worker_index),
                );
            }
        }

        if self.unregister_snapshot {
            unsafe { pg_sys::UnregisterSnapshot(self.snapshot) };
        }

        unsafe {
            pg_sys::DestroyParallelContext(self.pcxt);
            pg_sys::ExitParallelMode();
        }
    }
}

struct EcHnswParallelBuildWorkerScanState {
    queue_handle: *mut pg_sys::shm_mq_handle,
    indexed_vector_kind: source::IndexedVectorKind,
    encoded_tuples: u64,
}

unsafe extern "C-unwind" fn ec_hnsw_parallel_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = &mut *state.cast::<EcHnswParallelBuildWorkerScanState>();
            let heap_tid = shared::decode_heap_tid(tid);
            let tuple =
                build::build_heap_tuple(values, isnull, heap_tid, state.indexed_vector_kind);
            send_build_tuple_message(state.queue_handle, &tuple);
            state.encoded_tuples += tuple.heap_tids.len() as u64;
        })
    }
}

#[pgrx::pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ec_hnsw_parallel_build_main(
    seg: *mut pg_sys::dsm_segment,
    toc: *mut pg_sys::shm_toc,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            parallel_build_worker_main(seg, toc);
        })
    }
}

unsafe fn parallel_build_worker_main(seg: *mut pg_sys::dsm_segment, toc: *mut pg_sys::shm_toc) {
    let shared = unsafe {
        pg_sys::shm_toc_lookup(toc, PARALLEL_KEY_EC_HNSW_BUILD_SHARED, false)
            .cast::<EcHnswParallelBuildSharedHeader>()
    };
    unsafe { (*shared).validate() };

    let worker_number = unsafe { pg_sys::ParallelWorkerNumber };
    if worker_number < 0 {
        pgrx::error!("ec_hnsw parallel build worker started without a worker number");
    }

    let queue = unsafe {
        pg_sys::shm_toc_lookup(toc, queue_key(worker_number), false).cast::<pg_sys::shm_mq>()
    };
    unsafe { pg_sys::shm_mq_set_sender(queue, pg_sys::MyProc) };
    let queue_handle = unsafe { pg_sys::shm_mq_attach(queue, seg, ptr::null_mut()) };

    let (heap_lockmode, index_lockmode) = if unsafe { (*shared).is_concurrent } {
        (
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
            pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        )
    } else {
        (
            pg_sys::ShareLock as pg_sys::LOCKMODE,
            pg_sys::AccessExclusiveLock as pg_sys::LOCKMODE,
        )
    };

    let heap_relation = unsafe { pg_sys::table_open((*shared).heaprelid, heap_lockmode) };
    let index_relation = unsafe { pg_sys::index_open((*shared).indexrelid, index_lockmode) };

    unsafe { pg_sys::InstrStartParallelQuery() };

    let mut worker_state = EcHnswParallelBuildWorkerScanState {
        queue_handle,
        indexed_vector_kind: build::BuildState::new(index_relation).indexed_vector_kind,
        encoded_tuples: 0,
    };

    let index_info = unsafe { pg_sys::BuildIndexInfo(index_relation) };
    unsafe {
        (*index_info).ii_Concurrent = (*shared).is_concurrent;
    }
    let scan = unsafe {
        pg_sys::table_beginscan_parallel(heap_relation, parallel_table_scan_from_shared(shared))
    };
    let scanned_tuples = unsafe {
        pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            index_info,
            true,
            false,
            Some(ec_hnsw_parallel_build_callback),
            (&mut worker_state as *mut EcHnswParallelBuildWorkerScanState).cast(),
            scan,
        )
    };

    send_done_message(queue_handle);

    unsafe {
        pg_sys::SpinLockAcquire(&mut (*shared).mutex);
        (*shared).record_worker_counts(scanned_tuples, worker_state.encoded_tuples as f64);
        pg_sys::SpinLockRelease(&mut (*shared).mutex);
        pg_sys::ConditionVariableSignal(&mut (*shared).workersdonecv);
    }

    let bufferusage = unsafe {
        pg_sys::shm_toc_lookup(toc, PARALLEL_KEY_EC_HNSW_BUFFER_USAGE, false)
            .cast::<pg_sys::BufferUsage>()
    };
    let walusage = unsafe {
        pg_sys::shm_toc_lookup(toc, PARALLEL_KEY_EC_HNSW_WAL_USAGE, false)
            .cast::<pg_sys::WalUsage>()
    };
    unsafe {
        pg_sys::InstrEndParallelQuery(
            bufferusage.add(worker_number as usize),
            walusage.add(worker_number as usize),
        );
        pg_sys::index_close(index_relation, index_lockmode);
        pg_sys::table_close(heap_relation, heap_lockmode);
    }
}

enum EcHnswParallelBuildWorkerMessage {
    Tuple(build::BuildTuple),
    Done,
}

fn send_build_tuple_message(queue_handle: *mut pg_sys::shm_mq_handle, tuple: &build::BuildTuple) {
    let message = encode_build_tuple_message(tuple);
    unsafe { send_worker_message(queue_handle, &message) };
}

fn send_done_message(queue_handle: *mut pg_sys::shm_mq_handle) {
    let message = [BUILD_DONE_MESSAGE];
    unsafe { send_worker_message(queue_handle, &message) };
}

unsafe fn send_worker_message(queue_handle: *mut pg_sys::shm_mq_handle, message: &[u8]) {
    let result = unsafe {
        pg_sys::shm_mq_send(
            queue_handle,
            message.len() as pg_sys::Size,
            message.as_ptr().cast(),
            false,
            true,
        )
    };
    match result {
        pg_sys::shm_mq_result::SHM_MQ_SUCCESS => {}
        pg_sys::shm_mq_result::SHM_MQ_DETACHED => {
            pgrx::error!("ec_hnsw parallel build worker queue detached")
        }
        _ => pgrx::error!("ec_hnsw parallel build worker could not send a tuple message"),
    }
}

fn encode_build_tuple_message(tuple: &build::BuildTuple) -> Vec<u8> {
    if tuple.heap_tids.len() != 1 {
        pgrx::error!("ec_hnsw parallel build workers must emit one heap tuple per message");
    }
    let heap_tid = tuple.heap_tids[0];
    let source_len = tuple
        .source_vector
        .as_ref()
        .map_or(0_usize, |source| source.len());
    let source_count = if tuple.source_vector.is_some() {
        tuple.source_count
    } else {
        0
    };

    let code_len = checked_u32(tuple.code.len(), "parallel build code length");
    let source_len_u32 = checked_u32(source_len, "parallel build source vector length");
    let source_count_u32 = checked_u32(source_count, "parallel build source count");

    let mut message = Vec::with_capacity(
        1 + 1 + 4 + 2 + 2 + 8 + 4 + 4 + 4 + 4 + tuple.code.len() + source_len * 4,
    );
    message.push(BUILD_TUPLE_MESSAGE);
    message.push(tuple.bits);
    message.extend_from_slice(&heap_tid.block_number.to_le_bytes());
    message.extend_from_slice(&heap_tid.offset_number.to_le_bytes());
    message.extend_from_slice(&tuple.dimensions.to_le_bytes());
    message.extend_from_slice(&tuple.seed.to_le_bytes());
    message.extend_from_slice(&tuple.gamma.to_bits().to_le_bytes());
    message.extend_from_slice(&code_len.to_le_bytes());
    message.extend_from_slice(&source_len_u32.to_le_bytes());
    message.extend_from_slice(&source_count_u32.to_le_bytes());
    message.extend_from_slice(&tuple.code);
    if let Some(source_vector) = &tuple.source_vector {
        for value in source_vector {
            message.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
    message
}

fn decode_worker_message(bytes: &[u8]) -> EcHnswParallelBuildWorkerMessage {
    let mut cursor = 0_usize;
    let kind = read_u8(bytes, &mut cursor);
    match kind {
        BUILD_DONE_MESSAGE => {
            if cursor != bytes.len() {
                pgrx::error!("ec_hnsw parallel build done message had trailing bytes");
            }
            EcHnswParallelBuildWorkerMessage::Done
        }
        BUILD_TUPLE_MESSAGE => {
            let bits = read_u8(bytes, &mut cursor);
            let block_number = read_u32(bytes, &mut cursor);
            let offset_number = read_u16(bytes, &mut cursor);
            let dimensions = read_u16(bytes, &mut cursor);
            let seed = read_u64(bytes, &mut cursor);
            let gamma = f32::from_bits(read_u32(bytes, &mut cursor));
            let code_len = read_u32(bytes, &mut cursor) as usize;
            let source_len = read_u32(bytes, &mut cursor) as usize;
            let source_count = read_u32(bytes, &mut cursor) as usize;
            let code = read_bytes(bytes, &mut cursor, code_len).to_vec();
            let source_vector = if source_len == 0 {
                if source_count != 0 {
                    pgrx::error!("ec_hnsw parallel build source count without source vector");
                }
                None
            } else {
                if source_count == 0 {
                    pgrx::error!("ec_hnsw parallel build source vector without source count");
                }
                let mut source = Vec::with_capacity(source_len);
                for _ in 0..source_len {
                    source.push(f32::from_bits(read_u32(bytes, &mut cursor)));
                }
                Some(source)
            };
            if cursor != bytes.len() {
                pgrx::error!("ec_hnsw parallel build tuple message had trailing bytes");
            }

            EcHnswParallelBuildWorkerMessage::Tuple(build::BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number,
                    offset_number,
                }],
                dimensions,
                bits,
                seed,
                gamma,
                code,
                source_vector,
                source_count,
            })
        }
        _ => pgrx::error!("ec_hnsw parallel build worker sent an unknown message kind"),
    }
}

fn build_tuple_heap_tid_key(tuple: &build::BuildTuple) -> (u32, u16) {
    let heap_tid = tuple
        .heap_tids
        .first()
        .copied()
        .unwrap_or_else(|| pgrx::error!("ec_hnsw parallel build tuple had no heap tid"));
    (heap_tid.block_number, heap_tid.offset_number)
}

fn read_u8(bytes: &[u8], cursor: &mut usize) -> u8 {
    let value = *read_bytes(bytes, cursor, 1)
        .first()
        .expect("read_bytes returned exactly one byte");
    value
}

fn read_u16(bytes: &[u8], cursor: &mut usize) -> u16 {
    let mut raw = [0_u8; 2];
    raw.copy_from_slice(read_bytes(bytes, cursor, 2));
    u16::from_le_bytes(raw)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> u32 {
    let mut raw = [0_u8; 4];
    raw.copy_from_slice(read_bytes(bytes, cursor, 4));
    u32::from_le_bytes(raw)
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> u64 {
    let mut raw = [0_u8; 8];
    raw.copy_from_slice(read_bytes(bytes, cursor, 8));
    u64::from_le_bytes(raw)
}

fn read_bytes<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> &'a [u8] {
    let end = cursor
        .checked_add(len)
        .unwrap_or_else(|| pgrx::error!("ec_hnsw parallel build message cursor overflow"));
    if end > bytes.len() {
        pgrx::error!("ec_hnsw parallel build worker sent a truncated message");
    }
    let out = &bytes[*cursor..end];
    *cursor = end;
    out
}

fn checked_u32(value: usize, field: &str) -> u32 {
    u32::try_from(value).unwrap_or_else(|_| pgrx::error!("{field} does not fit in u32"))
}

fn checked_graph_u32(value: usize, field: &str) -> u32 {
    let value = checked_u32(value, field);
    if value == u32::MAX {
        pgrx::error!("{field} must leave u32::MAX reserved as the invalid graph index");
    }
    value
}

unsafe fn parallel_build_shared_workspace_size(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
) -> pg_sys::Size {
    checked_add_size(
        bufferalign(size_of::<EcHnswParallelBuildSharedHeader>() as pg_sys::Size),
        unsafe { pg_sys::table_parallelscan_estimate(heap_relation, snapshot) },
        "parallel build shared workspace size",
    )
}

unsafe fn parallel_table_scan_from_shared(
    shared: *mut EcHnswParallelBuildSharedHeader,
) -> pg_sys::ParallelTableScanDesc {
    unsafe {
        shared
            .cast::<u8>()
            .add(bufferalign(
                size_of::<EcHnswParallelBuildSharedHeader>() as pg_sys::Size
            ))
            .cast()
    }
}

unsafe fn estimate_chunk(estimator: *mut pg_sys::shm_toc_estimator, size: pg_sys::Size) {
    unsafe {
        (*estimator).space_for_chunks = checked_add_size(
            (*estimator).space_for_chunks,
            bufferalign(size),
            "parallel build DSM chunk estimate",
        );
    }
}

unsafe fn estimate_keys(estimator: *mut pg_sys::shm_toc_estimator, keys: pg_sys::Size) {
    unsafe {
        (*estimator).number_of_keys = checked_add_size(
            (*estimator).number_of_keys,
            keys,
            "parallel build DSM key estimate",
        );
    }
}

fn queue_key(worker_index: i32) -> u64 {
    if worker_index < 0 {
        pgrx::error!("ec_hnsw parallel build worker index was negative");
    }
    PARALLEL_KEY_EC_HNSW_QUEUE_BASE + worker_index as u64
}

fn bufferalign(size: pg_sys::Size) -> pg_sys::Size {
    typealign(pg_sys::ALIGNOF_BUFFER as pg_sys::Size, size)
}

fn typealign(alignment: pg_sys::Size, size: pg_sys::Size) -> pg_sys::Size {
    debug_assert!(alignment.is_power_of_two());
    (size + alignment - 1) & !(alignment - 1)
}

fn checked_add_size(lhs: pg_sys::Size, rhs: pg_sys::Size, context: &str) -> pg_sys::Size {
    lhs.checked_add(rhs)
        .unwrap_or_else(|| panic!("{context} overflowed pg_sys::Size"))
}

fn checked_mul_size(lhs: pg_sys::Size, rhs: pg_sys::Size, context: &str) -> pg_sys::Size {
    lhs.checked_mul(rhs)
        .unwrap_or_else(|| panic!("{context} overflowed pg_sys::Size"))
}

fn elapsed_us(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::alloc::{alloc_zeroed, dealloc, Layout};
    use std::collections::HashMap;
    use std::slice;

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
        assert_eq!(plan.participant_count, 3);
        assert!(!plan.leader_participates);
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
        let mut shared = EcHnswParallelBuildSharedHeader::new(
            plan,
            pg_sys::InvalidOid,
            pg_sys::InvalidOid,
            false,
        );

        shared.record_worker_counts(11.0, 7.0);
        shared.record_worker_counts(13.0, 5.0);

        assert_eq!(shared.scanned_heap_tuples(), 24.0);
        assert_eq!(shared.encoded_index_tuples(), 12.0);
    }

    #[test]
    fn concurrent_dsm_node_layout_plan_assigns_flat_neighbor_slices() {
        let levels = build::NativeBuildLevels::from_levels(vec![0, 2]);
        let node_plan = EcHnswConcurrentDsmNodeLayoutPlan::for_levels(&levels, 2);

        assert_eq!(node_plan.total_neighbor_slots, 12);
        assert_eq!(
            node_plan.nodes,
            vec![
                EcHnswConcurrentDsmNodeLayout {
                    level: 0,
                    neighbor_slot_offset: 0,
                    neighbor_slot_count: 4,
                },
                EcHnswConcurrentDsmNodeLayout {
                    level: 2,
                    neighbor_slot_offset: 4,
                    neighbor_slot_count: 8,
                },
            ]
        );
    }

    #[test]
    fn concurrent_dsm_node_layout_plan_handles_empty_levels() {
        let levels = build::NativeBuildLevels::from_levels(Vec::new());
        let node_plan = EcHnswConcurrentDsmNodeLayoutPlan::for_levels(&levels, 2);

        assert_eq!(node_plan.total_neighbor_slots, 0);
        assert!(node_plan.nodes.is_empty());
    }

    #[test]
    fn concurrent_dsm_node_partitions_split_remainder_across_early_participants() {
        let partitions = concurrent_dsm_node_partitions(10, 3);

        assert_eq!(
            partitions,
            vec![
                EcHnswConcurrentDsmNodePartition {
                    participant_index: 0,
                    start_node_idx: 0,
                    end_node_idx: 4,
                },
                EcHnswConcurrentDsmNodePartition {
                    participant_index: 1,
                    start_node_idx: 4,
                    end_node_idx: 7,
                },
                EcHnswConcurrentDsmNodePartition {
                    participant_index: 2,
                    start_node_idx: 7,
                    end_node_idx: 10,
                },
            ]
        );
        assert!(partitions[1].contains(4));
        assert!(partitions[1].contains(6));
        assert!(!partitions[1].contains(7));
    }

    #[test]
    fn concurrent_dsm_node_partitions_allow_empty_tail_participants() {
        let partitions = concurrent_dsm_node_partitions(2, 4);

        assert_eq!(
            partitions,
            vec![
                EcHnswConcurrentDsmNodePartition {
                    participant_index: 0,
                    start_node_idx: 0,
                    end_node_idx: 1,
                },
                EcHnswConcurrentDsmNodePartition {
                    participant_index: 1,
                    start_node_idx: 1,
                    end_node_idx: 2,
                },
                EcHnswConcurrentDsmNodePartition {
                    participant_index: 2,
                    start_node_idx: 2,
                    end_node_idx: 2,
                },
                EcHnswConcurrentDsmNodePartition {
                    participant_index: 3,
                    start_node_idx: 2,
                    end_node_idx: 2,
                },
            ]
        );
        assert!(!partitions[1].is_empty());
        assert!(partitions[2].is_empty());
        assert!(partitions[3].is_empty());
    }

    #[test]
    #[should_panic]
    fn concurrent_dsm_node_partitions_reject_zero_participants() {
        let _ = concurrent_dsm_node_partitions(2, 0);
    }

    #[test]
    fn concurrent_dsm_code_corpus_packs_fixed_width_codes() {
        let tuples = vec![
            build_tuple_with_code(vec![1, 2, 3]),
            build_tuple_with_code(vec![4, 5, 6]),
        ];
        let corpus = EcHnswConcurrentDsmCodeCorpus::from_tuples(&tuples);

        assert_eq!(corpus.node_count, 2);
        assert_eq!(corpus.code_len, 3);
        assert_eq!(corpus.bytes, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(corpus.code_for_node(0), &[1, 2, 3]);
        assert_eq!(corpus.code_for_node(1), &[4, 5, 6]);
    }

    #[test]
    fn concurrent_dsm_code_corpus_handles_empty_input() {
        let corpus = EcHnswConcurrentDsmCodeCorpus::from_tuples(&[]);

        assert_eq!(corpus.node_count, 0);
        assert_eq!(corpus.code_len, 0);
        assert!(corpus.bytes.is_empty());
    }

    #[test]
    #[should_panic]
    fn concurrent_dsm_code_corpus_rejects_variable_width_codes() {
        let tuples = vec![
            build_tuple_with_code(vec![1, 2, 3]),
            build_tuple_with_code(vec![4, 5]),
        ];

        let _ = EcHnswConcurrentDsmCodeCorpus::from_tuples(&tuples);
    }

    #[test]
    fn concurrent_dsm_graph_layout_sums_slots_and_aligns_sections() {
        let levels = build::NativeBuildLevels::from_levels(vec![0, 2]);
        let layout = EcHnswConcurrentDsmGraphLayout::for_levels(&levels, 2, 6);
        let alignment = pg_sys::ALIGNOF_BUFFER as pg_sys::Size;

        assert_eq!(layout.node_count, 2);
        assert_eq!(layout.entry_idx, Some(1));
        assert_eq!(layout.max_level, 2);
        assert_eq!(layout.total_neighbor_slots, 12);
        assert_eq!(layout.code_len, 6);
        assert_eq!(layout.header_offset, 0);
        assert_eq!(
            layout.nodes_offset,
            bufferalign(size_of::<EcHnswConcurrentDsmGraphHeader>() as pg_sys::Size)
        );
        assert_eq!(layout.nodes_offset % alignment, 0);
        assert_eq!(layout.neighbor_slots_offset % alignment, 0);
        assert_eq!(layout.codes_offset % alignment, 0);
        assert!(layout.nodes_offset < layout.neighbor_slots_offset);
        assert!(layout.neighbor_slots_offset < layout.codes_offset);
        assert!(layout.codes_offset < layout.total_bytes);
    }

    #[test]
    fn concurrent_dsm_graph_layout_handles_empty_levels() {
        let levels = build::NativeBuildLevels::from_levels(Vec::new());
        let layout = EcHnswConcurrentDsmGraphLayout::for_levels(&levels, 2, 6);

        assert_eq!(layout.node_count, 0);
        assert_eq!(layout.entry_idx, None);
        assert_eq!(layout.max_level, 0);
        assert_eq!(layout.total_neighbor_slots, 0);
        assert_eq!(layout.code_len, 6);
        assert_eq!(
            layout.total_bytes,
            bufferalign(size_of::<EcHnswConcurrentDsmGraphHeader>() as pg_sys::Size)
        );
    }

    #[test]
    fn concurrent_dsm_preassembly_plan_composes_levels_slots_codes_and_layout() {
        let mut state = build_state(None);
        state.push(build_tuple_with_code(vec![1, 2, 3]));
        state.push(build_tuple_with_code(vec![4, 5, 6]));

        let plan = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);

        assert_eq!(plan.levels.levels.len(), 2);
        assert_eq!(plan.node_layout.nodes.len(), 2);
        assert_eq!(plan.code_corpus.node_count, 2);
        assert_eq!(plan.code_corpus.code_len, 3);
        assert_eq!(plan.code_corpus.bytes, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(plan.graph_layout.node_count, 2);
        assert_eq!(plan.graph_layout.code_len, 3);
        assert_eq!(
            plan.graph_layout.total_neighbor_slots,
            plan.node_layout.total_neighbor_slots
        );
    }

    #[test]
    fn concurrent_dsm_preassembly_plan_handles_empty_state() {
        let state = build_state(None);
        let plan = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);

        assert!(plan.levels.levels.is_empty());
        assert!(plan.node_layout.nodes.is_empty());
        assert_eq!(plan.code_corpus.node_count, 0);
        assert_eq!(plan.code_corpus.code_len, 0);
        assert_eq!(plan.graph_layout.node_count, 0);
        assert_eq!(plan.graph_layout.total_neighbor_slots, 0);
    }

    #[test]
    #[should_panic]
    fn concurrent_dsm_preassembly_plan_rejects_source_scored_builds() {
        let state = build_state(Some("source"));

        let _ = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);
    }

    #[test]
    fn concurrent_dsm_graph_image_initializes_header_nodes_slots_and_codes() {
        let mut state = build_state(None);
        state.push(build_tuple_with_code(vec![1, 2, 3]));
        state.push(build_tuple_with_code(vec![4, 5, 6]));
        let plan = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);

        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };

        unsafe {
            assert_eq!(
                parts.header.cast::<u8>().offset_from(buffer.ptr),
                plan.graph_layout.header_offset as isize
            );
            assert_eq!(
                parts.nodes.cast::<u8>().offset_from(buffer.ptr),
                plan.graph_layout.nodes_offset as isize
            );
            assert_eq!(
                parts.neighbor_slots.cast::<u8>().offset_from(buffer.ptr),
                plan.graph_layout.neighbor_slots_offset as isize
            );
            assert_eq!(
                parts.codes.offset_from(buffer.ptr),
                plan.graph_layout.codes_offset as isize
            );

            let header = &*parts.header;
            assert_eq!(header.node_count, 2);
            assert_eq!(header.entry_idx, plan.graph_layout.entry_idx.unwrap());
            assert_eq!(header.max_level, plan.graph_layout.max_level);
            assert_eq!(
                header.total_neighbor_slots,
                plan.node_layout.total_neighbor_slots
            );
            assert_eq!(header.code_len, 3);

            let nodes = slice::from_raw_parts(parts.nodes, plan.graph_layout.node_count as usize);
            for (node_idx, (actual, expected)) in
                nodes.iter().zip(plan.node_layout.nodes.iter()).enumerate()
            {
                assert_eq!(actual.level, expected.level);
                assert_eq!(actual.neighbor_slot_offset, expected.neighbor_slot_offset);
                assert_eq!(actual.neighbor_slot_count, expected.neighbor_slot_count);
                let expected_insert_state = if Some(node_idx as u32) == plan.graph_layout.entry_idx
                {
                    EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY
                } else {
                    EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED
                };
                assert_eq!(actual.insert_state.value, expected_insert_state);
                assert_eq!(actual.lock.tranche, TEST_LOCK_TRANCHE_ID);
                assert_eq!(actual.lock.waiters.head, pg_sys::INVALID_PROC_NUMBER);
                assert_eq!(actual.lock.waiters.tail, pg_sys::INVALID_PROC_NUMBER);
            }

            let neighbor_slots = slice::from_raw_parts(
                parts.neighbor_slots,
                plan.graph_layout.total_neighbor_slots as usize,
            );
            assert!(neighbor_slots
                .iter()
                .all(|slot| *slot == EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX));

            let codes = slice::from_raw_parts(parts.codes, plan.code_corpus.bytes.len());
            assert_eq!(codes, plan.code_corpus.bytes.as_slice());
        }
    }

    #[test]
    fn concurrent_dsm_graph_image_initializes_empty_graph_header() {
        let state = build_state(None);
        let plan = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);

        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };

        unsafe {
            let header = &*parts.header;
            assert_eq!(header.node_count, 0);
            assert_eq!(header.entry_idx, EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX);
            assert_eq!(header.max_level, 0);
            assert_eq!(header.total_neighbor_slots, 0);
            assert_eq!(header.code_len, 0);
        }
    }

    #[test]
    fn concurrent_dsm_graph_readback_builds_page_staging_nodes() {
        let mut state = build_state(None);
        state.push(build_tuple_with_code(vec![1, 2, 3]));
        state.push(build_tuple_with_code(vec![4, 5, 6]));
        state.push(build_tuple_with_code(vec![7, 8, 9]));
        let plan = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);
        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };

        let node_levels = unsafe {
            let nodes =
                slice::from_raw_parts_mut(parts.nodes, plan.graph_layout.node_count as usize);
            let slots = slice::from_raw_parts_mut(
                parts.neighbor_slots,
                plan.graph_layout.total_neighbor_slots as usize,
            );
            for node in nodes.iter_mut() {
                node.insert_state.value = EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY;
            }

            let node0_start = nodes[0].neighbor_slot_offset as usize;
            slots[node0_start] = 1;
            slots[node0_start + 1] = 1;
            slots[node0_start + 2] = 0;
            slots[node0_start + 3] = 2;

            let node1_start = nodes[1].neighbor_slot_offset as usize;
            slots[node1_start] = 0;

            nodes.iter().map(|node| node.level).collect::<Vec<_>>()
        };

        let build_nodes =
            unsafe { concurrent_dsm_graph_to_build_nodes(parts, plan.graph_layout, 2) };

        assert_eq!(build_nodes.len(), 3);
        assert_eq!(build_nodes[0].level, node_levels[0]);
        assert_eq!(
            &build_nodes[0].neighbor_slots[0..4],
            &[Some(1), Some(1), Some(0), Some(2)]
        );
        assert_eq!(build_nodes[0].score_neighbors, vec![1, 2]);
        assert_eq!(build_nodes[1].neighbor_slots[0], Some(0));
        assert_eq!(build_nodes[1].score_neighbors, vec![0]);
        assert!(build_nodes[2].neighbor_slots.iter().all(Option::is_none));
        assert!(build_nodes[2].score_neighbors.is_empty());
    }

    #[test]
    #[should_panic]
    fn concurrent_dsm_graph_readback_rejects_uninserted_nodes() {
        let mut state = build_state(None);
        state.push(build_tuple_with_code(vec![1, 2, 3]));
        state.push(build_tuple_with_code(vec![4, 5, 6]));
        let plan = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);
        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };

        let _ = unsafe { concurrent_dsm_graph_to_build_nodes(parts, plan.graph_layout, 2) };
    }

    #[test]
    #[should_panic]
    fn concurrent_dsm_graph_readback_rejects_out_of_range_neighbor() {
        let mut state = build_state(None);
        state.push(build_tuple_with_code(vec![1, 2, 3]));
        state.push(build_tuple_with_code(vec![4, 5, 6]));
        let plan = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);
        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };

        unsafe {
            {
                let nodes =
                    slice::from_raw_parts_mut(parts.nodes, plan.graph_layout.node_count as usize);
                let slots = slice::from_raw_parts_mut(
                    parts.neighbor_slots,
                    plan.graph_layout.total_neighbor_slots as usize,
                );
                for node in nodes.iter_mut() {
                    node.insert_state.value = EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY;
                }
                slots[nodes[0].neighbor_slot_offset as usize] = plan.graph_layout.node_count;
            }

            let _ = concurrent_dsm_graph_to_build_nodes(parts, plan.graph_layout, 2);
        }
    }

    #[test]
    fn concurrent_dsm_graph_insert_writes_forward_slots_and_backlinks() {
        let plan = preassembly_plan_with_levels(
            vec![1, 0, 0],
            vec![
                vec![0x11, 0x11, 0x11],
                vec![0x22, 0x22, 0x22],
                vec![0x33, 0x33, 0x33],
            ],
        );
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);
        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };
        let config = test_insert_config(4);
        let mut scratch = EcHnswConcurrentDsmInsertScratch::new(
            plan.graph_layout.node_count as usize,
            config.ef_construction,
            config.m,
        );

        let inserted_1 = unsafe {
            insert_concurrent_dsm_graph_node(
                parts,
                plan.graph_layout,
                config,
                1,
                &mut scratch,
                test_lock_ops(),
            )
        };
        let inserted_2 = unsafe {
            insert_concurrent_dsm_graph_node(
                parts,
                plan.graph_layout,
                config,
                2,
                &mut scratch,
                test_lock_ops(),
            )
        };

        assert!(inserted_1);
        assert!(inserted_2);
        let build_nodes =
            unsafe { concurrent_dsm_graph_to_build_nodes(parts, plan.graph_layout, config.m) };
        assert_eq!(build_nodes.len(), 3);
        assert!(build_nodes[1].neighbor_slots.contains(&Some(0)));
        assert!(build_nodes[2].neighbor_slots.iter().any(Option::is_some));
        assert!(build_nodes[0].neighbor_slots.contains(&Some(1)));
    }

    #[test]
    fn concurrent_dsm_graph_insert_skips_preinitialized_entry_node() {
        let plan = preassembly_plan_with_levels(
            vec![1, 0],
            vec![vec![0x11, 0x11, 0x11], vec![0x22, 0x22, 0x22]],
        );
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);
        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };
        let config = test_insert_config(4);
        let mut scratch = EcHnswConcurrentDsmInsertScratch::new(
            plan.graph_layout.node_count as usize,
            config.ef_construction,
            config.m,
        );

        let inserted = unsafe {
            insert_concurrent_dsm_graph_node(
                parts,
                plan.graph_layout,
                config,
                0,
                &mut scratch,
                test_lock_ops(),
            )
        };

        assert!(!inserted);
        unsafe {
            let entry = parts.nodes.add(0);
            let entry_slots = concurrent_dsm_node_slots(parts, entry);
            assert!(entry_slots
                .iter()
                .all(|slot| *slot == EC_HNSW_CONCURRENT_DSM_INVALID_NODE_IDX));
        }
    }

    #[test]
    fn concurrent_dsm_graph_partition_insert_covers_node_range_once() {
        let plan = preassembly_plan_with_levels(
            vec![1, 0, 0, 0],
            vec![
                vec![0x11, 0x11, 0x11],
                vec![0x22, 0x22, 0x22],
                vec![0x33, 0x33, 0x33],
                vec![0x44, 0x44, 0x44],
            ],
        );
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);
        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };
        let config = test_insert_config(4);
        let partitions = concurrent_dsm_node_partitions(plan.graph_layout.node_count, 2);
        let mut scratch = EcHnswConcurrentDsmInsertScratch::new(
            plan.graph_layout.node_count as usize,
            config.ef_construction,
            config.m,
        );

        let inserted_0 = unsafe {
            insert_concurrent_dsm_graph_partition(
                parts,
                plan.graph_layout,
                config,
                partitions[0],
                &mut scratch,
                test_lock_ops(),
            )
        };
        let inserted_1 = unsafe {
            insert_concurrent_dsm_graph_partition(
                parts,
                plan.graph_layout,
                config,
                partitions[1],
                &mut scratch,
                test_lock_ops(),
            )
        };
        let inserted_again = unsafe {
            insert_concurrent_dsm_graph_partition(
                parts,
                plan.graph_layout,
                config,
                partitions[0],
                &mut scratch,
                test_lock_ops(),
            )
        };

        assert_eq!(inserted_0, 1);
        assert_eq!(inserted_1, 2);
        assert_eq!(inserted_again, 0);
        let build_nodes =
            unsafe { concurrent_dsm_graph_to_build_nodes(parts, plan.graph_layout, config.m) };
        assert_eq!(build_nodes.len(), 4);
        assert!(build_nodes
            .iter()
            .skip(1)
            .all(|node| node.neighbor_slots.iter().any(Option::is_some)));
    }

    #[test]
    fn concurrent_dsm_graph_single_participant_stages_current_format_pages() {
        let mut state = build_state(None);
        state.options.storage_format = crate::quant::Family::TurboQuant;
        state.push(build_tuple_with_code(vec![0x11, 0x11, 0x11]));
        state.push(build_tuple_with_code(vec![0x22, 0x22, 0x22]));
        state.push(build_tuple_with_code(vec![0x33, 0x33, 0x33]));
        state.push(build_tuple_with_code(vec![0x44, 0x44, 0x44]));
        let plan = EcHnswConcurrentDsmPreassemblyPlan::for_state(&state);
        let buffer = AlignedDsmBuffer::new(plan.graph_layout.total_bytes);
        let parts = unsafe {
            initialize_concurrent_dsm_graph_image(
                buffer.as_mut_ptr(),
                &plan,
                test_initialize_node_lock,
            )
        };
        let config = EcHnswConcurrentDsmInsertConfig::for_state(&state);
        let mut scratch = EcHnswConcurrentDsmInsertScratch::new(
            plan.graph_layout.node_count as usize,
            config.ef_construction,
            config.m,
        );

        let inserted = unsafe {
            insert_concurrent_dsm_graph_participant(
                parts,
                plan.graph_layout,
                config,
                1,
                0,
                &mut scratch,
                test_lock_ops(),
            )
        };
        assert_eq!(inserted, plan.graph_layout.node_count.saturating_sub(1));

        let graph_nodes =
            unsafe { concurrent_dsm_graph_to_build_nodes(parts, plan.graph_layout, config.m) };
        let output = build::current_format_flush_output_from_graph_nodes(&state, &graph_nodes);

        assert_eq!(
            output.metadata.format_version,
            page::INDEX_FORMAT_V3_TURBO_HOT_COLD
        );
        assert_ne!(output.metadata.entry_point, page::ItemPointer::INVALID);
        let max_level = graph_nodes.iter().map(|node| node.level).max().unwrap_or(0);
        assert_eq!(output.metadata.max_level, max_level);
        assert_eq!(output.metadata.m, state.options.m as u16);
    }

    struct AlignedDsmBuffer {
        ptr: *mut u8,
        layout: Layout,
    }

    impl AlignedDsmBuffer {
        fn new(size: pg_sys::Size) -> Self {
            let layout = Layout::from_size_align(size, pg_sys::ALIGNOF_BUFFER as usize).unwrap();
            let ptr = unsafe { alloc_zeroed(layout) };
            assert!(!ptr.is_null());
            Self { ptr, layout }
        }

        fn as_mut_ptr(&self) -> *mut c_void {
            self.ptr.cast()
        }
    }

    impl Drop for AlignedDsmBuffer {
        fn drop(&mut self) {
            unsafe { dealloc(self.ptr, self.layout) };
        }
    }

    const TEST_LOCK_TRANCHE_ID: u16 = 4242;

    unsafe fn test_initialize_node_lock(lock: *mut pg_sys::LWLock) {
        unsafe {
            (*lock).tranche = TEST_LOCK_TRANCHE_ID;
            (*lock).state.value = 0;
            (*lock).waiters.head = pg_sys::INVALID_PROC_NUMBER;
            (*lock).waiters.tail = pg_sys::INVALID_PROC_NUMBER;
        }
    }

    fn test_lock_ops() -> EcHnswConcurrentDsmLockOps {
        EcHnswConcurrentDsmLockOps {
            acquire_shared: test_lock_noop,
            acquire_exclusive: test_lock_noop,
            release: test_lock_noop,
        }
    }

    unsafe fn test_lock_noop(_lock: *mut pg_sys::LWLock) {}

    fn test_insert_config(dimensions: usize) -> EcHnswConcurrentDsmInsertConfig {
        EcHnswConcurrentDsmInsertConfig {
            dimensions,
            bits: 4,
            seed: 42,
            m: 2,
            ef_construction: 8,
        }
    }

    fn preassembly_plan_with_levels(
        levels: Vec<u8>,
        codes: Vec<Vec<u8>>,
    ) -> EcHnswConcurrentDsmPreassemblyPlan {
        let levels = build::NativeBuildLevels::from_levels(levels);
        let tuples = codes
            .into_iter()
            .map(build_tuple_with_code)
            .collect::<Vec<_>>();
        let node_layout = EcHnswConcurrentDsmNodeLayoutPlan::for_levels(&levels, 2);
        let code_corpus = EcHnswConcurrentDsmCodeCorpus::from_tuples(&tuples);
        let graph_layout =
            EcHnswConcurrentDsmGraphLayout::for_levels(&levels, 2, code_corpus.code_len as usize);

        EcHnswConcurrentDsmPreassemblyPlan {
            levels,
            node_layout,
            code_corpus,
            graph_layout,
        }
    }

    fn build_state(build_source_column: Option<&str>) -> build::BuildState {
        build::BuildState {
            options: crate::am::ec_hnsw::options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: build_source_column.map(str::to_owned),
                rerank_source_column: None,
                storage_format: crate::quant::Family::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 0,
            heap_tuples: Vec::new(),
            tuple_index_by_payload: HashMap::new(),
            dimensions: None,
            bits: None,
            seed: None,
        }
    }

    fn build_tuple_with_code(code: Vec<u8>) -> build::BuildTuple {
        build::BuildTuple {
            heap_tids: vec![page::ItemPointer {
                block_number: 1,
                offset_number: 1,
            }],
            dimensions: 4,
            bits: 4,
            seed: 42,
            gamma: 1.0,
            code,
            source_vector: None,
            source_count: 0,
        }
    }

    #[test]
    fn build_tuple_message_round_trips_source_payload() {
        let tuple = build::BuildTuple {
            heap_tids: vec![page::ItemPointer {
                block_number: 7,
                offset_number: 3,
            }],
            dimensions: 4,
            bits: 4,
            seed: 42,
            gamma: 1.25,
            code: vec![1, 2, 3, 4],
            source_vector: Some(vec![1.0, -2.0, 0.5, 3.5]),
            source_count: 1,
        };
        let message = encode_build_tuple_message(&tuple);

        let EcHnswParallelBuildWorkerMessage::Tuple(decoded) = decode_worker_message(&message)
        else {
            panic!("expected tuple message");
        };

        assert_eq!(decoded.heap_tids, tuple.heap_tids);
        assert_eq!(decoded.dimensions, tuple.dimensions);
        assert_eq!(decoded.bits, tuple.bits);
        assert_eq!(decoded.seed, tuple.seed);
        assert_eq!(decoded.gamma.to_bits(), tuple.gamma.to_bits());
        assert_eq!(decoded.code, tuple.code);
        assert_eq!(decoded.source_vector, tuple.source_vector);
        assert_eq!(decoded.source_count, tuple.source_count);
    }
}
