//! FR-082 coordinator-local exact scan-token registry foundation.
//!
//! The registry lives in add-in shared memory and is intentionally independent
//! of CustomScan, participant transport, publish decisions, and retirement.
//! This module owns only exact token identity/liveness and bounded logical-index
//! fence allocation.  Later slices acquire the exported fence operation
//! reference before waiting for or holding the matching heavyweight fence.

use std::cell::{Cell, RefCell};
use std::ffi::c_int;
use std::mem::{align_of, size_of};
use std::panic::AssertUnwindSafe;
use std::ptr;

use pgrx::datum::Uuid;
use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting};

const REGISTRY_MAGIC: u64 = 0x4543_4453_5250_3031; // "ECDSRP01"
const REGISTRY_VERSION: u32 = 1;
const DEFAULT_MAX_SCAN_PINS: i32 = 65_536;
const DEFAULT_MAX_RETIRE_FENCES: i32 = 4_096;
const MAX_CONFIGURED_SCAN_PINS: i32 = 1_048_576;
const MAX_CONFIGURED_RETIRE_FENCES: i32 = 65_536;
const OPERATION_SLOTS_PER_BACKEND: usize = 4;
const HASH_EMPTY: i32 = 0;
const HASH_TOMBSTONE: i32 = -1;
static MAX_SCAN_PINS_GUC: GucSetting<i32> = GucSetting::<i32>::new(DEFAULT_MAX_SCAN_PINS);
static MAX_RETIRE_FENCES_GUC: GucSetting<i32> = GucSetting::<i32>::new(DEFAULT_MAX_RETIRE_FENCES);

static mut PREV_SHMEM_REQUEST_HOOK: pg_sys::shmem_request_hook_type = None;
static mut PREV_SHMEM_STARTUP_HOOK: pg_sys::shmem_startup_hook_type = None;
static mut REGISTRY_HEADER: *mut RegistryHeader = ptr::null_mut();
static mut REGISTRY_LWLOCK: *mut pg_sys::LWLock = ptr::null_mut();

thread_local! {
    static BACKEND_OWNER: Cell<Option<BackendOwner>> = const { Cell::new(None) };
    static EXIT_CALLBACK_REGISTERED: Cell<bool> = const { Cell::new(false) };
    static XACT_FENCE_REFERENCES: RefCell<Vec<XactFenceReference>> = const { RefCell::new(Vec::new()) };
    static NEXT_XACT_FENCE_ID: Cell<u64> = const { Cell::new(1) };
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RegistryHeader {
    magic: u64,
    version: u32,
    total_bytes: usize,
    generation_capacity: u32,
    operation_capacity: u32,
    fence_capacity: u32,
    token_capacity: u32,
    operation_hash_capacity: u32,
    fence_hash_capacity: u32,
    token_hash_capacity: u32,
    operation_free_head: i32,
    fence_free_head: i32,
    token_free_head: i32,
    generation_offset: usize,
    operation_offset: usize,
    operation_hash_offset: usize,
    fence_offset: usize,
    fence_hash_offset: usize,
    token_offset: usize,
    token_hash_offset: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct BackendGeneration {
    generation: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct FenceOperationSlot {
    in_use: u8,
    _padding: [u8; 3],
    fence_id: u32,
    proc_number: pg_sys::ProcNumber,
    backend_pid: c_int,
    backend_generation: u64,
    nesting: u32,
    next_free: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct FenceSlot {
    in_use: u8,
    dropped: u8,
    _padding: [u8; 2],
    database_oid: pg_sys::Oid,
    logical_index_uuid: [u8; 16],
    fence_id: u32,
    operation_references: u32,
    token_count: u32,
    next_free: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenSlot {
    in_use: u8,
    _padding: [u8; 3],
    database_oid: pg_sys::Oid,
    logical_index_uuid: [u8; 16],
    epoch_fingerprint: [u8; 34],
    scan_token: [u8; 16],
    proc_number: pg_sys::ProcNumber,
    backend_pid: c_int,
    backend_generation: u64,
    next_free: i32,
    _tail_padding: [u8; 4],
}

impl Default for TokenSlot {
    fn default() -> Self {
        Self {
            in_use: 0,
            _padding: [0; 3],
            database_oid: pg_sys::InvalidOid,
            logical_index_uuid: [0; 16],
            epoch_fingerprint: [0; 34],
            scan_token: [0; 16],
            proc_number: -1,
            backend_pid: 0,
            backend_generation: 0,
            next_free: -1,
            _tail_padding: [0; 4],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct HashEntry {
    /// Zero is empty, -1 is a deleted bucket, positive values are slot+1.
    slot_plus_one: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScanTokenIdentity {
    pub(crate) database_oid: pg_sys::Oid,
    pub(crate) logical_index_uuid: [u8; 16],
    pub(crate) epoch_fingerprint: [u8; 34],
    pub(crate) scan_token: [u8; 16],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BackendOwner {
    proc_number: pg_sys::ProcNumber,
    pid: c_int,
    generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegisterOutcome {
    Registered,
    AlreadyRegistered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistryError {
    Unavailable,
    TokenCapacity,
    FenceCapacity,
    TokenConflict,
    LogicalIndexDropped,
    CorruptSharedState,
}

impl RegistryError {
    pub(crate) fn stable_message(self) -> &'static str {
        match self {
            Self::Unavailable => {
                "EC_EPOCH_REGISTRY_UNAVAILABLE: ecaz must be in shared_preload_libraries"
            }
            Self::TokenCapacity | Self::FenceCapacity => {
                "EC_EPOCH_PIN_CAPACITY: exact scan-token or fence capacity is exhausted"
            }
            Self::TokenConflict => {
                "EC_EPOCH_PIN_CONFLICT: scan token is already bound to another fingerprint"
            }
            Self::LogicalIndexDropped => {
                "EC_GENERATION_MISSING: logical-index scan fence has been dropped"
            }
            Self::CorruptSharedState => {
                "EC_EPOCH_REGISTRY_UNAVAILABLE: shared scan registry state is corrupt"
            }
        }
    }
}

pub(crate) fn register_gucs() {
    GucRegistry::define_int_guc(
        c"ec_distann.max_scan_pins",
        c"Maximum exact coordinator scan-token registrations.",
        c"Postmaster-start capacity for the FR-082 shared scan-token registry. Zero disables allocation and makes distributed registration fail closed.",
        &MAX_SCAN_PINS_GUC,
        0,
        MAX_CONFIGURED_SCAN_PINS,
        GucContext::Postmaster,
        GucFlags::default(),
    );
    GucRegistry::define_int_guc(
        c"ec_distann.max_retire_fences",
        c"Maximum coordinator logical-index retirement fences.",
        c"Postmaster-start capacity for collision-free FR-082 logical-index fence identities. Zero disables allocation.",
        &MAX_RETIRE_FENCES_GUC,
        0,
        MAX_CONFIGURED_RETIRE_FENCES,
        GucContext::Postmaster,
        GucFlags::default(),
    );
}

#[derive(Debug, Clone, Copy)]
struct RegistryLayout {
    total_bytes: usize,
    generation_capacity: usize,
    operation_capacity: usize,
    fence_capacity: usize,
    token_capacity: usize,
    operation_hash_capacity: usize,
    fence_hash_capacity: usize,
    token_hash_capacity: usize,
    generation_offset: usize,
    operation_offset: usize,
    operation_hash_offset: usize,
    fence_offset: usize,
    fence_hash_offset: usize,
    token_offset: usize,
    token_hash_offset: usize,
}

fn align_up(value: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
    (value + alignment - 1) & !(alignment - 1)
}

fn checked_region(
    offset: usize,
    count: usize,
    element_size: usize,
    alignment: usize,
) -> Option<(usize, usize)> {
    let aligned = align_up(offset, alignment);
    let bytes = count.checked_mul(element_size)?;
    Some((aligned, aligned.checked_add(bytes)?))
}

fn configured_layout() -> Option<RegistryLayout> {
    let generation_capacity = unsafe { pg_sys::MaxBackends.max(0) as usize };
    let fence_capacity = usize::try_from(MAX_RETIRE_FENCES_GUC.get()).ok()?;
    let token_capacity = usize::try_from(MAX_SCAN_PINS_GUC.get()).ok()?;
    RegistryLayout::new(generation_capacity, fence_capacity, token_capacity)
}

impl RegistryLayout {
    fn new(
        generation_capacity: usize,
        fence_capacity: usize,
        token_capacity: usize,
    ) -> Option<Self> {
        let (generation_offset, after_generations) = checked_region(
            size_of::<RegistryHeader>(),
            generation_capacity,
            size_of::<BackendGeneration>(),
            align_of::<BackendGeneration>(),
        )?;
        let operation_capacity = generation_capacity.checked_mul(OPERATION_SLOTS_PER_BACKEND)?;
        let operation_hash_capacity = hash_capacity(operation_capacity)?;
        let fence_hash_capacity = hash_capacity(fence_capacity)?;
        let token_hash_capacity = hash_capacity(token_capacity)?;
        let (operation_offset, after_operations) = checked_region(
            after_generations,
            operation_capacity,
            size_of::<FenceOperationSlot>(),
            align_of::<FenceOperationSlot>(),
        )?;
        let (operation_hash_offset, after_operation_hash) = checked_region(
            after_operations,
            operation_hash_capacity,
            size_of::<HashEntry>(),
            align_of::<HashEntry>(),
        )?;
        let (fence_offset, after_fences) = checked_region(
            after_operation_hash,
            fence_capacity,
            size_of::<FenceSlot>(),
            align_of::<FenceSlot>(),
        )?;
        let (fence_hash_offset, after_fence_hash) = checked_region(
            after_fences,
            fence_hash_capacity,
            size_of::<HashEntry>(),
            align_of::<HashEntry>(),
        )?;
        let (token_offset, total_bytes) = checked_region(
            after_fence_hash,
            token_capacity,
            size_of::<TokenSlot>(),
            align_of::<TokenSlot>(),
        )?;
        let (token_hash_offset, total_bytes) = checked_region(
            total_bytes,
            token_hash_capacity,
            size_of::<HashEntry>(),
            align_of::<HashEntry>(),
        )?;
        Some(Self {
            total_bytes,
            generation_capacity,
            operation_capacity,
            fence_capacity,
            token_capacity,
            operation_hash_capacity,
            fence_hash_capacity,
            token_hash_capacity,
            generation_offset,
            operation_offset,
            operation_hash_offset,
            fence_offset,
            fence_hash_offset,
            token_offset,
            token_hash_offset,
        })
    }
}

fn hash_capacity(slot_capacity: usize) -> Option<usize> {
    if slot_capacity == 0 {
        return Some(0);
    }
    slot_capacity.checked_mul(2)?.checked_next_power_of_two()
}

/// Installs shared-memory hooks only during `shared_preload_libraries`.
pub(crate) unsafe fn register_shared_memory_hooks() {
    if !unsafe { pg_sys::process_shared_preload_libraries_in_progress } {
        return;
    }
    unsafe {
        PREV_SHMEM_REQUEST_HOOK = pg_sys::shmem_request_hook;
        pg_sys::shmem_request_hook = Some(scan_registry_shmem_request);
        PREV_SHMEM_STARTUP_HOOK = pg_sys::shmem_startup_hook;
        pg_sys::shmem_startup_hook = Some(scan_registry_shmem_startup);
    }
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn scan_registry_shmem_request() {
    unsafe {
        if let Some(previous) = PREV_SHMEM_REQUEST_HOOK {
            previous();
        }
        let Some(layout) = configured_layout() else {
            return;
        };
        pg_sys::RequestAddinShmemSpace(layout.total_bytes);
        pg_sys::RequestNamedLWLockTranche(c"ecaz_distann_scan_registry".as_ptr(), 1);
    }
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn scan_registry_shmem_startup() {
    unsafe {
        if let Some(previous) = PREV_SHMEM_STARTUP_HOOK {
            previous();
        }
        let Some(layout) = configured_layout() else {
            return;
        };
        // PostgreSQL's add-in shmem initialization lock is MainLWLockArray[21],
        // the same stable accessor used by pgrx's PgLwLock implementation.
        let init_lock = &raw mut (*pg_sys::MainLWLockArray.add(21)).lock;
        pg_sys::LWLockAcquire(init_lock, pg_sys::LWLockMode::LW_EXCLUSIVE);
        let mut found = false;
        let base = pg_sys::ShmemInitStruct(
            c"ecaz_distann_scan_registry".as_ptr(),
            layout.total_bytes,
            &mut found,
        )
        .cast::<u8>();
        if base.is_null() {
            pg_sys::LWLockRelease(init_lock);
            return;
        }
        if !found {
            ptr::write_bytes(base, 0, layout.total_bytes);
            base.cast::<RegistryHeader>().write(RegistryHeader {
                magic: REGISTRY_MAGIC,
                version: REGISTRY_VERSION,
                total_bytes: layout.total_bytes,
                generation_capacity: layout.generation_capacity as u32,
                operation_capacity: layout.operation_capacity as u32,
                fence_capacity: layout.fence_capacity as u32,
                token_capacity: layout.token_capacity as u32,
                operation_hash_capacity: layout.operation_hash_capacity as u32,
                fence_hash_capacity: layout.fence_hash_capacity as u32,
                token_hash_capacity: layout.token_hash_capacity as u32,
                operation_free_head: initial_free_head(layout.operation_capacity),
                fence_free_head: initial_free_head(layout.fence_capacity),
                token_free_head: initial_free_head(layout.token_capacity),
                generation_offset: layout.generation_offset,
                operation_offset: layout.operation_offset,
                operation_hash_offset: layout.operation_hash_offset,
                fence_offset: layout.fence_offset,
                fence_hash_offset: layout.fence_hash_offset,
                token_offset: layout.token_offset,
                token_hash_offset: layout.token_hash_offset,
            });
            initialize_free_lists(base, layout);
        }
        let tranche = pg_sys::GetNamedLWLockTranche(c"ecaz_distann_scan_registry".as_ptr());
        if tranche.is_null() {
            REGISTRY_HEADER = ptr::null_mut();
            REGISTRY_LWLOCK = ptr::null_mut();
            pg_sys::LWLockRelease(init_lock);
            return;
        }
        REGISTRY_HEADER = base.cast::<RegistryHeader>();
        REGISTRY_LWLOCK = &raw mut (*tranche).lock;
        pg_sys::LWLockRelease(init_lock);
    }
}

fn initial_free_head(capacity: usize) -> i32 {
    if capacity == 0 {
        -1
    } else {
        0
    }
}

unsafe fn initialize_free_lists(base: *mut u8, layout: RegistryLayout) {
    let operations = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(layout.operation_offset)
                .cast::<FenceOperationSlot>(),
            layout.operation_capacity,
        )
    };
    let operation_len = operations.len();
    for (index, slot) in operations.iter_mut().enumerate() {
        *slot = FenceOperationSlot {
            next_free: next_free(index, operation_len),
            ..FenceOperationSlot::default()
        };
    }
    let fences = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(layout.fence_offset).cast::<FenceSlot>(),
            layout.fence_capacity,
        )
    };
    let fence_len = fences.len();
    for (index, slot) in fences.iter_mut().enumerate() {
        *slot = FenceSlot {
            next_free: next_free(index, fence_len),
            ..FenceSlot::default()
        };
    }
    let tokens = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(layout.token_offset).cast::<TokenSlot>(),
            layout.token_capacity,
        )
    };
    let token_len = tokens.len();
    for (index, slot) in tokens.iter_mut().enumerate() {
        *slot = TokenSlot {
            next_free: next_free(index, token_len),
            ..TokenSlot::default()
        };
    }
}

fn next_free(index: usize, capacity: usize) -> i32 {
    if index + 1 < capacity {
        (index + 1) as i32
    } else {
        -1
    }
}

struct RegistryLockGuard;

impl Drop for RegistryLockGuard {
    fn drop(&mut self) {
        unsafe {
            if !REGISTRY_LWLOCK.is_null() {
                pg_sys::LWLockRelease(REGISTRY_LWLOCK);
            }
        }
    }
}

unsafe fn lock_registry() -> Result<RegistryLockGuard, RegistryError> {
    if unsafe { REGISTRY_HEADER.is_null() || REGISTRY_LWLOCK.is_null() } {
        return Err(RegistryError::Unavailable);
    }
    unsafe { pg_sys::LWLockAcquire(REGISTRY_LWLOCK, pg_sys::LWLockMode::LW_EXCLUSIVE) };
    Ok(RegistryLockGuard)
}

struct RegistryView<'a> {
    header: &'a mut RegistryHeader,
    generations: &'a mut [BackendGeneration],
    operations: &'a mut [FenceOperationSlot],
    operation_hash: &'a mut [HashEntry],
    fences: &'a mut [FenceSlot],
    fence_hash: &'a mut [HashEntry],
    tokens: &'a mut [TokenSlot],
    token_hash: &'a mut [HashEntry],
}

unsafe fn shared_view() -> Result<RegistryView<'static>, RegistryError> {
    let header = unsafe { REGISTRY_HEADER.as_ref() }.ok_or(RegistryError::Unavailable)?;
    if header.magic != REGISTRY_MAGIC || header.version != REGISTRY_VERSION {
        return Err(RegistryError::CorruptSharedState);
    }
    let expected = RegistryLayout::new(
        header.generation_capacity as usize,
        header.fence_capacity as usize,
        header.token_capacity as usize,
    )
    .ok_or(RegistryError::CorruptSharedState)?;
    if header.total_bytes != expected.total_bytes
        || header.generation_offset != expected.generation_offset
        || header.operation_capacity as usize != expected.operation_capacity
        || header.operation_hash_capacity as usize != expected.operation_hash_capacity
        || header.fence_hash_capacity as usize != expected.fence_hash_capacity
        || header.token_hash_capacity as usize != expected.token_hash_capacity
        || header.operation_offset != expected.operation_offset
        || header.operation_hash_offset != expected.operation_hash_offset
        || header.fence_offset != expected.fence_offset
        || header.fence_hash_offset != expected.fence_hash_offset
        || header.token_offset != expected.token_offset
        || header.token_hash_offset != expected.token_hash_offset
    {
        return Err(RegistryError::CorruptSharedState);
    }
    let base = REGISTRY_HEADER.cast::<u8>();
    let generations = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(header.generation_offset)
                .cast::<BackendGeneration>(),
            header.generation_capacity as usize,
        )
    };
    let operation_hash = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(header.operation_hash_offset).cast::<HashEntry>(),
            header.operation_hash_capacity as usize,
        )
    };
    let operations = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(header.operation_offset)
                .cast::<FenceOperationSlot>(),
            header.operation_capacity as usize,
        )
    };
    let fence_hash = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(header.fence_hash_offset).cast::<HashEntry>(),
            header.fence_hash_capacity as usize,
        )
    };
    let fences = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(header.fence_offset).cast::<FenceSlot>(),
            header.fence_capacity as usize,
        )
    };
    let tokens = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(header.token_offset).cast::<TokenSlot>(),
            header.token_capacity as usize,
        )
    };
    let token_hash = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(header.token_hash_offset).cast::<HashEntry>(),
            header.token_hash_capacity as usize,
        )
    };
    debug_assert_eq!(generations.len(), header.generation_capacity as usize);
    debug_assert_eq!(operations.len(), header.operation_capacity as usize);
    debug_assert_eq!(
        operation_hash.len(),
        header.operation_hash_capacity as usize
    );
    debug_assert_eq!(fences.len(), header.fence_capacity as usize);
    debug_assert_eq!(fence_hash.len(), header.fence_hash_capacity as usize);
    debug_assert_eq!(tokens.len(), header.token_capacity as usize);
    debug_assert_eq!(token_hash.len(), header.token_hash_capacity as usize);
    let header = unsafe { &mut *REGISTRY_HEADER };
    Ok(RegistryView {
        header,
        generations,
        operations,
        operation_hash,
        fences,
        fence_hash,
        tokens,
        token_hash,
    })
}

#[cfg(not(test))]
fn backend_is_live(generations: &[BackendGeneration], owner: BackendOwner) -> bool {
    let Ok(proc_index) = usize::try_from(owner.proc_number) else {
        return false;
    };
    if proc_index >= generations.len() || owner.pid <= 0 || owner.generation == 0 {
        return false;
    }
    unsafe {
        let proc_global = pg_sys::ProcGlobal;
        if proc_global.is_null() || (*proc_global).allProcs.is_null() {
            return false;
        }
        let proc = &*(*proc_global).allProcs.add(proc_index);
        proc.pid == owner.pid && generations[proc_index].generation == owner.generation
    }
}

#[cfg(test)]
fn backend_is_live(_generations: &[BackendGeneration], _owner: BackendOwner) -> bool {
    false
}

fn next_nonzero_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}

fn current_backend_owner() -> Result<BackendOwner, RegistryError> {
    if let Some(owner) = BACKEND_OWNER.get() {
        return Ok(owner);
    }
    let _guard = unsafe { lock_registry()? };
    let view = unsafe { shared_view()? };
    let proc_number = unsafe { pg_sys::MyProcNumber };
    let pid = unsafe { pg_sys::MyProcPid };
    let proc_index = usize::try_from(proc_number).map_err(|_| RegistryError::Unavailable)?;
    let slot = view
        .generations
        .get_mut(proc_index)
        .ok_or(RegistryError::Unavailable)?;
    slot.generation = next_nonzero_generation(slot.generation);
    let owner = BackendOwner {
        proc_number,
        pid,
        generation: slot.generation,
    };
    BACKEND_OWNER.set(Some(owner));
    ensure_exit_callback();
    Ok(owner)
}

fn ensure_exit_callback() {
    EXIT_CALLBACK_REGISTERED.with(|registered| {
        if registered.replace(true) {
            return;
        }
        unsafe {
            pg_sys::before_shmem_exit(
                Some(scan_registry_before_shmem_exit),
                pg_sys::Datum::from(0_usize),
            );
        }
    });
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn scan_registry_before_shmem_exit(_code: c_int, _arg: pg_sys::Datum) {
    let Some(owner) = BACKEND_OWNER.take() else {
        return;
    };
    let Ok(_guard) = (unsafe { lock_registry() }) else {
        return;
    };
    let Ok(mut view) = (unsafe { shared_view() }) else {
        return;
    };
    let _ = view.release_owner_tokens(owner);
    let _ = view.recycle_dropped_fences();
}

fn hash_bytes(parts: &[&[u8]]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for part in parts {
        for byte in *part {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

fn hash_lookup(
    table: &[HashEntry],
    hash: u64,
    mut matches: impl FnMut(usize) -> bool,
) -> Option<(usize, usize)> {
    if table.is_empty() {
        return None;
    }
    let mask = table.len() - 1;
    for probe in 0..table.len() {
        let bucket = (hash as usize).wrapping_add(probe) & mask;
        match table[bucket].slot_plus_one {
            HASH_EMPTY => return None,
            HASH_TOMBSTONE => {}
            value if value > 0 => {
                let slot = usize::try_from(value - 1).ok()?;
                if matches(slot) {
                    return Some((bucket, slot));
                }
            }
            _ => return None,
        }
    }
    None
}

fn hash_insert(table: &mut [HashEntry], hash: u64, slot: usize) -> Result<(), RegistryError> {
    if table.is_empty() {
        return Err(RegistryError::CorruptSharedState);
    }
    let mask = table.len() - 1;
    let mut tombstone = None;
    for probe in 0..table.len() {
        let bucket = (hash as usize).wrapping_add(probe) & mask;
        match table[bucket].slot_plus_one {
            HASH_EMPTY => {
                let target = tombstone.unwrap_or(bucket);
                table[target].slot_plus_one =
                    i32::try_from(slot + 1).map_err(|_| RegistryError::CorruptSharedState)?;
                return Ok(());
            }
            HASH_TOMBSTONE => tombstone.get_or_insert(bucket),
            _ => continue,
        };
    }
    if let Some(bucket) = tombstone {
        table[bucket].slot_plus_one =
            i32::try_from(slot + 1).map_err(|_| RegistryError::CorruptSharedState)?;
        Ok(())
    } else {
        Err(RegistryError::CorruptSharedState)
    }
}

fn hash_backshift_delete(
    table: &mut [HashEntry],
    mut hole: usize,
    mut slot_hash: impl FnMut(usize) -> Result<u64, RegistryError>,
) -> Result<(), RegistryError> {
    if table.is_empty() || hole >= table.len() || table[hole].slot_plus_one <= 0 {
        return Err(RegistryError::CorruptSharedState);
    }
    let mask = table.len() - 1;
    let mut scan = (hole + 1) & mask;
    let mut moves = Vec::new();
    let mut found_empty = false;
    for _ in 0..table.len() {
        let value = table[scan].slot_plus_one;
        if value == HASH_EMPTY {
            found_empty = true;
            break;
        }
        if value == HASH_TOMBSTONE {
            return Err(RegistryError::CorruptSharedState);
        }
        let slot = usize::try_from(value - 1).map_err(|_| RegistryError::CorruptSharedState)?;
        let ideal = slot_hash(slot)? as usize & mask;
        let scan_distance = scan.wrapping_sub(ideal) & mask;
        let hole_distance = hole.wrapping_sub(ideal) & mask;
        if hole_distance < scan_distance {
            moves.push((scan, hole));
            hole = scan;
        }
        scan = (scan + 1) & mask;
    }
    if !found_empty {
        return Err(RegistryError::CorruptSharedState);
    }
    for (from, to) in moves {
        table[to] = table[from];
    }
    table[hole].slot_plus_one = HASH_EMPTY;
    Ok(())
}

fn owner_matches(
    proc_number: pg_sys::ProcNumber,
    pid: c_int,
    generation: u64,
    owner: BackendOwner,
) -> bool {
    proc_number == owner.proc_number && pid == owner.pid && generation == owner.generation
}

impl RegistryView<'_> {
    fn delete_operation_hash(&mut self, bucket: usize) -> Result<(), RegistryError> {
        let operations = &self.operations;
        hash_backshift_delete(self.operation_hash, bucket, |index| {
            let slot = *operations
                .get(index)
                .filter(|slot| slot.in_use != 0)
                .ok_or(RegistryError::CorruptSharedState)?;
            Ok(Self::operation_hash_value(
                BackendOwner {
                    proc_number: slot.proc_number,
                    pid: slot.backend_pid,
                    generation: slot.backend_generation,
                },
                slot.fence_id,
            ))
        })
    }

    fn delete_token_hash(&mut self, bucket: usize) -> Result<(), RegistryError> {
        let tokens = &self.tokens;
        hash_backshift_delete(self.token_hash, bucket, |index| {
            let slot = *tokens
                .get(index)
                .filter(|slot| slot.in_use != 0)
                .ok_or(RegistryError::CorruptSharedState)?;
            Ok(Self::token_hash_value(&ScanTokenIdentity {
                database_oid: slot.database_oid,
                logical_index_uuid: slot.logical_index_uuid,
                epoch_fingerprint: slot.epoch_fingerprint,
                scan_token: slot.scan_token,
            }))
        })
    }

    fn delete_fence_hash(&mut self, bucket: usize) -> Result<(), RegistryError> {
        let fences = &self.fences;
        hash_backshift_delete(self.fence_hash, bucket, |index| {
            let slot = *fences
                .get(index)
                .filter(|slot| slot.in_use != 0)
                .ok_or(RegistryError::CorruptSharedState)?;
            Ok(Self::fence_hash_value(
                slot.database_oid,
                &slot.logical_index_uuid,
            ))
        })
    }

    fn fence_hash_value(database_oid: pg_sys::Oid, logical: &[u8; 16]) -> u64 {
        hash_bytes(&[&database_oid.to_u32().to_le_bytes(), logical])
    }

    fn token_hash_value(identity: &ScanTokenIdentity) -> u64 {
        hash_bytes(&[
            &identity.database_oid.to_u32().to_le_bytes(),
            &identity.logical_index_uuid,
            &identity.scan_token,
        ])
    }

    fn operation_hash_value(owner: BackendOwner, fence_id: u32) -> u64 {
        hash_bytes(&[
            &owner.proc_number.to_le_bytes(),
            &owner.pid.to_le_bytes(),
            &owner.generation.to_le_bytes(),
            &fence_id.to_le_bytes(),
        ])
    }

    fn find_fence_index(&self, database_oid: pg_sys::Oid, logical: &[u8; 16]) -> Option<usize> {
        hash_lookup(
            self.fence_hash,
            Self::fence_hash_value(database_oid, logical),
            |index| {
                self.fences.get(index).is_some_and(|slot| {
                    slot.in_use != 0
                        && slot.database_oid == database_oid
                        && &slot.logical_index_uuid == logical
                })
            },
        )
        .map(|(_, index)| index)
    }

    fn find_token(&self, identity: &ScanTokenIdentity) -> Option<(usize, usize)> {
        hash_lookup(self.token_hash, Self::token_hash_value(identity), |index| {
            self.tokens.get(index).is_some_and(|slot| {
                slot.in_use != 0
                    && slot.database_oid == identity.database_oid
                    && slot.logical_index_uuid == identity.logical_index_uuid
                    && slot.scan_token == identity.scan_token
            })
        })
    }

    fn find_operation(&self, owner: BackendOwner, fence_id: u32) -> Option<(usize, usize)> {
        hash_lookup(
            self.operation_hash,
            Self::operation_hash_value(owner, fence_id),
            |index| {
                self.operations.get(index).is_some_and(|slot| {
                    slot.in_use != 0
                        && slot.fence_id == fence_id
                        && owner_matches(
                            slot.proc_number,
                            slot.backend_pid,
                            slot.backend_generation,
                            owner,
                        )
                })
            },
        )
    }

    fn pop_operation(&mut self) -> Option<usize> {
        let index = usize::try_from(self.header.operation_free_head).ok()?;
        let slot = self.operations.get(index)?;
        self.header.operation_free_head = slot.next_free;
        Some(index)
    }

    fn pop_fence(&mut self) -> Option<usize> {
        let index = usize::try_from(self.header.fence_free_head).ok()?;
        let slot = self.fences.get(index)?;
        self.header.fence_free_head = slot.next_free;
        Some(index)
    }

    fn pop_token(&mut self) -> Option<usize> {
        let index = usize::try_from(self.header.token_free_head).ok()?;
        let slot = self.tokens.get(index)?;
        self.header.token_free_head = slot.next_free;
        Some(index)
    }

    fn push_operation(&mut self, index: usize) {
        self.operations[index] = FenceOperationSlot {
            next_free: self.header.operation_free_head,
            ..Default::default()
        };
        self.header.operation_free_head = index as i32;
    }

    fn push_fence(&mut self, index: usize) {
        self.fences[index] = FenceSlot {
            next_free: self.header.fence_free_head,
            ..Default::default()
        };
        self.header.fence_free_head = index as i32;
    }

    fn push_token(&mut self, index: usize) {
        self.tokens[index] = TokenSlot {
            next_free: self.header.token_free_head,
            ..Default::default()
        };
        self.header.token_free_head = index as i32;
    }

    fn ensure_fence(
        &mut self,
        database_oid: pg_sys::Oid,
        logical: [u8; 16],
    ) -> Result<usize, RegistryError> {
        if let Some(index) = self.find_fence_index(database_oid, &logical) {
            return if self.fences[index].dropped == 0 {
                Ok(index)
            } else {
                Err(RegistryError::LogicalIndexDropped)
            };
        }
        if self.header.fence_free_head < 0 {
            self.recycle_dropped_fences()?;
        }
        let index = self.pop_fence().ok_or(RegistryError::FenceCapacity)?;
        let fence_id = u32::try_from(index + 1).map_err(|_| RegistryError::FenceCapacity)?;
        self.fences[index] = FenceSlot {
            in_use: 1,
            dropped: 0,
            _padding: [0; 2],
            database_oid,
            logical_index_uuid: logical,
            fence_id,
            operation_references: 0,
            token_count: 0,
            next_free: -1,
        };
        if let Err(error) = hash_insert(
            self.fence_hash,
            Self::fence_hash_value(database_oid, &logical),
            index,
        ) {
            self.push_fence(index);
            return Err(error);
        }
        Ok(index)
    }

    fn fence_index_for_id(&self, fence_id: u32) -> Result<usize, RegistryError> {
        let index = fence_id.checked_sub(1).map(|value| value as usize);
        index
            .filter(|index| {
                self.fences
                    .get(*index)
                    .is_some_and(|fence| fence.in_use != 0 && fence.fence_id == fence_id)
            })
            .ok_or(RegistryError::CorruptSharedState)
    }

    fn fence_hash_bucket(&self, index: usize) -> Result<usize, RegistryError> {
        let fence = *self
            .fences
            .get(index)
            .ok_or(RegistryError::CorruptSharedState)?;
        let (bucket, found) = hash_lookup(
            self.fence_hash,
            Self::fence_hash_value(fence.database_oid, &fence.logical_index_uuid),
            |candidate| candidate == index,
        )
        .ok_or(RegistryError::CorruptSharedState)?;
        if found != index {
            return Err(RegistryError::CorruptSharedState);
        }
        Ok(bucket)
    }

    fn remove_operation(&mut self, bucket: usize, index: usize) -> Result<usize, RegistryError> {
        let operation = self.operations[index];
        let fence_index = self.fence_index_for_id(operation.fence_id)?;
        let remaining = self.fences[fence_index]
            .operation_references
            .checked_sub(operation.nesting)
            .ok_or(RegistryError::CorruptSharedState)?;
        if self.fences[fence_index].dropped != 0
            && remaining == 0
            && self.fences[fence_index].token_count == 0
        {
            self.fence_hash_bucket(fence_index)?;
        }
        self.delete_operation_hash(bucket)?;
        self.fences[fence_index].operation_references = remaining;
        self.push_operation(index);
        Ok(fence_index)
    }

    fn remove_token(&mut self, bucket: usize, index: usize) -> Result<usize, RegistryError> {
        let token = self.tokens[index];
        let fence_index = self
            .find_fence_index(token.database_oid, &token.logical_index_uuid)
            .ok_or(RegistryError::CorruptSharedState)?;
        let remaining = self.fences[fence_index]
            .token_count
            .checked_sub(1)
            .ok_or(RegistryError::CorruptSharedState)?;
        if self.fences[fence_index].dropped != 0
            && remaining == 0
            && self.fences[fence_index].operation_references == 0
        {
            self.fence_hash_bucket(fence_index)?;
        }
        self.delete_token_hash(bucket)?;
        self.fences[fence_index].token_count = remaining;
        self.push_token(index);
        Ok(fence_index)
    }

    fn reap_dead_with(
        &mut self,
        mut live: impl FnMut(BackendOwner) -> bool,
    ) -> Result<usize, RegistryError> {
        let mut reaped = 0;
        for index in 0..self.tokens.len() {
            let token = self.tokens[index];
            if token.in_use == 0 {
                continue;
            }
            let owner = BackendOwner {
                proc_number: token.proc_number,
                pid: token.backend_pid,
                generation: token.backend_generation,
            };
            if !live(owner) {
                let identity = ScanTokenIdentity {
                    database_oid: token.database_oid,
                    logical_index_uuid: token.logical_index_uuid,
                    epoch_fingerprint: token.epoch_fingerprint,
                    scan_token: token.scan_token,
                };
                if let Some((bucket, found)) = self.find_token(&identity) {
                    let fence_index = self.remove_token(bucket, found)?;
                    self.recycle_fence_if_eligible(fence_index)?;
                    reaped += 1;
                }
            }
        }
        for index in 0..self.operations.len() {
            let operation = self.operations[index];
            if operation.in_use == 0 {
                continue;
            }
            let owner = BackendOwner {
                proc_number: operation.proc_number,
                pid: operation.backend_pid,
                generation: operation.backend_generation,
            };
            if !live(owner) {
                if let Some((bucket, found)) = self.find_operation(owner, operation.fence_id) {
                    let fence_index = self.remove_operation(bucket, found)?;
                    self.recycle_fence_if_eligible(fence_index)?;
                    reaped += 1;
                }
            }
        }
        Ok(reaped)
    }

    fn reap_dead(&mut self) -> Result<usize, RegistryError> {
        let generations = self.generations.as_ptr();
        let len = self.generations.len();
        self.reap_dead_with(|owner| {
            let generations = unsafe { std::slice::from_raw_parts(generations, len) };
            backend_is_live(generations, owner)
        })
    }

    fn release_owner_tokens(&mut self, owner: BackendOwner) -> Result<usize, RegistryError> {
        let mut released = 0;
        for index in 0..self.tokens.len() {
            let token = self.tokens[index];
            if token.in_use != 0
                && owner_matches(
                    token.proc_number,
                    token.backend_pid,
                    token.backend_generation,
                    owner,
                )
            {
                let identity = ScanTokenIdentity {
                    database_oid: token.database_oid,
                    logical_index_uuid: token.logical_index_uuid,
                    epoch_fingerprint: token.epoch_fingerprint,
                    scan_token: token.scan_token,
                };
                if let Some((bucket, found)) = self.find_token(&identity) {
                    let fence_index = self.remove_token(bucket, found)?;
                    self.recycle_fence_if_eligible(fence_index)?;
                    released += 1;
                }
            }
        }
        Ok(released)
    }

    fn acquire_fence_operation(
        &mut self,
        fence_index: usize,
        owner: BackendOwner,
    ) -> Result<u32, RegistryError> {
        let fence_id = self
            .fences
            .get(fence_index)
            .filter(|slot| slot.in_use != 0)
            .ok_or(RegistryError::CorruptSharedState)?
            .fence_id;
        let next_references = self.fences[fence_index]
            .operation_references
            .checked_add(1)
            .ok_or(RegistryError::CorruptSharedState)?;
        if let Some((_, index)) = self.find_operation(owner, fence_id) {
            let next_nesting = self.operations[index]
                .nesting
                .checked_add(1)
                .ok_or(RegistryError::CorruptSharedState)?;
            self.operations[index].nesting = next_nesting;
        } else {
            if self.header.operation_free_head < 0 {
                self.reap_dead()?;
            }
            let index = self.pop_operation().ok_or(RegistryError::FenceCapacity)?;
            self.operations[index] = FenceOperationSlot {
                in_use: 1,
                _padding: [0; 3],
                fence_id,
                proc_number: owner.proc_number,
                backend_pid: owner.pid,
                backend_generation: owner.generation,
                nesting: 1,
                next_free: -1,
            };
            if let Err(error) = hash_insert(
                self.operation_hash,
                Self::operation_hash_value(owner, fence_id),
                index,
            ) {
                self.push_operation(index);
                return Err(error);
            }
        }
        self.fences[fence_index].operation_references = next_references;
        Ok(fence_id)
    }

    fn release_fence_operation(
        &mut self,
        owner: BackendOwner,
        fence_id: u32,
    ) -> Result<(), RegistryError> {
        let (bucket, index) = self
            .find_operation(owner, fence_id)
            .ok_or(RegistryError::CorruptSharedState)?;
        if self.operations[index].nesting == 0 {
            return Err(RegistryError::CorruptSharedState);
        }
        let fence_index = self.fence_index_for_id(fence_id)?;
        let next_references = self.fences[fence_index]
            .operation_references
            .checked_sub(1)
            .ok_or(RegistryError::CorruptSharedState)?;
        if self.fences[fence_index].dropped != 0
            && next_references == 0
            && self.fences[fence_index].token_count == 0
        {
            self.fence_hash_bucket(fence_index)?;
        }
        self.operations[index].nesting -= 1;
        self.fences[fence_index].operation_references = next_references;
        if self.operations[index].nesting == 0 {
            self.delete_operation_hash(bucket)?;
            self.push_operation(index);
        }
        self.recycle_fence_if_eligible(fence_index)?;
        Ok(())
    }

    fn recycle_fence_if_eligible(&mut self, index: usize) -> Result<bool, RegistryError> {
        let fence = *self
            .fences
            .get(index)
            .ok_or(RegistryError::CorruptSharedState)?;
        if fence.in_use == 0
            || fence.dropped == 0
            || fence.operation_references != 0
            || fence.token_count != 0
        {
            return Ok(false);
        }
        let bucket = self.fence_hash_bucket(index)?;
        self.delete_fence_hash(bucket)?;
        self.push_fence(index);
        Ok(true)
    }

    fn recycle_dropped_fences(&mut self) -> Result<usize, RegistryError> {
        let mut recycled = 0;
        for index in 0..self.fences.len() {
            let fence = self.fences[index];
            if fence.in_use != 0
                && fence.dropped != 0
                && fence.operation_references == 0
                && fence.token_count == 0
                && self.recycle_fence_if_eligible(index)?
            {
                recycled += 1;
            }
        }
        Ok(recycled)
    }

    fn discard_empty_fence(&mut self, index: usize) -> Result<(), RegistryError> {
        let fence = *self
            .fences
            .get(index)
            .ok_or(RegistryError::CorruptSharedState)?;
        if fence.in_use == 0 || fence.operation_references != 0 || fence.token_count != 0 {
            return Err(RegistryError::CorruptSharedState);
        }
        let bucket = self.fence_hash_bucket(index)?;
        self.delete_fence_hash(bucket)?;
        self.push_fence(index);
        Ok(())
    }

    fn register_with(
        &mut self,
        identity: ScanTokenIdentity,
        owner: BackendOwner,
        mut live: impl FnMut(BackendOwner) -> bool,
    ) -> Result<RegisterOutcome, RegistryError> {
        let existing_fence =
            self.find_fence_index(identity.database_oid, &identity.logical_index_uuid);
        if existing_fence.is_some_and(|index| self.fences[index].dropped != 0) {
            return Err(RegistryError::LogicalIndexDropped);
        }
        if let Some((bucket, index)) = self.find_token(&identity) {
            let token = self.tokens[index];
            let existing_owner = BackendOwner {
                proc_number: token.proc_number,
                pid: token.backend_pid,
                generation: token.backend_generation,
            };
            if !owner_matches(
                token.proc_number,
                token.backend_pid,
                token.backend_generation,
                owner,
            ) {
                if live(existing_owner) {
                    return Err(RegistryError::TokenConflict);
                }
                let fence_index = self.remove_token(bucket, index)?;
                self.recycle_fence_if_eligible(fence_index)?;
            } else {
                return if token.epoch_fingerprint == identity.epoch_fingerprint {
                    Ok(RegisterOutcome::AlreadyRegistered)
                } else {
                    Err(RegistryError::TokenConflict)
                };
            }
        }
        if self.header.token_free_head < 0 {
            self.reap_dead_with(&mut live)?;
        }
        if self.header.token_free_head < 0 {
            return Err(RegistryError::TokenCapacity);
        }
        let (fence_index, created_fence) = if let Some(index) = existing_fence {
            (index, false)
        } else {
            let index = match self.ensure_fence(identity.database_oid, identity.logical_index_uuid)
            {
                Err(RegistryError::FenceCapacity) => {
                    self.reap_dead_with(&mut live)?;
                    self.ensure_fence(identity.database_oid, identity.logical_index_uuid)?
                }
                result => result?,
            };
            (index, true)
        };
        let next_token_count = self.fences[fence_index]
            .token_count
            .checked_add(1)
            .ok_or(RegistryError::CorruptSharedState)?;
        let Some(index) = self.pop_token() else {
            if created_fence {
                self.discard_empty_fence(fence_index)?;
            }
            return Err(RegistryError::CorruptSharedState);
        };
        self.tokens[index] = TokenSlot {
            in_use: 1,
            _padding: [0; 3],
            database_oid: identity.database_oid,
            logical_index_uuid: identity.logical_index_uuid,
            epoch_fingerprint: identity.epoch_fingerprint,
            scan_token: identity.scan_token,
            proc_number: owner.proc_number,
            backend_pid: owner.pid,
            backend_generation: owner.generation,
            next_free: -1,
            _tail_padding: [0; 4],
        };
        if let Err(error) = hash_insert(self.token_hash, Self::token_hash_value(&identity), index) {
            self.push_token(index);
            if created_fence {
                self.discard_empty_fence(fence_index)?;
            }
            return Err(error);
        }
        self.fences[fence_index].token_count = next_token_count;
        Ok(RegisterOutcome::Registered)
    }

    fn register(
        &mut self,
        identity: ScanTokenIdentity,
        owner: BackendOwner,
    ) -> Result<RegisterOutcome, RegistryError> {
        let generations = self.generations.as_ptr();
        let len = self.generations.len();
        self.register_with(identity, owner, |candidate| {
            let generations = unsafe { std::slice::from_raw_parts(generations, len) };
            backend_is_live(generations, candidate)
        })
    }

    fn release(
        &mut self,
        identity: ScanTokenIdentity,
        owner: BackendOwner,
    ) -> Result<bool, RegistryError> {
        let Some((bucket, index)) = self.find_token(&identity) else {
            return Ok(false);
        };
        let token = self.tokens[index];
        if token.epoch_fingerprint != identity.epoch_fingerprint
            || !owner_matches(
                token.proc_number,
                token.backend_pid,
                token.backend_generation,
                owner,
            )
        {
            return Ok(false);
        }
        let fence_index = self.remove_token(bucket, index)?;
        self.recycle_fence_if_eligible(fence_index)?;
        Ok(true)
    }

    fn token_count_for_fingerprint(
        &self,
        database_oid: pg_sys::Oid,
        logical_index_uuid: [u8; 16],
        epoch_fingerprint: [u8; 34],
    ) -> Result<u32, RegistryError> {
        let count = self
            .tokens
            .iter()
            .filter(|token| {
                token.in_use != 0
                    && token.database_oid == database_oid
                    && token.logical_index_uuid == logical_index_uuid
                    && token.epoch_fingerprint == epoch_fingerprint
            })
            .count();
        u32::try_from(count).map_err(|_| RegistryError::CorruptSharedState)
    }

    fn mark_dropped(
        &mut self,
        database_oid: pg_sys::Oid,
        logical: [u8; 16],
    ) -> Result<bool, RegistryError> {
        let Some(index) = self.find_fence_index(database_oid, &logical) else {
            return Ok(false);
        };
        if self.fences[index].operation_references == 0 && self.fences[index].token_count == 0 {
            self.fence_hash_bucket(index)?;
        }
        self.fences[index].dropped = 1;
        self.recycle_fence_if_eligible(index)?;
        Ok(true)
    }
}

pub(crate) fn register_scan_token(
    logical_index_uuid: Uuid,
    epoch_fingerprint: [u8; 34],
    scan_token: Uuid,
) -> Result<RegisterOutcome, RegistryError> {
    let owner = current_backend_owner()?;
    let identity = ScanTokenIdentity {
        database_oid: unsafe { pg_sys::MyDatabaseId },
        logical_index_uuid: *logical_index_uuid.as_bytes(),
        epoch_fingerprint,
        scan_token: *scan_token.as_bytes(),
    };
    let _guard = unsafe { lock_registry()? };
    unsafe { shared_view()? }.register(identity, owner)
}

pub(crate) fn release_scan_token(
    logical_index_uuid: Uuid,
    epoch_fingerprint: [u8; 34],
    scan_token: Uuid,
) -> Result<bool, RegistryError> {
    let owner = current_backend_owner()?;
    let identity = ScanTokenIdentity {
        database_oid: unsafe { pg_sys::MyDatabaseId },
        logical_index_uuid: *logical_index_uuid.as_bytes(),
        epoch_fingerprint,
        scan_token: *scan_token.as_bytes(),
    };
    let _guard = unsafe { lock_registry()? };
    unsafe { shared_view()? }.release(identity, owner)
}

/// Exact scan registration owned by executor state. Registration happens under
/// the logical-index shared fence; normal executor shutdown releases eagerly,
/// while abrupt backend death remains covered by the shared registry reaper.
pub(crate) struct ScanTokenGuard {
    logical_index_uuid: Uuid,
    epoch_fingerprint: [u8; 34],
    scan_token: Uuid,
    active: bool,
}

impl ScanTokenGuard {
    pub(crate) fn register(
        logical_index_uuid: Uuid,
        epoch_fingerprint: [u8; 34],
    ) -> Result<Self, RegistryError> {
        let mut token_bytes = [0u8; 16];
        if !unsafe { pg_sys::pg_strong_random(token_bytes.as_mut_ptr().cast(), token_bytes.len()) }
        {
            return Err(RegistryError::CorruptSharedState);
        }
        token_bytes[6] = (token_bytes[6] & 0x0f) | 0x40;
        token_bytes[8] = (token_bytes[8] & 0x3f) | 0x80;
        let scan_token = Uuid::from_bytes(token_bytes);
        let database_oid = unsafe { pg_sys::MyDatabaseId };
        with_scan_registration_fence(database_oid, *logical_index_uuid.as_bytes(), || {
            register_scan_token(logical_index_uuid, epoch_fingerprint, scan_token)
        })??;
        Ok(Self {
            logical_index_uuid,
            epoch_fingerprint,
            scan_token,
            active: true,
        })
    }

    pub(crate) fn release(mut self) -> Result<bool, RegistryError> {
        let released = release_scan_token(
            self.logical_index_uuid,
            self.epoch_fingerprint,
            self.scan_token,
        )?;
        self.active = false;
        Ok(released)
    }
}

impl Drop for ScanTokenGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let _ = release_scan_token(
            self.logical_index_uuid,
            self.epoch_fingerprint,
            self.scan_token,
        );
        self.active = false;
    }
}

pub(crate) fn live_token_count_for_fingerprint(
    logical_index_uuid: Uuid,
    epoch_fingerprint: [u8; 34],
) -> Result<u32, RegistryError> {
    let database_oid = unsafe { pg_sys::MyDatabaseId };
    let _guard = unsafe { lock_registry()? };
    let mut view = unsafe { shared_view()? };
    view.reap_dead()?;
    view.token_count_for_fingerprint(
        database_oid,
        *logical_index_uuid.as_bytes(),
        epoch_fingerprint,
    )
}

pub(crate) fn mark_logical_index_dropped(
    database_oid: pg_sys::Oid,
    logical_index_uuid: [u8; 16],
) -> Result<bool, RegistryError> {
    let _guard = unsafe { lock_registry()? };
    unsafe { shared_view()? }.mark_dropped(database_oid, logical_index_uuid)
}

/// Pins a fence-map identity while a later slice waits for or holds its
/// heavyweight fence.  The exact owner incarnation is shared-state-visible so
/// abrupt backend death can reap the operation reference before recycling a
/// dropped UUID's fence id.
struct FenceOperationReference {
    owner: BackendOwner,
    fence_id: u32,
    active: bool,
}

impl Drop for FenceOperationReference {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let Ok(_guard) = (unsafe { lock_registry() }) else {
            return;
        };
        if let Ok(mut view) = unsafe { shared_view() } {
            let _ = view.release_fence_operation(self.owner, self.fence_id);
        }
        self.active = false;
    }
}

fn acquire_fence_operation_reference(
    database_oid: pg_sys::Oid,
    logical_index_uuid: [u8; 16],
) -> Result<FenceOperationReference, RegistryError> {
    let owner = current_backend_owner()?;
    let _guard = unsafe { lock_registry()? };
    let mut view = unsafe { shared_view()? };
    let existing_fence = view.find_fence_index(database_oid, &logical_index_uuid);
    let fence_index = match view.ensure_fence(database_oid, logical_index_uuid) {
        Err(RegistryError::FenceCapacity) => {
            view.reap_dead()?;
            view.ensure_fence(database_oid, logical_index_uuid)?
        }
        result => result?,
    };
    let fence_id = match view.acquire_fence_operation(fence_index, owner) {
        Ok(fence_id) => fence_id,
        Err(error) => {
            if existing_fence.is_none() {
                view.discard_empty_fence(fence_index)?;
            }
            return Err(error);
        }
    };
    Ok(FenceOperationReference {
        owner,
        fence_id,
        active: true,
    })
}

struct XactFenceReference {
    id: u64,
    pending_subid: pg_sys::SubTransactionId,
    tag: pg_sys::LOCKTAG,
    reference: FenceOperationReference,
}

fn fence_lock_tag(database_oid: pg_sys::Oid, fence_id: u32) -> pg_sys::LOCKTAG {
    pg_sys::LOCKTAG {
        locktag_field1: database_oid.to_u32(),
        locktag_field2: 0x4543_4453, // "ECDS" lock namespace
        locktag_field3: fence_id,
        locktag_field4: 1,
        locktag_type: pg_sys::LockTagType::LOCKTAG_ADVISORY as u8,
        locktag_lockmethodid: pg_sys::USER_LOCKMETHOD as u8,
    }
}

fn acquire_heavyweight_lock(
    tag: &pg_sys::LOCKTAG,
    mode: pg_sys::LOCKMODE,
    session: bool,
) -> Result<(), RegistryError> {
    let result = unsafe { pg_sys::LockAcquire(tag, mode, session, false) };
    if result == pg_sys::LockAcquireResult::LOCKACQUIRE_OK
        || result == pg_sys::LockAcquireResult::LOCKACQUIRE_ALREADY_HELD
    {
        Ok(())
    } else {
        Err(RegistryError::CorruptSharedState)
    }
}

fn drop_after_confirmed_lock_release<T>(value: T, released: bool) {
    if released {
        drop(value);
    } else {
        std::mem::forget(value);
    }
}

/// Runs one registration critical section under the logical index's session
/// shared heavyweight lock. The operation reference is acquired before the
/// wait and released only after `LockRelease`, including PostgreSQL ERROR.
pub(crate) fn with_scan_registration_fence<R>(
    database_oid: pg_sys::Oid,
    logical_index_uuid: [u8; 16],
    operation: impl FnOnce() -> R,
) -> Result<R, RegistryError> {
    let reference = acquire_fence_operation_reference(database_oid, logical_index_uuid)?;
    let tag = fence_lock_tag(database_oid, reference.fence_id);
    let acquired = Cell::new(false);
    let reference = RefCell::new(Some(reference));
    pg_sys::PgTryBuilder::new(AssertUnwindSafe(|| {
        acquire_heavyweight_lock(&tag, pg_sys::ShareLock as pg_sys::LOCKMODE, true)?;
        acquired.set(true);
        Ok(operation())
    }))
    .finally(|| {
        let mut released = !acquired.get();
        if acquired.get() {
            released =
                unsafe { pg_sys::LockRelease(&tag, pg_sys::ShareLock as pg_sys::LOCKMODE, true) };
        }
        if let Some(reference) = reference.borrow_mut().take() {
            drop_after_confirmed_lock_release(reference, released);
        }
    })
    .execute()
}

fn remove_xact_fence_reference(id: u64) {
    XACT_FENCE_REFERENCES.with(|references| {
        let mut references = references.borrow_mut();
        if let Some(position) = references.iter().position(|entry| entry.id == id) {
            let entry = references.remove(position);
            let released = unsafe {
                pg_sys::LockRelease(&entry.tag, pg_sys::ExclusiveLock as pg_sys::LOCKMODE, true)
            };
            drop_after_confirmed_lock_release(entry.reference, released);
        }
    });
}

/// Takes the logical index's transaction-exclusive retirement fence. The
/// opaque operation reference is retained until PostgreSQL's xact/subxact
/// cleanup has released the corresponding heavyweight lock.
pub(crate) fn acquire_retirement_fence_xact(
    database_oid: pg_sys::Oid,
    logical_index_uuid: [u8; 16],
) -> Result<(), RegistryError> {
    let reference = acquire_fence_operation_reference(database_oid, logical_index_uuid)?;
    let tag = fence_lock_tag(database_oid, reference.fence_id);
    let acquired = Cell::new(false);
    let completed = Cell::new(false);
    let reference = RefCell::new(Some(reference));
    pg_sys::PgTryBuilder::new(AssertUnwindSafe(|| {
        acquire_heavyweight_lock(&tag, pg_sys::ExclusiveLock as pg_sys::LOCKMODE, true)?;
        acquired.set(true);
        completed.set(true);
        Ok(())
    }))
    .finally(|| {
        if !completed.get() {
            let mut released = !acquired.get();
            if acquired.get() {
                released = unsafe {
                    pg_sys::LockRelease(&tag, pg_sys::ExclusiveLock as pg_sys::LOCKMODE, true)
                };
            }
            if let Some(reference) = reference.borrow_mut().take() {
                drop_after_confirmed_lock_release(reference, released);
            }
        }
    })
    .execute()?;
    let reference = reference
        .borrow_mut()
        .take()
        .ok_or(RegistryError::CorruptSharedState)?;
    let id = NEXT_XACT_FENCE_ID.with(|next| {
        let id = next.get();
        next.set(id.checked_add(1).expect("fence callback id exhausted"));
        id
    });
    let pending_subid = unsafe { pg_sys::GetCurrentSubTransactionId() };
    XACT_FENCE_REFERENCES.with(|references| {
        references.borrow_mut().push(XactFenceReference {
            id,
            pending_subid,
            tag,
            reference,
        });
    });
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Commit, move || {
        remove_xact_fence_reference(id);
    });
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Abort, move || {
        remove_xact_fence_reference(id);
    });
    pgrx::register_subxact_callback(
        pgrx::PgSubXactCallbackEvent::CommitSub,
        move |my_subid, parent_subid| {
            XACT_FENCE_REFERENCES.with(|references| {
                if let Some(entry) = references
                    .borrow_mut()
                    .iter_mut()
                    .find(|entry| entry.id == id && entry.pending_subid == my_subid)
                {
                    entry.pending_subid = parent_subid;
                }
            });
        },
    );
    pgrx::register_subxact_callback(
        pgrx::PgSubXactCallbackEvent::AbortSub,
        move |my_subid, _parent_subid| {
            let remove = XACT_FENCE_REFERENCES.with(|references| {
                references
                    .borrow()
                    .iter()
                    .any(|entry| entry.id == id && entry.pending_subid == my_subid)
            });
            if remove {
                remove_xact_fence_reference(id);
            }
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    struct TestRegistry {
        header: RegistryHeader,
        generations: Vec<BackendGeneration>,
        operations: Vec<FenceOperationSlot>,
        operation_hash: Vec<HashEntry>,
        fences: Vec<FenceSlot>,
        fence_hash: Vec<HashEntry>,
        tokens: Vec<TokenSlot>,
        token_hash: Vec<HashEntry>,
    }

    impl TestRegistry {
        fn new(generations: usize, fences: usize, tokens: usize) -> Self {
            let layout = RegistryLayout::new(generations, fences, tokens).unwrap();
            let mut registry = Self {
                header: RegistryHeader {
                    magic: REGISTRY_MAGIC,
                    version: REGISTRY_VERSION,
                    total_bytes: layout.total_bytes,
                    generation_capacity: generations as u32,
                    operation_capacity: layout.operation_capacity as u32,
                    fence_capacity: fences as u32,
                    token_capacity: tokens as u32,
                    operation_hash_capacity: layout.operation_hash_capacity as u32,
                    fence_hash_capacity: layout.fence_hash_capacity as u32,
                    token_hash_capacity: layout.token_hash_capacity as u32,
                    operation_free_head: initial_free_head(layout.operation_capacity),
                    fence_free_head: initial_free_head(fences),
                    token_free_head: initial_free_head(tokens),
                    generation_offset: layout.generation_offset,
                    operation_offset: layout.operation_offset,
                    operation_hash_offset: layout.operation_hash_offset,
                    fence_offset: layout.fence_offset,
                    fence_hash_offset: layout.fence_hash_offset,
                    token_offset: layout.token_offset,
                    token_hash_offset: layout.token_hash_offset,
                },
                generations: vec![BackendGeneration::default(); generations],
                operations: vec![FenceOperationSlot::default(); layout.operation_capacity],
                operation_hash: vec![HashEntry::default(); layout.operation_hash_capacity],
                fences: vec![FenceSlot::default(); fences],
                fence_hash: vec![HashEntry::default(); layout.fence_hash_capacity],
                tokens: vec![TokenSlot::default(); tokens],
                token_hash: vec![HashEntry::default(); layout.token_hash_capacity],
            };
            let operation_len = registry.operations.len();
            for (index, slot) in registry.operations.iter_mut().enumerate() {
                slot.next_free = next_free(index, operation_len);
            }
            let fence_len = registry.fences.len();
            for (index, slot) in registry.fences.iter_mut().enumerate() {
                slot.next_free = next_free(index, fence_len);
            }
            let token_len = registry.tokens.len();
            for (index, slot) in registry.tokens.iter_mut().enumerate() {
                slot.next_free = next_free(index, token_len);
            }
            registry
        }

        fn view(&mut self) -> RegistryView<'_> {
            RegistryView {
                header: &mut self.header,
                generations: &mut self.generations,
                operations: &mut self.operations,
                operation_hash: &mut self.operation_hash,
                fences: &mut self.fences,
                fence_hash: &mut self.fence_hash,
                tokens: &mut self.tokens,
                token_hash: &mut self.token_hash,
            }
        }
    }

    fn owner(proc_number: i32, pid: i32, generation: u64) -> BackendOwner {
        BackendOwner {
            proc_number,
            pid,
            generation,
        }
    }

    #[test]
    fn operation_owner_drops_only_after_confirmed_lock_release() {
        struct DropProbe(Rc<Cell<u32>>);
        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let released = Rc::new(Cell::new(0));
        drop_after_confirmed_lock_release(DropProbe(released.clone()), true);
        assert_eq!(released.get(), 1);
        let retained = Rc::new(Cell::new(0));
        drop_after_confirmed_lock_release(DropProbe(retained.clone()), false);
        assert_eq!(retained.get(), 0);
    }

    fn identity(database_oid: u32, logical: u8, fingerprint: u8, token: u8) -> ScanTokenIdentity {
        ScanTokenIdentity {
            database_oid: pg_sys::Oid::from(database_oid),
            logical_index_uuid: [logical; 16],
            epoch_fingerprint: [fingerprint; 34],
            scan_token: [token; 16],
        }
    }

    #[test]
    fn layout_is_checked_and_nonoverlapping() {
        let layout = RegistryLayout::new(32, 4_096, 65_536).unwrap();
        assert_eq!(layout.operation_capacity, 32 * OPERATION_SLOTS_PER_BACKEND);
        assert!(layout.generation_offset >= size_of::<RegistryHeader>());
        assert!(
            layout.generation_offset + 32 * size_of::<BackendGeneration>()
                <= layout.operation_offset
        );
        assert!(
            layout.operation_offset + layout.operation_capacity * size_of::<FenceOperationSlot>()
                <= layout.operation_hash_offset
        );
        assert!(
            layout.operation_hash_offset + layout.operation_hash_capacity * size_of::<HashEntry>()
                <= layout.fence_offset
        );
        assert!(
            layout.fence_offset + layout.fence_capacity * size_of::<FenceSlot>()
                <= layout.fence_hash_offset
        );
        assert!(
            layout.fence_hash_offset + layout.fence_hash_capacity * size_of::<HashEntry>()
                <= layout.token_offset
        );
        assert!(
            layout.token_offset + layout.token_capacity * size_of::<TokenSlot>()
                <= layout.token_hash_offset
        );
        assert!(
            layout.token_hash_offset + layout.token_hash_capacity * size_of::<HashEntry>()
                <= layout.total_bytes
        );
        assert!(RegistryLayout::new(usize::MAX, 1, 1).is_none());
    }

    #[test]
    fn zero_and_one_token_capacity_are_fail_closed() {
        let live = owner(0, 100, 1);
        let mut zero = TestRegistry::new(1, 1, 0);
        assert_eq!(
            zero.view()
                .register_with(identity(1, 1, 1, 1), live, |_| true),
            Err(RegistryError::TokenCapacity)
        );
        assert!(zero.fences.iter().all(|slot| slot.in_use == 0));

        let mut one = TestRegistry::new(1, 1, 1);
        assert_eq!(
            one.view()
                .register_with(identity(1, 1, 1, 1), live, |_| true),
            Ok(RegisterOutcome::Registered)
        );
        assert_eq!(
            one.view()
                .register_with(identity(1, 1, 1, 2), live, |_| true),
            Err(RegistryError::TokenCapacity)
        );
        assert_eq!(one.fences[0].operation_references, 0);
        assert!(one.operations.iter().all(|slot| slot.in_use == 0));
    }

    #[test]
    fn zero_one_and_max_plus_one_fence_capacity_are_fail_closed() {
        let live = owner(0, 100, 1);
        let mut zero = TestRegistry::new(1, 0, 1);
        assert_eq!(
            zero.view()
                .register_with(identity(1, 1, 1, 1), live, |_| true),
            Err(RegistryError::FenceCapacity)
        );

        const MAX: usize = 4;
        let mut bounded = TestRegistry::new(1, MAX, MAX + 1);
        for logical in 0..MAX as u8 {
            bounded
                .view()
                .register_with(identity(1, logical, 1, logical), live, |_| true)
                .unwrap();
        }
        assert_eq!(
            bounded
                .view()
                .register_with(identity(1, MAX as u8, 1, MAX as u8), live, |_| true,),
            Err(RegistryError::FenceCapacity)
        );
        assert_eq!(
            bounded
                .fences
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            MAX
        );
    }

    #[test]
    fn exact_replay_is_idempotent_and_conflict_preserves_slot() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 1, 1);
        let exact = identity(1, 1, 7, 9);
        assert_eq!(
            registry.view().register_with(exact, live, |_| true),
            Ok(RegisterOutcome::Registered)
        );
        assert_eq!(
            registry.view().register_with(exact, live, |_| true),
            Ok(RegisterOutcome::AlreadyRegistered)
        );
        assert_eq!(
            registry
                .view()
                .register_with(identity(1, 1, 8, 9), live, |_| true),
            Err(RegistryError::TokenConflict)
        );
        assert_eq!(
            registry
                .tokens
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
        assert!(registry
            .fence_hash
            .iter()
            .all(|entry| entry.slot_plus_one != HASH_TOMBSTONE));
        assert_eq!(registry.tokens[0].epoch_fingerprint, [7; 34]);
    }

    #[test]
    fn live_cross_owner_replay_conflicts_and_release_is_owner_exact() {
        let first = owner(0, 100, 1);
        let second = owner(1, 200, 1);
        let mut registry = TestRegistry::new(2, 1, 1);
        let exact = identity(1, 1, 7, 9);
        assert_eq!(
            registry.view().register_with(exact, first, |_| true),
            Ok(RegisterOutcome::Registered)
        );
        assert_eq!(
            registry.view().register_with(exact, second, |_| true),
            Err(RegistryError::TokenConflict)
        );
        assert!(!registry.view().release(exact, second).unwrap());
        assert_eq!(
            registry
                .tokens
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
        assert!(registry.view().release(exact, first).unwrap());
    }

    #[test]
    fn database_and_logical_uuid_namespace_tokens() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 3, 3);
        for request in [
            identity(1, 1, 7, 9),
            identity(2, 1, 8, 9),
            identity(1, 2, 9, 9),
        ] {
            assert_eq!(
                registry.view().register_with(request, live, |_| true),
                Ok(RegisterOutcome::Registered)
            );
        }
        assert_eq!(
            registry
                .fences
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            3
        );
        assert_eq!(
            registry
                .tokens
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            3
        );
    }

    #[test]
    fn dead_backend_reaping_uses_pid_and_generation() {
        let first = owner(0, 100, 4);
        let replacement = owner(0, 101, 5);
        let mut registry = TestRegistry::new(1, 1, 1);
        assert_eq!(
            registry
                .view()
                .register_with(identity(1, 1, 7, 1), first, |_| true),
            Ok(RegisterOutcome::Registered)
        );
        assert_eq!(
            registry
                .view()
                .register_with(identity(1, 1, 7, 2), replacement, |candidate| candidate
                    == replacement,),
            Ok(RegisterOutcome::Registered)
        );
        assert_eq!(registry.tokens[0].backend_pid, replacement.pid);
        assert_eq!(
            registry.tokens[0].backend_generation,
            replacement.generation
        );
    }

    #[test]
    fn normal_owner_exit_releases_only_exact_incarnation() {
        let first = owner(0, 100, 4);
        let second = owner(1, 200, 8);
        let mut registry = TestRegistry::new(2, 1, 2);
        registry
            .view()
            .register_with(identity(1, 1, 7, 1), first, |_| true)
            .unwrap();
        registry
            .view()
            .register_with(identity(1, 1, 7, 2), second, |_| true)
            .unwrap();
        assert_eq!(registry.view().release_owner_tokens(first), Ok(1));
        assert_eq!(
            registry
                .tokens
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
        assert_eq!(registry.tokens[1].backend_pid, second.pid);
    }

    #[test]
    fn fence_capacity_recycles_only_dropped_unreferenced_uuid() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 1, 2);
        let first = identity(1, 1, 7, 1);
        registry
            .view()
            .register_with(first, live, |_| true)
            .unwrap();
        assert_eq!(
            registry
                .view()
                .register_with(identity(1, 2, 7, 2), live, |_| true),
            Err(RegistryError::FenceCapacity)
        );
        assert_eq!(registry.view().mark_dropped(1.into(), [1; 16]), Ok(true));
        assert_eq!(registry.fences[0].dropped, 1);
        assert!(registry.view().release(first, live).unwrap());
        assert_eq!(registry.fences[0].in_use, 0);
        assert_eq!(
            registry
                .view()
                .register_with(identity(1, 2, 7, 2), live, |_| true),
            Ok(RegisterOutcome::Registered)
        );
        assert_eq!(registry.fences[0].fence_id, 1);
    }

    #[test]
    fn operation_reference_prevents_dropped_fence_recycling() {
        let mut registry = TestRegistry::new(1, 1, 0);
        let index = registry.view().ensure_fence(1.into(), [1; 16]).unwrap();
        let live = owner(0, 100, 1);
        let fence_id = registry
            .view()
            .acquire_fence_operation(index, live)
            .unwrap();
        assert_eq!(registry.view().mark_dropped(1.into(), [1; 16]), Ok(true));
        assert_eq!(registry.view().recycle_dropped_fences(), Ok(0));
        registry
            .view()
            .release_fence_operation(live, fence_id)
            .unwrap();
        assert_eq!(registry.fences[index].in_use, 0);
    }

    #[test]
    fn one_owner_can_hold_multiple_distinct_fence_operations() {
        let mut registry = TestRegistry::new(1, 2, 0);
        let live = owner(0, 100, 1);
        let first = registry.view().ensure_fence(1.into(), [1; 16]).unwrap();
        let second = registry.view().ensure_fence(1.into(), [2; 16]).unwrap();
        let first_id = registry
            .view()
            .acquire_fence_operation(first, live)
            .unwrap();
        let second_id = registry
            .view()
            .acquire_fence_operation(second, live)
            .unwrap();
        assert_ne!(first_id, second_id);
        assert_eq!(
            registry
                .operations
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            2
        );
        registry
            .view()
            .release_fence_operation(live, first_id)
            .unwrap();
        assert_eq!(
            registry
                .operations
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
        registry
            .view()
            .release_fence_operation(live, second_id)
            .unwrap();
    }

    #[test]
    fn exit_releases_tokens_but_operation_waits_for_exact_liveness_reap() {
        let stale = owner(0, 100, 1);
        let replacement = owner(0, 101, 2);
        let mut registry = TestRegistry::new(1, 1, 1);
        let token = identity(1, 1, 7, 1);
        registry
            .view()
            .register_with(token, stale, |_| true)
            .unwrap();
        let fence = registry
            .view()
            .find_fence_index(1.into(), &[1; 16])
            .unwrap();
        registry
            .view()
            .acquire_fence_operation(fence, stale)
            .unwrap();
        assert_eq!(registry.view().release_owner_tokens(stale), Ok(1));
        assert_eq!(
            registry
                .operations
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
        assert_eq!(
            registry
                .view()
                .reap_dead_with(|candidate| candidate == stale),
            Ok(0)
        );
        assert_eq!(
            registry
                .view()
                .reap_dead_with(|candidate| candidate == replacement),
            Ok(1)
        );
        assert_eq!(
            registry
                .operations
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            0
        );
    }

    #[test]
    fn dropped_uuid_churn_does_not_exhaust_live_fence_capacity() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 2, 2);
        let permanent = identity(1, 0xF0, 7, 0xF0);
        registry
            .view()
            .register_with(permanent, live, |_| true)
            .unwrap();

        for generation in 1..=32_u8 {
            let transient = identity(1, generation, 7, generation);
            registry
                .view()
                .register_with(transient, live, |_| true)
                .unwrap();
            assert_eq!(
                registry
                    .view()
                    .mark_dropped(pg_sys::Oid::from(1), [generation; 16]),
                Ok(true)
            );
            assert!(registry.view().release(transient, live).unwrap());
        }

        assert!(registry.tokens.iter().any(|slot| {
            slot.in_use != 0 && slot.logical_index_uuid == permanent.logical_index_uuid
        }));
        assert_eq!(
            registry
                .fences
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
        assert!(registry
            .fence_hash
            .iter()
            .all(|entry| entry.slot_plus_one != HASH_TOMBSTONE));
    }

    #[test]
    fn abrupt_backend_reaps_operation_reference_before_recycling() {
        let stale = owner(0, 100, 1);
        let replacement = owner(0, 101, 2);
        let mut registry = TestRegistry::new(1, 1, 0);
        let index = registry.view().ensure_fence(1.into(), [1; 16]).unwrap();
        registry
            .view()
            .acquire_fence_operation(index, stale)
            .unwrap();
        registry.fences[index].dropped = 1;
        assert_eq!(
            registry.view().reap_dead_with(|owner| owner == replacement),
            Ok(1)
        );
        assert_eq!(registry.fences[index].in_use, 0);
    }

    #[test]
    fn max_plus_one_capacity_does_not_mutate_existing_tokens() {
        const MAX: usize = 8;
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 1, MAX);
        for token in 0..MAX as u8 {
            registry
                .view()
                .register_with(identity(1, 1, 7, token), live, |_| true)
                .unwrap();
        }
        assert_eq!(
            registry
                .view()
                .register_with(identity(1, 1, 7, MAX as u8), live, |_| true),
            Err(RegistryError::TokenCapacity)
        );
        assert_eq!(
            registry
                .tokens
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            MAX
        );
    }

    #[test]
    fn fingerprint_count_is_exact_not_logical_index_wide() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 1, 3);
        for identity in [
            identity(1, 1, 7, 1),
            identity(1, 1, 7, 2),
            identity(1, 1, 8, 3),
        ] {
            registry
                .view()
                .register_with(identity, live, |_| true)
                .unwrap();
        }
        assert_eq!(
            registry
                .view()
                .token_count_for_fingerprint(1.into(), [1; 16], [7; 34]),
            Ok(2)
        );
        assert_eq!(
            registry
                .view()
                .token_count_for_fingerprint(1.into(), [1; 16], [8; 34]),
            Ok(1)
        );
        assert_eq!(
            registry
                .view()
                .token_count_for_fingerprint(1.into(), [2; 16], [7; 34]),
            Ok(0)
        );
    }

    #[test]
    fn token_capacity_preflight_does_not_leak_a_new_fence() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 2, 1);
        registry
            .view()
            .register_with(identity(1, 1, 7, 1), live, |_| true)
            .unwrap();
        assert_eq!(
            registry
                .view()
                .register_with(identity(1, 2, 7, 2), live, |_| true),
            Err(RegistryError::TokenCapacity)
        );
        assert_eq!(
            registry
                .fences
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
        assert!(registry
            .view()
            .find_fence_index(1.into(), &[2; 16])
            .is_none());
    }

    #[test]
    fn stable_error_categories_are_explicit() {
        assert!(RegistryError::Unavailable
            .stable_message()
            .starts_with("EC_EPOCH_REGISTRY_UNAVAILABLE:"));
        assert!(RegistryError::TokenCapacity
            .stable_message()
            .starts_with("EC_EPOCH_PIN_CAPACITY:"));
        assert!(RegistryError::TokenConflict
            .stable_message()
            .starts_with("EC_EPOCH_PIN_CONFLICT:"));
    }

    #[test]
    fn hash_collisions_survive_head_and_middle_backshift_deletion() {
        let mut table = vec![HashEntry::default(); 8];
        for slot in 0..3 {
            hash_insert(&mut table, 3, slot).unwrap();
        }
        let (head, _) = hash_lookup(&table, 3, |slot| slot == 0).unwrap();
        hash_backshift_delete(&mut table, head, |_| Ok(3)).unwrap();
        assert_eq!(
            hash_lookup(&table, 3, |slot| slot == 2).map(|pair| pair.1),
            Some(2)
        );
        let (middle, _) = hash_lookup(&table, 3, |slot| slot == 1).unwrap();
        hash_backshift_delete(&mut table, middle, |_| Ok(3)).unwrap();
        hash_insert(&mut table, 3, 3).unwrap();
        assert_eq!(
            hash_lookup(&table, 3, |slot| slot == 3).map(|pair| pair.1),
            Some(3)
        );
        assert_eq!(
            hash_lookup(&table, 3, |slot| slot == 2).map(|pair| pair.1),
            Some(2)
        );
        assert!(table
            .iter()
            .all(|entry| entry.slot_plus_one != HASH_TOMBSTONE));
    }

    #[test]
    fn backshift_prevalidates_late_corruption_and_wraparound_mixed_hashes() {
        let mut corrupt = vec![HashEntry::default(); 8];
        for slot in 0..3 {
            hash_insert(&mut corrupt, 6, slot).unwrap();
        }
        corrupt[0].slot_plus_one = 99;
        let before = corrupt.clone();
        assert_eq!(
            hash_backshift_delete(&mut corrupt, 6, |slot| {
                if slot < 3 {
                    Ok(6)
                } else {
                    Err(RegistryError::CorruptSharedState)
                }
            }),
            Err(RegistryError::CorruptSharedState)
        );
        assert_eq!(corrupt, before);

        let mut wrapped = vec![HashEntry::default(); 8];
        hash_insert(&mut wrapped, 7, 0).unwrap();
        hash_insert(&mut wrapped, 7, 1).unwrap();
        hash_insert(&mut wrapped, 0, 2).unwrap();
        let (head, _) = hash_lookup(&wrapped, 7, |slot| slot == 0).unwrap();
        hash_backshift_delete(&mut wrapped, head, |slot| match slot {
            1 => Ok(7),
            2 => Ok(0),
            _ => Err(RegistryError::CorruptSharedState),
        })
        .unwrap();
        assert_eq!(
            hash_lookup(&wrapped, 7, |slot| slot == 1).map(|p| p.1),
            Some(1)
        );
        assert_eq!(
            hash_lookup(&wrapped, 0, |slot| slot == 2).map(|p| p.1),
            Some(2)
        );
    }

    #[test]
    fn counter_underflow_fails_without_releasing_token_or_operation() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 1, 1);
        let token = identity(1, 1, 7, 1);
        registry
            .view()
            .register_with(token, live, |_| true)
            .unwrap();
        let fence = registry
            .view()
            .find_fence_index(1.into(), &[1; 16])
            .unwrap();
        registry.fences[fence].token_count = 0;
        assert_eq!(
            registry.view().release(token, live),
            Err(RegistryError::CorruptSharedState)
        );
        assert_eq!(
            registry
                .tokens
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
        registry.fences[fence].token_count = 1;
        let fence_id = registry
            .view()
            .acquire_fence_operation(fence, live)
            .unwrap();
        registry.fences[fence].operation_references = 0;
        assert_eq!(
            registry.view().release_fence_operation(live, fence_id),
            Err(RegistryError::CorruptSharedState)
        );
        assert_eq!(
            registry
                .operations
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
    }

    #[test]
    fn recycle_hash_corruption_fails_before_release_mutation() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 1, 1);
        let token = identity(1, 1, 7, 1);
        registry
            .view()
            .register_with(token, live, |_| true)
            .unwrap();
        let fence = registry
            .view()
            .find_fence_index(1.into(), &[1; 16])
            .unwrap();
        registry.fences[fence].dropped = 1;
        let bucket = registry.view().fence_hash_bucket(fence).unwrap();
        registry.fence_hash[bucket].slot_plus_one = HASH_TOMBSTONE;
        assert_eq!(
            registry.view().release(token, live),
            Err(RegistryError::CorruptSharedState)
        );
        assert_eq!(registry.fences[fence].token_count, 1);
        assert_eq!(
            registry
                .tokens
                .iter()
                .filter(|slot| slot.in_use != 0)
                .count(),
            1
        );
    }

    #[test]
    fn free_lists_survive_repeated_exhaust_and_repopulation() {
        let live = owner(0, 100, 1);
        let mut registry = TestRegistry::new(1, 1, 2);
        for round in 0..64_u8 {
            let first = identity(1, 1, 7, round.wrapping_mul(2));
            let second = identity(1, 1, 7, round.wrapping_mul(2).wrapping_add(1));
            registry
                .view()
                .register_with(first, live, |_| true)
                .unwrap();
            registry
                .view()
                .register_with(second, live, |_| true)
                .unwrap();
            assert_eq!(registry.header.token_free_head, -1);
            assert!(registry.view().release(first, live).unwrap());
            assert!(registry.view().release(second, live).unwrap());
            assert_eq!(
                registry
                    .tokens
                    .iter()
                    .filter(|slot| slot.in_use != 0)
                    .count(),
                0
            );
        }
        assert!(registry.header.token_free_head >= 0);
        assert_eq!(registry.fences[0].token_count, 0);
        assert!(registry
            .token_hash
            .iter()
            .all(|entry| entry.slot_plus_one != HASH_TOMBSTONE));

        let mut operations = TestRegistry::new(1, 4, 0);
        for _ in 0..16 {
            let mut fence_ids = Vec::new();
            for logical in 0..4_u8 {
                let fence = operations
                    .view()
                    .ensure_fence(1.into(), [logical; 16])
                    .unwrap();
                fence_ids.push(
                    operations
                        .view()
                        .acquire_fence_operation(fence, live)
                        .unwrap(),
                );
            }
            assert_eq!(operations.header.operation_free_head, -1);
            for fence_id in fence_ids {
                operations
                    .view()
                    .release_fence_operation(live, fence_id)
                    .unwrap();
            }
            assert!(operations.header.operation_free_head >= 0);
            assert!(operations.operations.iter().all(|slot| slot.in_use == 0));
            assert!(operations
                .operation_hash
                .iter()
                .all(|entry| entry.slot_plus_one != HASH_TOMBSTONE));
        }
    }
}
