use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::{self, NonNull};
use std::slice;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;

use pgrx::pg_sys;

use super::build::{self, BuildTuple};
use crate::am::common::callback::{pg_am_callback, pg_callback};
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
use crate::storage::snapshot_guard::RegisteredSnapshotGuard;

const EC_IVF_PARALLEL_BUILD_MAGIC: u32 = u32::from_le_bytes(*b"ECIP");
const EC_IVF_PARALLEL_BUILD_VERSION: u16 = 1;
const EC_IVF_PARALLEL_BUILD_QUEUE_BYTES: pg_sys::Size = 1024 * 1024;

const PARALLEL_KEY_EC_IVF_BUILD_SHARED: u64 = 0xEC1F_0000_0000_0001;
const PARALLEL_KEY_EC_IVF_WAL_USAGE: u64 = 0xEC1F_0000_0000_0002;
const PARALLEL_KEY_EC_IVF_BUFFER_USAGE: u64 = 0xEC1F_0000_0000_0003;
const PARALLEL_KEY_EC_IVF_QUEUE_BASE: u64 = 0xEC1F_0000_0001_0000;

const EC_IVF_PARALLEL_BUILD_LIBRARY: &[u8] = b"ecaz\0";
const EC_IVF_PARALLEL_BUILD_ENTRYPOINT: &[u8] = b"ec_ivf_parallel_build_main\0";

const BUILD_TUPLE_MESSAGE: u8 = 1;
const BUILD_DONE_MESSAGE: u8 = 2;

static LAST_PARALLEL_BUILD_WORKERS_LAUNCHED: AtomicI32 = AtomicI32::new(0);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum EcIvfBuildCoordinatorKind {
    LeaderLocal,
    DedicatedParallelBuild,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct EcIvfParallelBuildPlan {
    pub(super) requested_workers: i32,
    coordinator: EcIvfBuildCoordinatorKind,
}

impl EcIvfParallelBuildPlan {
    pub(super) fn from_index_info(index_info: *mut pg_sys::IndexInfo) -> Self {
        let requested_workers = if index_info.is_null() {
            0
        } else {
            // SAFETY: PostgreSQL owns `index_info` for the duration of the
            // build callback and initializes `ii_ParallelWorkers`.
            unsafe { (*index_info).ii_ParallelWorkers }
        };
        Self::for_requested_workers(requested_workers)
    }

    #[cfg(test)]
    pub(super) fn for_requested_workers(requested_workers: i32) -> Self {
        Self::for_requested_workers_inner(requested_workers)
    }

    #[cfg(not(test))]
    fn for_requested_workers(requested_workers: i32) -> Self {
        Self::for_requested_workers_inner(requested_workers)
    }

    fn for_requested_workers_inner(requested_workers: i32) -> Self {
        if requested_workers <= 0 {
            return Self {
                requested_workers: 0,
                coordinator: EcIvfBuildCoordinatorKind::LeaderLocal,
            };
        }
        Self {
            requested_workers,
            coordinator: EcIvfBuildCoordinatorKind::DedicatedParallelBuild,
        }
    }

    pub(super) fn uses_serial_build_path(self) -> bool {
        self.coordinator == EcIvfBuildCoordinatorKind::LeaderLocal
    }
}

#[repr(C)]
#[derive(Debug)]
struct EcIvfParallelBuildSharedHeader {
    magic: u32,
    version: u16,
    requested_workers: u16,
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

#[derive(Debug, Copy, Clone)]
struct EcIvfParallelBuildSharedCounts {
    participants_done: i32,
    scanned_heap_tuples: f64,
    encoded_index_tuples: f64,
}

impl EcIvfParallelBuildSharedHeader {
    fn new(
        plan: EcIvfParallelBuildPlan,
        heaprelid: pg_sys::Oid,
        indexrelid: pg_sys::Oid,
        is_concurrent: bool,
    ) -> Self {
        Self {
            magic: EC_IVF_PARALLEL_BUILD_MAGIC,
            version: EC_IVF_PARALLEL_BUILD_VERSION,
            requested_workers: checked_u16(plan.requested_workers, "requested workers"),
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

    fn record_worker_counts(&mut self, heap_tuples: f64, index_tuples: f64) {
        self.nparticipantsdone += 1;
        self.scanned_heap_tuples += heap_tuples;
        self.encoded_index_tuples += index_tuples;
    }

    fn counts(&self) -> EcIvfParallelBuildSharedCounts {
        EcIvfParallelBuildSharedCounts {
            participants_done: self.nparticipantsdone,
            scanned_heap_tuples: self.scanned_heap_tuples,
            encoded_index_tuples: self.encoded_index_tuples,
        }
    }

    fn validate(&self) {
        if self.magic != EC_IVF_PARALLEL_BUILD_MAGIC
            || self.version != EC_IVF_PARALLEL_BUILD_VERSION
        {
            pgrx::error!("ec_ivf parallel build worker saw incompatible shared state");
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct EcIvfParallelBuildResult {
    pub(super) heap_tuples: f64,
    pub(super) begin_us: u64,
    pub(super) drain_us: u64,
    pub(super) sort_push_us: u64,
    pub(super) worker_tuple_buffer_capacity: u64,
    pub(super) worker_tuple_buffer_struct_bytes: u64,
}

pub(super) unsafe fn try_parallel_build(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    state: &mut build::BuildState,
    plan: EcIvfParallelBuildPlan,
) -> Option<EcIvfParallelBuildResult> {
    if plan.uses_serial_build_path() {
        return None;
    }

    let begin_start = Instant::now();
    // SAFETY: The AM build callback owns live heap/index/IndexInfo handles.
    let mut leader = unsafe {
        EcIvfParallelBuildLeader::begin(heap_relation, index_relation, index_info, plan)
    }?;
    let begin_us = elapsed_us(begin_start);

    let drain_start = Instant::now();
    let mut worker_tuples = Vec::new();
    // SAFETY: `leader` owns the live worker queues until `finish`.
    unsafe { leader.drain_worker_messages(&mut worker_tuples) };
    let shared_counts = leader.finish();
    let worker_tuple_buffer_capacity = usize_to_u64(worker_tuples.capacity());
    let worker_tuple_buffer_struct_bytes = usize_to_u64(
        worker_tuples
            .capacity()
            .saturating_mul(size_of::<BuildTuple>()),
    );
    let drain_us = elapsed_us(drain_start);

    let sort_push_start = Instant::now();
    worker_tuples.sort_by_key(build_tuple_heap_tid_key);
    for tuple in worker_tuples {
        state.push(tuple);
    }
    validate_parallel_build_counts(plan, shared_counts, state.scanned_tuples);
    let sort_push_us = elapsed_us(sort_push_start);

    Some(EcIvfParallelBuildResult {
        heap_tuples: state.scanned_tuples as f64,
        begin_us,
        drain_us,
        sort_push_us,
        worker_tuple_buffer_capacity,
        worker_tuple_buffer_struct_bytes,
    })
}

struct EcIvfParallelBuildLeader {
    pcxt: *mut pg_sys::ParallelContext,
    shared: *mut EcIvfParallelBuildSharedHeader,
    snapshot_guard: Option<RegisteredSnapshotGuard>,
    queue_handles: Vec<*mut pg_sys::shm_mq_handle>,
    walusage: *mut pg_sys::WalUsage,
    bufferusage: *mut pg_sys::BufferUsage,
}

impl EcIvfParallelBuildLeader {
    unsafe fn begin(
        heap_relation: pg_sys::Relation,
        index_relation: pg_sys::Relation,
        index_info: *mut pg_sys::IndexInfo,
        plan: EcIvfParallelBuildPlan,
    ) -> Option<Self> {
        debug_assert!(plan.requested_workers > 0);
        // SAFETY: Paired with `ExitParallelMode` on every path below.
        unsafe { pg_sys::EnterParallelMode() };

        // SAFETY: Static library/entrypoint names and positive worker count.
        let pcxt = unsafe {
            pg_sys::CreateParallelContext(
                EC_IVF_PARALLEL_BUILD_LIBRARY.as_ptr().cast(),
                EC_IVF_PARALLEL_BUILD_ENTRYPOINT.as_ptr().cast(),
                plan.requested_workers,
            )
        };
        if pcxt.is_null() {
            // SAFETY: Cleanup for successful `EnterParallelMode`.
            unsafe { pg_sys::ExitParallelMode() };
            return None;
        }

        // SAFETY: Non-null `index_info` belongs to PostgreSQL build setup.
        let is_concurrent = unsafe { !index_info.is_null() && (*index_info).ii_Concurrent };
        let snapshot_guard = if is_concurrent {
            Some(RegisteredSnapshotGuard::transaction().unwrap_or_else(|| {
                pgrx::error!("ec_ivf parallel build failed to register transaction snapshot")
            }))
        } else {
            None
        };
        let snapshot = snapshot_guard
            .as_ref()
            .map_or(ptr::addr_of_mut!(pg_sys::SnapshotAnyData), |guard| {
                guard.as_ptr()
            });

        let shared_bytes = parallel_build_shared_workspace_size(heap_relation, snapshot);
        // SAFETY: Estimator belongs to a live ParallelContext before DSM init.
        unsafe {
            estimate_chunk(&mut (*pcxt).estimator, shared_bytes);
            estimate_keys(&mut (*pcxt).estimator, 1);
            for _ in 0..plan.requested_workers {
                estimate_chunk(&mut (*pcxt).estimator, EC_IVF_PARALLEL_BUILD_QUEUE_BYTES);
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

        // SAFETY: `pcxt` is live and fully estimated; DSM allocations are sized.
        let shared = unsafe {
            pg_sys::InitializeParallelDSM(pcxt);
            if (*pcxt).seg.is_null() {
                pg_sys::DestroyParallelContext(pcxt);
                pg_sys::ExitParallelMode();
                return None;
            }
            let shared = pg_sys::shm_toc_allocate((*pcxt).toc, shared_bytes)
                .cast::<EcIvfParallelBuildSharedHeader>();
            ptr::write(
                shared,
                EcIvfParallelBuildSharedHeader::new(
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
            pg_sys::shm_toc_insert((*pcxt).toc, PARALLEL_KEY_EC_IVF_BUILD_SHARED, shared.cast());
            shared
        };

        // SAFETY: Queue chunks were estimated and inserted before launch.
        unsafe {
            for worker_index in 0..plan.requested_workers {
                let mq = pg_sys::shm_mq_create(
                    pg_sys::shm_toc_allocate((*pcxt).toc, EC_IVF_PARALLEL_BUILD_QUEUE_BYTES),
                    EC_IVF_PARALLEL_BUILD_QUEUE_BYTES,
                );
                pg_sys::shm_mq_set_receiver(mq, pg_sys::MyProc);
                pg_sys::shm_toc_insert((*pcxt).toc, queue_key(worker_index), mq.cast::<c_void>());
            }
        }

        // SAFETY: Accounting arrays are sized for requested workers and
        // inserted before workers launch.
        let (walusage, bufferusage, workers_launched) = unsafe {
            let walusage = pg_sys::shm_toc_allocate(
                (*pcxt).toc,
                checked_mul_size(
                    size_of::<pg_sys::WalUsage>() as pg_sys::Size,
                    plan.requested_workers as pg_sys::Size,
                    "parallel build WAL usage allocation",
                ),
            )
            .cast::<pg_sys::WalUsage>();
            let bufferusage = pg_sys::shm_toc_allocate(
                (*pcxt).toc,
                checked_mul_size(
                    size_of::<pg_sys::BufferUsage>() as pg_sys::Size,
                    plan.requested_workers as pg_sys::Size,
                    "parallel build buffer usage allocation",
                ),
            )
            .cast::<pg_sys::BufferUsage>();
            pg_sys::shm_toc_insert((*pcxt).toc, PARALLEL_KEY_EC_IVF_WAL_USAGE, walusage.cast());
            pg_sys::shm_toc_insert(
                (*pcxt).toc,
                PARALLEL_KEY_EC_IVF_BUFFER_USAGE,
                bufferusage.cast(),
            );
            pg_sys::LaunchParallelWorkers(pcxt);
            let workers_launched = (*pcxt).nworkers_launched;
            (walusage, bufferusage, workers_launched)
        };
        record_debug_parallel_build_workers_launched(workers_launched);

        let mut leader = Self {
            pcxt,
            shared,
            snapshot_guard,
            queue_handles: Vec::with_capacity(workers_launched.max(0) as usize),
            walusage,
            bufferusage,
        };

        if workers_launched == 0 {
            leader.finish();
            return None;
        }

        // SAFETY: Worker queues were inserted before launch and `pcxt` is live.
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

    unsafe fn drain_worker_messages(&mut self, tuples: &mut Vec<BuildTuple>) {
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
                    // SAFETY: Queue handle is attached to this leader's worker queue.
                    let result = unsafe {
                        pg_sys::shm_mq_receive(queue_handle, &mut nbytes, &mut data, true)
                    };

                    match result {
                        pg_sys::shm_mq_result::SHM_MQ_SUCCESS => {
                            made_progress = true;
                            if data.is_null() || nbytes == 0 {
                                pgrx::error!("ec_ivf parallel build worker sent an empty message");
                            }
                            // SAFETY: PG returned `nbytes` bytes at a valid message pointer.
                            let bytes = unsafe { slice::from_raw_parts(data.cast::<u8>(), nbytes) };
                            match decode_worker_message(bytes) {
                                EcIvfParallelBuildWorkerMessage::Tuple(tuple) => tuples.push(tuple),
                                EcIvfParallelBuildWorkerMessage::Done => {
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
                        _ => pgrx::error!("ec_ivf parallel build saw unknown shm_mq result"),
                    }
                }
            }

            if done_count < self.queue_handles.len() && !made_progress {
                // SAFETY: Backend-safe while waiting in parallel mode.
                unsafe {
                    pg_sys::ProcessInterrupts();
                    pg_sys::pg_usleep(1000);
                }
            }
        }
    }

    fn finish(mut self) -> EcIvfParallelBuildSharedCounts {
        // SAFETY: `pcxt` and accounting arrays are live until destroyed below.
        let shared_counts = unsafe {
            pg_sys::WaitForParallelWorkersToFinish(self.pcxt);
            let launched = (*self.pcxt).nworkers_launched.max(0) as usize;
            for worker_index in 0..launched {
                pg_sys::InstrAccumParallelQuery(
                    self.bufferusage.add(worker_index),
                    self.walusage.add(worker_index),
                );
            }
            (*self.shared).counts()
        };

        drop(self.snapshot_guard.take());

        // SAFETY: Paired cleanup for `CreateParallelContext` and `EnterParallelMode`.
        unsafe {
            pg_sys::DestroyParallelContext(self.pcxt);
            pg_sys::ExitParallelMode();
        }
        shared_counts
    }
}

fn validate_parallel_build_counts(
    plan: EcIvfParallelBuildPlan,
    shared_counts: EcIvfParallelBuildSharedCounts,
    scanned_tuples: usize,
) {
    let workers_launched = debug_last_parallel_build_workers_launched();
    if shared_counts.participants_done != workers_launched {
        pgrx::error!(
            "ec_ivf parallel build launched {workers_launched} workers but recorded {} finished participants",
            shared_counts.participants_done
        );
    }
    if workers_launched > plan.requested_workers {
        pgrx::error!(
            "ec_ivf parallel build launched {workers_launched} workers but only requested {}",
            plan.requested_workers
        );
    }
    let scanned = scanned_tuples as f64;
    if shared_counts.scanned_heap_tuples != scanned {
        pgrx::error!(
            "ec_ivf parallel build workers scanned {} heap tuples but leader observed {scanned}",
            shared_counts.scanned_heap_tuples
        );
    }
    if shared_counts.encoded_index_tuples != scanned {
        pgrx::error!(
            "ec_ivf parallel build workers encoded {} index tuples but leader observed {scanned}",
            shared_counts.encoded_index_tuples
        );
    }
}

struct EcIvfParallelBuildWorkerScanState {
    queue_handle: *mut pg_sys::shm_mq_handle,
    indexed_vector_kind: build::IndexedVectorKind,
    storage_format: super::options::StorageFormat,
    quant_bits: u8,
    encoded_tuples: u64,
}

unsafe extern "C-unwind" fn ec_ivf_parallel_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    pg_am_callback!({
        let state = &mut *state.cast::<EcIvfParallelBuildWorkerScanState>();
        let heap_tid = build::decode_heap_tid(tid, "parallel ambuild");
        let tuple = build::build_index_tuple(
            values,
            isnull,
            heap_tid,
            state.indexed_vector_kind,
            state.storage_format,
            state.quant_bits,
            "parallel ambuild",
        );
        send_build_tuple_message(state.queue_handle, &tuple);
        state.encoded_tuples += 1;
    })
}

#[pgrx::pg_guard]
#[no_mangle]
/// # Safety
///
/// PostgreSQL invokes this dynamic background-worker entrypoint with a live DSM
/// segment and TOC created by the leader's `ParallelContext`. Both pointers
/// must remain valid for the duration of the worker callback.
pub unsafe extern "C-unwind" fn ec_ivf_parallel_build_main(
    seg: *mut pg_sys::dsm_segment,
    toc: *mut pg_sys::shm_toc,
) {
    pg_callback!({
        parallel_build_worker_main(seg, toc);
    })
}

/// # Safety
///
/// `seg` and `toc` must refer to the live DSM and TOC populated by
/// `EcIvfParallelBuildLeader::begin`. The expected shared-state keys and
/// per-worker queue key for `ParallelWorkerNumber` must be present.
unsafe fn parallel_build_worker_main(seg: *mut pg_sys::dsm_segment, toc: *mut pg_sys::shm_toc) {
    let shared: *mut EcIvfParallelBuildSharedHeader =
        shm_toc_lookup_required(toc, PARALLEL_KEY_EC_IVF_BUILD_SHARED);
    // SAFETY: The leader inserted the shared header before launching workers.
    let header: &EcIvfParallelBuildSharedHeader = unsafe {
        NonNull::new(shared)
            .unwrap_or_else(|| pgrx::error!("ec_ivf parallel build shared header is null"))
            .as_ref()
    };
    header.validate();

    // SAFETY: PostgreSQL assigns this before invoking the worker entrypoint.
    let worker_number = unsafe { pg_sys::ParallelWorkerNumber };
    if worker_number < 0 {
        pgrx::error!("ec_ivf parallel build worker started without a worker number");
    }

    let queue: *mut pg_sys::shm_mq = shm_toc_lookup_required(toc, queue_key(worker_number));
    // SAFETY: This worker is the sender for its queue in the inherited DSM.
    let queue_handle = unsafe {
        pg_sys::shm_mq_set_sender(queue, pg_sys::MyProc);
        pg_sys::shm_mq_attach(queue, seg, ptr::null_mut())
    };

    let is_concurrent = header.is_concurrent;
    let (heap_lockmode, index_lockmode) = if is_concurrent {
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

    let heap_relation_guard = HeapRelationGuard::try_open(header.heaprelid, heap_lockmode)
        .unwrap_or_else(|| pgrx::error!("ec_ivf parallel build worker could not open heap"));
    let index_relation_guard = IndexRelationGuard::try_open(header.indexrelid, index_lockmode)
        .unwrap_or_else(|| pgrx::error!("ec_ivf parallel build worker could not open index"));
    let heap_relation = heap_relation_guard.as_ptr();
    let index_relation = index_relation_guard.as_ptr();

    // SAFETY: Worker is running inside PostgreSQL parallel-query instrumentation.
    unsafe { pg_sys::InstrStartParallelQuery() };

    let options = super::options::relation_options(
        NonNull::new(index_relation)
            .unwrap_or_else(|| pgrx::error!("ec_ivf parallel build worker opened null index")),
    );
    let mut index_info = IndexInfoView::build_borrowed(index_relation, "parallel build worker");
    index_info.set_concurrent(is_concurrent);
    let indexed_vector_kind = build::resolve_indexed_vector_kind(
        heap_relation,
        index_info.as_ptr(),
        "parallel build worker",
    );
    let mut worker_state = EcIvfParallelBuildWorkerScanState {
        queue_handle,
        indexed_vector_kind,
        storage_format: options.storage_format,
        quant_bits: options.effective_quant_bits(),
        encoded_tuples: 0,
    };

    // SAFETY: Shared parallel table scan state and relation handles are live.
    let scanned_tuples = unsafe {
        let scan = pg_sys::table_beginscan_parallel(
            heap_relation,
            parallel_table_scan_from_shared(shared),
        );
        pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            index_info.as_ptr(),
            true,
            false,
            Some(ec_ivf_parallel_build_callback),
            (&mut worker_state as *mut EcIvfParallelBuildWorkerScanState).cast(),
            scan,
        )
    };

    send_done_message(queue_handle);

    // SAFETY: Spinlock protects aggregate worker counters.
    unsafe {
        pg_sys::SpinLockAcquire(&mut (*shared).mutex);
        (*shared).record_worker_counts(scanned_tuples, worker_state.encoded_tuples as f64);
        pg_sys::SpinLockRelease(&mut (*shared).mutex);
        pg_sys::ConditionVariableSignal(&mut (*shared).workersdonecv);
    }

    let bufferusage: *mut pg_sys::BufferUsage =
        shm_toc_lookup_required(toc, PARALLEL_KEY_EC_IVF_BUFFER_USAGE);
    let walusage: *mut pg_sys::WalUsage =
        shm_toc_lookup_required(toc, PARALLEL_KEY_EC_IVF_WAL_USAGE);
    // SAFETY: Worker number indexes the accounting arrays allocated by leader.
    unsafe {
        pg_sys::InstrEndParallelQuery(
            bufferusage.add(worker_number as usize),
            walusage.add(worker_number as usize),
        );
    }
    drop(index_relation_guard);
    drop(heap_relation_guard);
}

enum EcIvfParallelBuildWorkerMessage {
    Tuple(BuildTuple),
    Done,
}

fn send_build_tuple_message(queue_handle: *mut pg_sys::shm_mq_handle, tuple: &BuildTuple) {
    let message = encode_build_tuple_message(tuple);
    send_worker_message(queue_handle, &message);
}

fn send_done_message(queue_handle: *mut pg_sys::shm_mq_handle) {
    let message = [BUILD_DONE_MESSAGE];
    send_worker_message(queue_handle, &message);
}

fn send_worker_message(queue_handle: *mut pg_sys::shm_mq_handle, message: &[u8]) {
    // SAFETY: The queue handle is attached and message bytes are valid.
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
            pgrx::error!("ec_ivf parallel build worker queue detached")
        }
        _ => pgrx::error!("ec_ivf parallel build worker could not send a tuple message"),
    }
}

fn encode_build_tuple_message(tuple: &BuildTuple) -> Vec<u8> {
    let payload_len = checked_u32(tuple.payload.len(), "parallel build payload length");
    let source_len = checked_u32(
        tuple.source_vector.len(),
        "parallel build source vector length",
    );
    let mut message = Vec::with_capacity(
        1 + 4 + 2 + 2 + 4 + 4 + 4 + tuple.payload.len() + tuple.source_vector.len() * 4,
    );
    message.push(BUILD_TUPLE_MESSAGE);
    message.extend_from_slice(&tuple.heap_tid.block_number.to_le_bytes());
    message.extend_from_slice(&tuple.heap_tid.offset_number.to_le_bytes());
    message.extend_from_slice(&tuple.dimensions.to_le_bytes());
    message.extend_from_slice(&tuple.gamma.to_bits().to_le_bytes());
    message.extend_from_slice(&payload_len.to_le_bytes());
    message.extend_from_slice(&source_len.to_le_bytes());
    message.extend_from_slice(&tuple.payload);
    for value in &tuple.source_vector {
        message.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    message
}

fn decode_worker_message(bytes: &[u8]) -> EcIvfParallelBuildWorkerMessage {
    let mut cursor = 0_usize;
    let kind = read_u8(bytes, &mut cursor);
    match kind {
        BUILD_DONE_MESSAGE => {
            if cursor != bytes.len() {
                pgrx::error!("ec_ivf parallel build done message had trailing bytes");
            }
            EcIvfParallelBuildWorkerMessage::Done
        }
        BUILD_TUPLE_MESSAGE => {
            let block_number = read_u32(bytes, &mut cursor);
            let offset_number = read_u16(bytes, &mut cursor);
            let dimensions = read_u16(bytes, &mut cursor);
            let gamma = f32::from_bits(read_u32(bytes, &mut cursor));
            let payload_len = read_u32(bytes, &mut cursor) as usize;
            let source_len = read_u32(bytes, &mut cursor) as usize;
            let payload = read_bytes(bytes, &mut cursor, payload_len).to_vec();
            let mut source_vector = Vec::with_capacity(source_len);
            for _ in 0..source_len {
                source_vector.push(f32::from_bits(read_u32(bytes, &mut cursor)));
            }
            if cursor != bytes.len() {
                pgrx::error!("ec_ivf parallel build tuple message had trailing bytes");
            }
            EcIvfParallelBuildWorkerMessage::Tuple(BuildTuple {
                heap_tid: crate::storage::page::ItemPointer {
                    block_number,
                    offset_number,
                },
                dimensions,
                gamma,
                payload,
                source_vector,
            })
        }
        _ => pgrx::error!("ec_ivf parallel build worker sent an unknown message kind"),
    }
}

fn build_tuple_heap_tid_key(tuple: &BuildTuple) -> (u32, u16) {
    (tuple.heap_tid.block_number, tuple.heap_tid.offset_number)
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

fn read_bytes<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> &'a [u8] {
    let end = cursor
        .checked_add(len)
        .unwrap_or_else(|| pgrx::error!("ec_ivf parallel build message cursor overflow"));
    if end > bytes.len() {
        pgrx::error!("ec_ivf parallel build worker sent a truncated message");
    }
    let out = &bytes[*cursor..end];
    *cursor = end;
    out
}

fn checked_u16(value: i32, field: &str) -> u16 {
    u16::try_from(value).unwrap_or_else(|_| panic!("parallel build {field} should fit in u16"))
}

fn checked_u32(value: usize, field: &str) -> u32 {
    u32::try_from(value).unwrap_or_else(|_| pgrx::error!("{field} does not fit in u32"))
}

fn shm_toc_lookup_required<T>(toc: *mut pg_sys::shm_toc, key: u64) -> *mut T {
    // SAFETY: Leader inserted this TOC key before worker launch.
    unsafe { pg_sys::shm_toc_lookup(toc, key, false) }.cast::<T>()
}

pub(super) fn reset_debug_last_parallel_build_workers_launched() {
    LAST_PARALLEL_BUILD_WORKERS_LAUNCHED.store(0, Ordering::Release);
}

fn record_debug_parallel_build_workers_launched(workers_launched: i32) {
    LAST_PARALLEL_BUILD_WORKERS_LAUNCHED.store(workers_launched, Ordering::Release);
}

pub(crate) fn debug_last_parallel_build_workers_launched() -> i32 {
    LAST_PARALLEL_BUILD_WORKERS_LAUNCHED.load(Ordering::Acquire)
}

fn parallel_build_shared_workspace_size(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
) -> pg_sys::Size {
    checked_add_size(
        bufferalign(size_of::<EcIvfParallelBuildSharedHeader>() as pg_sys::Size),
        // SAFETY: Inputs are live PG handles and the estimate does not retain them.
        unsafe { pg_sys::table_parallelscan_estimate(heap_relation, snapshot) },
        "parallel build shared workspace size",
    )
}

fn parallel_table_scan_from_shared(
    shared: *mut EcIvfParallelBuildSharedHeader,
) -> pg_sys::ParallelTableScanDesc {
    // SAFETY: The parallel table scan descriptor starts after the aligned header.
    unsafe {
        shared
            .cast::<u8>()
            .add(bufferalign(
                size_of::<EcIvfParallelBuildSharedHeader>() as pg_sys::Size
            ))
            .cast()
    }
}

/// # Safety
///
/// `estimator` must be a live `shm_toc_estimator` owned by a
/// `ParallelContext` before DSM initialization.
unsafe fn estimate_chunk(estimator: *mut pg_sys::shm_toc_estimator, size: pg_sys::Size) {
    // SAFETY: Caller passes a live estimator before DSM initialization.
    unsafe {
        (*estimator).space_for_chunks = checked_add_size(
            (*estimator).space_for_chunks,
            bufferalign(size),
            "parallel build DSM chunk estimate",
        );
    }
}

/// # Safety
///
/// `estimator` must be a live `shm_toc_estimator` owned by a
/// `ParallelContext` before DSM initialization.
unsafe fn estimate_keys(estimator: *mut pg_sys::shm_toc_estimator, keys: pg_sys::Size) {
    // SAFETY: Caller passes a live estimator before DSM initialization.
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
        pgrx::error!("ec_ivf parallel build worker index was negative");
    }
    PARALLEL_KEY_EC_IVF_QUEUE_BASE + worker_index as u64
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

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn build_index_info_inner(
    index_relation: pg_sys::Relation,
    label: &str,
) -> NonNull<pg_sys::IndexInfo> {
    let index_relation = NonNull::new(index_relation)
        .unwrap_or_else(|| pgrx::error!("ec_ivf {label} needs a valid index relation"));
    // SAFETY: Index relation is live and PG owns allocation in current context.
    let ptr = unsafe { pg_sys::BuildIndexInfo(index_relation.as_ptr()) };
    NonNull::new(ptr)
        .unwrap_or_else(|| pgrx::error!("ec_ivf {label} could not build index metadata"))
}

struct IndexInfoView<'scope> {
    ptr: NonNull<pg_sys::IndexInfo>,
    _scope: PhantomData<&'scope mut pg_sys::IndexInfo>,
}

impl<'scope> IndexInfoView<'scope> {
    fn build_borrowed(index_relation: pg_sys::Relation, label: &str) -> Self {
        Self {
            ptr: build_index_info_inner(index_relation, label),
            _scope: PhantomData,
        }
    }

    fn as_ptr(&self) -> *mut pg_sys::IndexInfo {
        self.ptr.as_ptr()
    }

    fn set_concurrent(&mut self, is_concurrent: bool) {
        // SAFETY: `&mut self` enforces exclusive access for this bounded view.
        unsafe {
            self.ptr.as_mut().ii_Concurrent = is_concurrent;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuple() -> BuildTuple {
        BuildTuple {
            heap_tid: crate::storage::page::ItemPointer {
                block_number: 7,
                offset_number: 3,
            },
            dimensions: 4,
            gamma: 1.25,
            payload: vec![1, 2, 3, 4],
            source_vector: vec![0.25, -1.0, f32::INFINITY, f32::NAN],
        }
    }

    #[test]
    fn parallel_build_plan_stays_serial_without_requested_workers() {
        let plan = EcIvfParallelBuildPlan::for_requested_workers(0);
        assert!(plan.uses_serial_build_path());
        assert_eq!(plan.requested_workers, 0);
    }

    #[test]
    fn parallel_build_plan_uses_dedicated_build_coordinator() {
        let plan = EcIvfParallelBuildPlan::for_requested_workers(3);
        assert!(!plan.uses_serial_build_path());
        assert_eq!(plan.requested_workers, 3);
    }

    #[test]
    fn build_tuple_message_round_trips_payload_and_source_bits() {
        let tuple = tuple();
        let message = encode_build_tuple_message(&tuple);
        match decode_worker_message(&message) {
            EcIvfParallelBuildWorkerMessage::Tuple(decoded) => {
                assert_eq!(decoded.heap_tid, tuple.heap_tid);
                assert_eq!(decoded.dimensions, tuple.dimensions);
                assert_eq!(decoded.gamma.to_bits(), tuple.gamma.to_bits());
                assert_eq!(decoded.payload, tuple.payload);
                assert_eq!(decoded.source_vector.len(), tuple.source_vector.len());
                for (actual, expected) in decoded.source_vector.iter().zip(&tuple.source_vector) {
                    assert_eq!(actual.to_bits(), expected.to_bits());
                }
            }
            EcIvfParallelBuildWorkerMessage::Done => panic!("expected tuple message"),
        }
    }

    #[test]
    fn done_message_round_trips() {
        match decode_worker_message(&[BUILD_DONE_MESSAGE]) {
            EcIvfParallelBuildWorkerMessage::Done => {}
            EcIvfParallelBuildWorkerMessage::Tuple(_) => panic!("expected done message"),
        }
    }
}
