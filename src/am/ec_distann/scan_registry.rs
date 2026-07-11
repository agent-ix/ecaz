//! FR-082 coordinator-local exact scan-token registry foundation.
//!
//! The registry lives in add-in shared memory and is intentionally independent
//! of CustomScan, participant transport, publish decisions, and retirement.
//! This module owns only exact token identity/liveness and bounded logical-index
//! fence allocation.  Later slices acquire the exported fence operation
//! reference before waiting for or holding the matching heavyweight fence.

use std::cell::Cell;
use std::ffi::c_int;
use std::mem::{align_of, size_of};
use std::ptr;

use pgrx::datum::Uuid;
use pgrx::{pg_sys, GucContext, GucFlags, GucRegistry, GucSetting};

const REGISTRY_MAGIC: u64 = 0x4543_4453_5250_3031; // "ECDSRP01"
const REGISTRY_VERSION: u32 = 1;
const DEFAULT_MAX_SCAN_PINS: i32 = 65_536;
const DEFAULT_MAX_RETIRE_FENCES: i32 = 4_096;
const MAX_CONFIGURED_SCAN_PINS: i32 = 1_048_576;
const MAX_CONFIGURED_RETIRE_FENCES: i32 = 65_536;
static MAX_SCAN_PINS_GUC: GucSetting<i32> = GucSetting::<i32>::new(DEFAULT_MAX_SCAN_PINS);
static MAX_RETIRE_FENCES_GUC: GucSetting<i32> = GucSetting::<i32>::new(DEFAULT_MAX_RETIRE_FENCES);

static mut PREV_SHMEM_REQUEST_HOOK: pg_sys::shmem_request_hook_type = None;
static mut PREV_SHMEM_STARTUP_HOOK: pg_sys::shmem_startup_hook_type = None;
static mut REGISTRY_HEADER: *mut RegistryHeader = ptr::null_mut();
static mut REGISTRY_LWLOCK: *mut pg_sys::LWLock = ptr::null_mut();

thread_local! {
    static BACKEND_OWNER: Cell<Option<BackendOwner>> = const { Cell::new(None) };
    static EXIT_CALLBACK_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RegistryHeader {
    magic: u64,
    version: u32,
    total_bytes: usize,
    generation_capacity: u32,
    fence_capacity: u32,
    token_capacity: u32,
    generation_offset: usize,
    operation_offset: usize,
    fence_offset: usize,
    token_offset: usize,
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
    _tail_padding: [u8; 4],
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
        }
    }
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
    fence_capacity: usize,
    token_capacity: usize,
    generation_offset: usize,
    operation_offset: usize,
    fence_offset: usize,
    token_offset: usize,
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
        // One slot per PGPROC is sufficient: a PostgreSQL backend is
        // single-threaded and can wait on only one heavyweight fence at a
        // time. Re-entrant references to that same fence increment `nesting`.
        let (operation_offset, after_operations) = checked_region(
            after_generations,
            generation_capacity,
            size_of::<FenceOperationSlot>(),
            align_of::<FenceOperationSlot>(),
        )?;
        let (fence_offset, after_fences) = checked_region(
            after_operations,
            fence_capacity,
            size_of::<FenceSlot>(),
            align_of::<FenceSlot>(),
        )?;
        let (token_offset, total_bytes) = checked_region(
            after_fences,
            token_capacity,
            size_of::<TokenSlot>(),
            align_of::<TokenSlot>(),
        )?;
        Some(Self {
            total_bytes,
            generation_capacity,
            fence_capacity,
            token_capacity,
            generation_offset,
            operation_offset,
            fence_offset,
            token_offset,
        })
    }
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
                fence_capacity: layout.fence_capacity as u32,
                token_capacity: layout.token_capacity as u32,
                generation_offset: layout.generation_offset,
                operation_offset: layout.operation_offset,
                fence_offset: layout.fence_offset,
                token_offset: layout.token_offset,
            });
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
    generations: &'a mut [BackendGeneration],
    operations: &'a mut [FenceOperationSlot],
    fences: &'a mut [FenceSlot],
    tokens: &'a mut [TokenSlot],
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
        || header.operation_offset != expected.operation_offset
        || header.fence_offset != expected.fence_offset
        || header.token_offset != expected.token_offset
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
    let operations = unsafe {
        std::slice::from_raw_parts_mut(
            base.add(header.operation_offset)
                .cast::<FenceOperationSlot>(),
            header.generation_capacity as usize,
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
    Ok(RegistryView {
        generations,
        operations,
        fences,
        tokens,
    })
}

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
    view.release_owner(owner);
    view.recycle_dropped_fences();
}

impl RegistryView<'_> {
    fn reap_dead_with(&mut self, mut live: impl FnMut(BackendOwner) -> bool) -> usize {
        let mut reaped = 0;
        for token in &mut *self.tokens {
            if token.in_use == 0 {
                continue;
            }
            let owner = BackendOwner {
                proc_number: token.proc_number,
                pid: token.backend_pid,
                generation: token.backend_generation,
            };
            if !live(owner) {
                *token = TokenSlot::default();
                reaped += 1;
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
                self.decrement_fence_reference(operation.fence_id, operation.nesting);
                self.operations[index] = FenceOperationSlot::default();
                reaped += 1;
            }
        }
        reaped
    }

    fn reap_dead(&mut self) -> usize {
        let generations = self.generations.as_ptr();
        let generation_len = self.generations.len();
        self.reap_dead_with(|owner| {
            let generations = unsafe { std::slice::from_raw_parts(generations, generation_len) };
            backend_is_live(generations, owner)
        })
    }

    fn release_owner(&mut self, owner: BackendOwner) -> usize {
        let mut released = 0;
        for token in &mut *self.tokens {
            if token.in_use != 0
                && token.proc_number == owner.proc_number
                && token.backend_pid == owner.pid
                && token.backend_generation == owner.generation
            {
                *token = TokenSlot::default();
                released += 1;
            }
        }
        for index in 0..self.operations.len() {
            let operation = self.operations[index];
            if operation.in_use != 0
                && operation.proc_number == owner.proc_number
                && operation.backend_pid == owner.pid
                && operation.backend_generation == owner.generation
            {
                self.decrement_fence_reference(operation.fence_id, operation.nesting);
                self.operations[index] = FenceOperationSlot::default();
                released += 1;
            }
        }
        released
    }

    fn find_fence_index(&self, database_oid: pg_sys::Oid, logical: &[u8; 16]) -> Option<usize> {
        self.fences.iter().position(|fence| {
            fence.in_use != 0
                && fence.database_oid == database_oid
                && &fence.logical_index_uuid == logical
        })
    }

    fn allocate_fence_id(&self) -> Option<u32> {
        (1..=self.fences.len() as u32).find(|candidate| {
            !self
                .fences
                .iter()
                .any(|slot| slot.in_use != 0 && slot.fence_id == *candidate)
        })
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
        self.recycle_dropped_fences();
        let index = self
            .fences
            .iter()
            .position(|fence| fence.in_use == 0)
            .ok_or(RegistryError::FenceCapacity)?;
        let fence_id = self
            .allocate_fence_id()
            .ok_or(RegistryError::FenceCapacity)?;
        self.fences[index] = FenceSlot {
            in_use: 1,
            dropped: 0,
            _padding: [0; 2],
            database_oid,
            logical_index_uuid: logical,
            fence_id,
            operation_references: 0,
        };
        Ok(index)
    }

    fn has_token_for_fence(&self, database_oid: pg_sys::Oid, logical: &[u8; 16]) -> bool {
        self.tokens.iter().any(|token| {
            token.in_use != 0
                && token.database_oid == database_oid
                && &token.logical_index_uuid == logical
        })
    }

    fn decrement_fence_reference(&mut self, fence_id: u32, count: u32) {
        if let Some(fence) = self
            .fences
            .iter_mut()
            .find(|fence| fence.in_use != 0 && fence.fence_id == fence_id)
        {
            fence.operation_references = fence.operation_references.saturating_sub(count);
        }
    }

    fn acquire_fence_operation(
        &mut self,
        fence_index: usize,
        owner: BackendOwner,
    ) -> Result<u32, RegistryError> {
        let proc_index =
            usize::try_from(owner.proc_number).map_err(|_| RegistryError::CorruptSharedState)?;
        let operation = self
            .operations
            .get_mut(proc_index)
            .ok_or(RegistryError::CorruptSharedState)?;
        let fence_id = self.fences[fence_index].fence_id;
        if operation.in_use != 0 {
            if operation.fence_id != fence_id
                || operation.proc_number != owner.proc_number
                || operation.backend_pid != owner.pid
                || operation.backend_generation != owner.generation
            {
                return Err(RegistryError::CorruptSharedState);
            }
            operation.nesting = operation
                .nesting
                .checked_add(1)
                .ok_or(RegistryError::CorruptSharedState)?;
        } else {
            *operation = FenceOperationSlot {
                in_use: 1,
                _padding: [0; 3],
                fence_id,
                proc_number: owner.proc_number,
                backend_pid: owner.pid,
                backend_generation: owner.generation,
                nesting: 1,
                _tail_padding: [0; 4],
            };
        }
        self.fences[fence_index].operation_references = self.fences[fence_index]
            .operation_references
            .checked_add(1)
            .ok_or(RegistryError::CorruptSharedState)?;
        Ok(fence_id)
    }

    fn release_fence_operation(
        &mut self,
        owner: BackendOwner,
        fence_id: u32,
    ) -> Result<(), RegistryError> {
        let proc_index =
            usize::try_from(owner.proc_number).map_err(|_| RegistryError::CorruptSharedState)?;
        let operation = self
            .operations
            .get_mut(proc_index)
            .ok_or(RegistryError::CorruptSharedState)?;
        if operation.in_use == 0
            || operation.fence_id != fence_id
            || operation.proc_number != owner.proc_number
            || operation.backend_pid != owner.pid
            || operation.backend_generation != owner.generation
            || operation.nesting == 0
        {
            return Err(RegistryError::CorruptSharedState);
        }
        operation.nesting -= 1;
        if operation.nesting == 0 {
            *operation = FenceOperationSlot::default();
        }
        self.decrement_fence_reference(fence_id, 1);
        self.recycle_dropped_fences();
        Ok(())
    }

    fn recycle_dropped_fences(&mut self) -> usize {
        let mut recycled = 0;
        for index in 0..self.fences.len() {
            let fence = self.fences[index];
            if fence.in_use != 0
                && fence.dropped != 0
                && fence.operation_references == 0
                && !self.has_token_for_fence(fence.database_oid, &fence.logical_index_uuid)
            {
                self.fences[index] = FenceSlot::default();
                recycled += 1;
            }
        }
        recycled
    }

    fn register_with(
        &mut self,
        identity: ScanTokenIdentity,
        owner: BackendOwner,
        live: impl FnMut(BackendOwner) -> bool,
    ) -> Result<RegisterOutcome, RegistryError> {
        self.reap_dead_with(live);
        let fence_index = self.ensure_fence(identity.database_oid, identity.logical_index_uuid)?;
        let fence_id = self.acquire_fence_operation(fence_index, owner)?;

        let result = if let Some(existing) = self.tokens.iter().find(|token| {
            token.in_use != 0
                && token.database_oid == identity.database_oid
                && token.logical_index_uuid == identity.logical_index_uuid
                && token.scan_token == identity.scan_token
        }) {
            if existing.epoch_fingerprint == identity.epoch_fingerprint {
                Ok(RegisterOutcome::AlreadyRegistered)
            } else {
                Err(RegistryError::TokenConflict)
            }
        } else if let Some(slot) = self.tokens.iter_mut().find(|token| token.in_use == 0) {
            *slot = TokenSlot {
                in_use: 1,
                _padding: [0; 3],
                database_oid: identity.database_oid,
                logical_index_uuid: identity.logical_index_uuid,
                epoch_fingerprint: identity.epoch_fingerprint,
                scan_token: identity.scan_token,
                proc_number: owner.proc_number,
                backend_pid: owner.pid,
                backend_generation: owner.generation,
            };
            Ok(RegisterOutcome::Registered)
        } else {
            Err(RegistryError::TokenCapacity)
        };

        let release = self.release_fence_operation(owner, fence_id);
        match (result, release) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Ok(outcome), Ok(())) => Ok(outcome),
        }
    }

    fn register(
        &mut self,
        identity: ScanTokenIdentity,
        owner: BackendOwner,
    ) -> Result<RegisterOutcome, RegistryError> {
        let generations = self.generations.as_ptr();
        let generation_len = self.generations.len();
        self.register_with(identity, owner, |candidate| {
            let generations = unsafe { std::slice::from_raw_parts(generations, generation_len) };
            backend_is_live(generations, candidate)
        })
    }

    fn release(&mut self, identity: ScanTokenIdentity) -> bool {
        if let Some(slot) = self.tokens.iter_mut().find(|token| {
            token.in_use != 0
                && token.database_oid == identity.database_oid
                && token.logical_index_uuid == identity.logical_index_uuid
                && token.epoch_fingerprint == identity.epoch_fingerprint
                && token.scan_token == identity.scan_token
        }) {
            *slot = TokenSlot::default();
            self.recycle_dropped_fences();
            true
        } else {
            false
        }
    }

    fn mark_dropped(&mut self, database_oid: pg_sys::Oid, logical: [u8; 16]) -> bool {
        let Some(index) = self.find_fence_index(database_oid, &logical) else {
            return false;
        };
        self.fences[index].dropped = 1;
        self.recycle_dropped_fences();
        true
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
    let identity = ScanTokenIdentity {
        database_oid: unsafe { pg_sys::MyDatabaseId },
        logical_index_uuid: *logical_index_uuid.as_bytes(),
        epoch_fingerprint,
        scan_token: *scan_token.as_bytes(),
    };
    let _guard = unsafe { lock_registry()? };
    Ok(unsafe { shared_view()? }.release(identity))
}

pub(crate) fn mark_logical_index_dropped(
    database_oid: pg_sys::Oid,
    logical_index_uuid: [u8; 16],
) -> Result<bool, RegistryError> {
    let _guard = unsafe { lock_registry()? };
    Ok(unsafe { shared_view()? }.mark_dropped(database_oid, logical_index_uuid))
}

/// Pins a fence-map identity while a later slice waits for or holds its
/// heavyweight fence.  The exact owner incarnation is shared-state-visible so
/// abrupt backend death can reap the operation reference before recycling a
/// dropped UUID's fence id.
pub(crate) struct FenceOperationReference {
    owner: BackendOwner,
    fence_id: u32,
    active: bool,
}

impl FenceOperationReference {
    pub(crate) fn fence_id(&self) -> u32 {
        self.fence_id
    }
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

pub(crate) fn acquire_fence_operation_reference(
    database_oid: pg_sys::Oid,
    logical_index_uuid: [u8; 16],
) -> Result<FenceOperationReference, RegistryError> {
    let owner = current_backend_owner()?;
    let _guard = unsafe { lock_registry()? };
    let mut view = unsafe { shared_view()? };
    view.reap_dead();
    let fence_index = view.ensure_fence(database_oid, logical_index_uuid)?;
    let fence_id = view.acquire_fence_operation(fence_index, owner)?;
    Ok(FenceOperationReference {
        owner,
        fence_id,
        active: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRegistry {
        generations: Vec<BackendGeneration>,
        operations: Vec<FenceOperationSlot>,
        fences: Vec<FenceSlot>,
        tokens: Vec<TokenSlot>,
    }

    impl TestRegistry {
        fn new(generations: usize, fences: usize, tokens: usize) -> Self {
            Self {
                generations: vec![BackendGeneration::default(); generations],
                operations: vec![FenceOperationSlot::default(); generations],
                fences: vec![FenceSlot::default(); fences],
                tokens: vec![TokenSlot::default(); tokens],
            }
        }

        fn view(&mut self) -> RegistryView<'_> {
            RegistryView {
                generations: &mut self.generations,
                operations: &mut self.operations,
                fences: &mut self.fences,
                tokens: &mut self.tokens,
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
        assert!(layout.generation_offset >= size_of::<RegistryHeader>());
        assert!(layout.fence_offset > layout.generation_offset);
        assert!(layout.operation_offset > layout.generation_offset);
        assert!(layout.fence_offset > layout.operation_offset);
        assert!(layout.token_offset > layout.fence_offset);
        assert!(layout.total_bytes > layout.token_offset);
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
        assert_eq!(one.operations[0], FenceOperationSlot::default());
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
        assert_eq!(registry.tokens[0].epoch_fingerprint, [7; 34]);
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
        assert_eq!(registry.view().release_owner(first), 1);
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
        let mut registry = TestRegistry::new(1, 1, 1);
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
        assert!(registry.view().mark_dropped(1.into(), [1; 16]));
        assert_eq!(registry.fences[0].dropped, 1);
        assert!(registry.view().release(first));
        assert_eq!(registry.fences[0], FenceSlot::default());
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
        assert!(registry.view().mark_dropped(1.into(), [1; 16]));
        assert_eq!(registry.view().recycle_dropped_fences(), 0);
        registry
            .view()
            .release_fence_operation(live, fence_id)
            .unwrap();
        assert_eq!(registry.fences[index], FenceSlot::default());
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
            assert!(registry
                .view()
                .mark_dropped(pg_sys::Oid::from(1), [generation; 16]));
            assert!(registry.view().release(transient));
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
            1
        );
        assert_eq!(registry.view().recycle_dropped_fences(), 1);
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
}
