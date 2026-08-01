//! Stubs for PostgreSQL backend `static mut` globals that the CLI's static-link
//! graph references via `ecaz::bench_api` → `crate::am::*` → `pgrx::pg_sys::*`.
//!
//! The CLI never invokes any of these code paths (it talks to PG over libpq).
//! macOS 26's chained-fixups loader eagerly binds flat-namespace `static mut`
//! symbols at dyld load time, so undefined references fail before `main`.
//! Null definitions satisfy the linker and let the binary start; the symbols
//! would only be touched if the CLI invoked PG-backend code, which it does not.
//!
//! Linux uses `-Wl,--unresolved-symbols=ignore-all` (see `.cargo/config.toml`)
//! and doesn't need these stubs.

#![cfg(target_os = "macos")]
#![allow(non_upper_case_globals)]

use std::ffi::{c_char, c_void};

type MemoryContext = *mut c_void;

// PG18 Oid (`postgres_ext.h`) and process globals (`miscadmin.h`).
#[no_mangle]
pub static mut MyDatabaseId: u32 = 0;

#[no_mangle]
pub static mut MyProcPid: i32 = 0;

// PG18 ProcNumber is an `int` (`storage/procnumber.h`).
#[no_mangle]
pub static mut MyProcNumber: i32 = -1;

// PG18 `PROC_HDR *` (`storage/proc.h`).
#[no_mangle]
pub static mut ProcGlobal: *mut c_void = std::ptr::null_mut();

// PG18 volatile sig_atomic_t interrupt flags (`miscadmin.h`). The CLI never
// reads them; definitions are present only so dyld can bind the unreachable
// extension graph retained by release LTO.
#[no_mangle]
pub static mut InterruptPending: i32 = 0;

#[no_mangle]
pub static mut QueryCancelPending: i32 = 0;

#[no_mangle]
pub static mut ProcDiePending: i32 = 0;

// PG18 transaction isolation enum storage is an `int` (`access/xact.h`).
#[no_mangle]
pub static mut XactIsoLevel: i32 = 0;

#[no_mangle]
pub static mut BufferBlocks: *mut c_char = std::ptr::null_mut();

#[no_mangle]
pub static mut PG_exception_stack: *mut c_void = std::ptr::null_mut();

#[no_mangle]
pub static mut error_context_stack: *mut c_void = std::ptr::null_mut();

#[no_mangle]
pub static mut CurrentMemoryContext: MemoryContext = std::ptr::null_mut();

#[no_mangle]
pub static mut CacheMemoryContext: MemoryContext = std::ptr::null_mut();

#[no_mangle]
pub static mut TopMemoryContext: MemoryContext = std::ptr::null_mut();

#[no_mangle]
pub static mut MessageContext: MemoryContext = std::ptr::null_mut();

#[no_mangle]
pub static mut PortalContext: MemoryContext = std::ptr::null_mut();

#[no_mangle]
pub static mut ErrorContext: MemoryContext = std::ptr::null_mut();

#[no_mangle]
pub static mut TopTransactionContext: MemoryContext = std::ptr::null_mut();

#[no_mangle]
pub static mut CurTransactionContext: MemoryContext = std::ptr::null_mut();

#[no_mangle]
pub static mut PostmasterContext: MemoryContext = std::ptr::null_mut();

// TransactionId is a u32 (xid)
#[no_mangle]
pub static mut CheckXidAlive: u32 = 0;

#[no_mangle]
pub static mut bsysscan: bool = false;

// `*mut Block` (Block = *mut c_void in PG's bufmgr.h)
#[no_mangle]
pub static mut LocalBufferBlockPointers: *mut *mut c_void = std::ptr::null_mut();

// uint64
#[no_mangle]
pub static mut SPI_processed: u64 = 0;

// `*mut SPITupleTable` (opaque)
#[no_mangle]
pub static mut SPI_tuptable: *mut c_void = std::ptr::null_mut();
