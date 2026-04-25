use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;

use pgrx::pg_sys;

use super::{build, page, shared, source};

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
        let code_len = tuples
            .first()
            .map_or(0_usize, |tuple| tuple.code.len());
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
    let mut leader =
        unsafe { EcHnswParallelBuildLeader::begin(heap_relation, index_relation, index_info, plan) }?;
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
                pg_sys::shm_toc_insert(
                    (*pcxt).toc,
                    queue_key(worker_index),
                    mq.cast::<c_void>(),
                );
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
                    let result =
                        unsafe { pg_sys::shm_mq_receive(queue_handle, &mut nbytes, &mut data, true) };

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
            let tuple = build::build_heap_tuple(values, isnull, heap_tid, state.indexed_vector_kind);
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

unsafe fn parallel_build_worker_main(
    seg: *mut pg_sys::dsm_segment,
    toc: *mut pg_sys::shm_toc,
) {
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

    let mut message =
        Vec::with_capacity(1 + 1 + 4 + 2 + 2 + 8 + 4 + 4 + 4 + 4 + tuple.code.len() + source_len * 4);
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
                size_of::<EcHnswParallelBuildSharedHeader>() as pg_sys::Size,
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
