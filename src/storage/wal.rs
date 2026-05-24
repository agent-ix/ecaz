//! WAL helpers shared across access methods.
//!
//! ECAZ currently writes PostgreSQL GenericXLog records only. Those records
//! carry page images/deltas rather than an extension-owned record body, so the
//! versioned byte contract lives in the page payloads themselves.
//!
//! P3 typed wrappers (Task 54):
//!
//! - [`WalTxnScope`] is the safe-surface name for [`GenericXLogTxn`]; it
//!   encodes the relation-liveness invariant in the type and exposes
//!   safe `register_page` / `finish` operations. Constructors remain
//!   `unsafe fn` because PostgreSQL's `Relation` is an opaque pointer.
//! - [`RegisteredBufferPage`] gains safe `init` and `add_item` operations
//!   so consumers no longer need a per-call `unsafe { pg_sys::PageInit }`
//!   / `unsafe { pg_sys::PageAddItemExtended }` block.

use std::marker::PhantomData;

use pgrx::pg_sys;

use crate::storage::buffer_guard::LockedBufferGuard;
use crate::storage::page::ItemPointer;
use crate::storage::relation::RelationHandle;

/// Current format tag for future extension-owned ECAZ WAL payloads.
pub const ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION: u8 = 1;

/// Byte offset reserved for the format tag in future extension-owned WAL records.
pub const ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION_OFFSET: usize = 0;

/// ECAZ does not currently emit extension-owned WAL record payloads.
///
/// Keep this false until Task 37 adds custom redo/replay payloads. When that
/// happens, every custom record must carry
/// [`ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION`] at byte 0 and replay must call
/// [`validate_custom_wal_record_format_version`] before reading the body.
pub const ECAZ_CUSTOM_WAL_RECORDS_ENABLED: bool = false;

/// Validate the leading format tag for a future extension-owned WAL record.
pub fn validate_custom_wal_record_format_version(record: &[u8]) -> Result<(), &'static str> {
    let Some(version) = record.get(ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION_OFFSET) else {
        return Err("missing ECAZ custom WAL record format version");
    };
    if *version != ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION {
        return Err("unknown ECAZ custom WAL record format version");
    }
    Ok(())
}

/// RAII wrapper around PostgreSQL's GenericXLog state.
///
/// If the transaction is dropped without calling [`finish`](Self::finish),
/// PostgreSQL aborts the pending generic WAL record.
pub struct GenericXLogTxn {
    state: *mut pg_sys::GenericXLogState,
    finished: bool,
}

pub(crate) struct RegisteredBufferPage<'txn, 'buffer> {
    page: pg_sys::Page,
    buffer: &'buffer LockedBufferGuard,
    _txn: PhantomData<&'txn mut GenericXLogTxn>,
}

impl GenericXLogTxn {
    /// Start a new GenericXLog transaction for the given relation.
    ///
    /// # Safety
    ///
    /// `relation` must be a valid PostgreSQL relation pointer for the duration
    /// of the transaction.
    pub unsafe fn start(relation: pg_sys::Relation) -> Self {
        // SAFETY: The caller guarantees `relation` is a live PostgreSQL relation.
        let state = unsafe { pg_sys::GenericXLogStart(relation) };
        assert!(!state.is_null(), "GenericXLogStart returned null");
        Self {
            state,
            finished: false,
        }
    }

    /// Register a locked buffer as a full-page image and return the writable page.
    pub fn register_locked_buffer_full_image(
        &mut self,
        buffer: &LockedBufferGuard,
    ) -> pg_sys::Page {
        self.register_locked_buffer_full_image_raw(buffer)
    }

    pub(crate) fn register_locked_buffer_full_image_page<'txn, 'buffer>(
        &'txn mut self,
        buffer: &'buffer LockedBufferGuard,
    ) -> RegisteredBufferPage<'txn, 'buffer> {
        let page = self.register_locked_buffer_full_image_raw(buffer);
        RegisteredBufferPage {
            page,
            buffer,
            _txn: PhantomData,
        }
    }

    fn register_locked_buffer_full_image_raw(
        &mut self,
        buffer: &LockedBufferGuard,
    ) -> pg_sys::Page {
        assert!(
            !self.finished,
            "cannot register buffer after GenericXLogFinish"
        );
        // SAFETY: `LockedBufferGuard` owns a valid locked buffer. ECAZ's current
        // GenericXLog usage registers full-page images only.
        let page = unsafe {
            pg_sys::GenericXLogRegisterBuffer(
                self.state,
                buffer.buffer(),
                pg_sys::GENERIC_XLOG_FULL_IMAGE as i32,
            )
        };
        assert!(!page.is_null(), "GenericXLogRegisterBuffer returned null");
        page
    }

    /// Finish the GenericXLog transaction and return the written WAL pointer.
    ///
    /// `GenericXLogTxn` owns the PostgreSQL WAL state and prevents double
    /// finish through `self` ownership. Callers are still responsible for
    /// writing final page contents before calling this method.
    pub fn finish(mut self) -> pg_sys::XLogRecPtr {
        self.finished = true;
        // SAFETY: `self.state` came from `GenericXLogStart` and has not yet been finished/aborted.
        unsafe { pg_sys::GenericXLogFinish(self.state) }
    }
}

impl RegisteredBufferPage<'_, '_> {
    pub(crate) fn visit_tuple_bytes_mut<R, F>(
        &mut self,
        tid: ItemPointer,
        tuple_kind: &str,
        visit: F,
    ) -> Result<R, String>
    where
        F: FnOnce(&mut [u8]) -> Result<R, String>,
    {
        let page_size = self.buffer.page_size();

        // SAFETY: this page was returned by GenericXLogRegisterBuffer for the
        // locked buffer borrowed by this token. The TID offset, line pointer,
        // tuple bounds, and tuple pointer are checked before exposing mutable
        // bytes for the duration of the visitor only.
        unsafe {
            let max_offset = pg_sys::PageGetMaxOffsetNumber(self.page);
            if tid.offset_number == pg_sys::InvalidOffsetNumber || tid.offset_number > max_offset {
                return Err(format!(
                    "{tuple_kind} tuple ({},{}) has invalid offset {} (max {})",
                    tid.block_number, tid.offset_number, tid.offset_number, max_offset
                ));
            }

            let item_id = pg_sys::PageGetItemId(self.page, tid.offset_number);
            if item_id.is_null() {
                return Err(format!(
                    "{tuple_kind} tuple ({},{}) returned a null item id",
                    tid.block_number, tid.offset_number
                ));
            }
            let item_id_ref = &*item_id;
            if item_id_ref.lp_flags() == 0 {
                return Err(format!(
                    "{tuple_kind} tuple ({},{}) points at an unused slot",
                    tid.block_number, tid.offset_number
                ));
            }

            let tuple_offset = item_id_ref.lp_off() as usize;
            let tuple_len = item_id_ref.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                return Err(format!(
                    "{tuple_kind} tuple ({},{}) has invalid bounds",
                    tid.block_number, tid.offset_number
                ));
            }

            let tuple_ptr = pg_sys::PageGetItem(self.page, item_id).cast::<u8>();
            if tuple_ptr.is_null() {
                return Err(format!(
                    "{tuple_kind} tuple ({},{}) returned a null tuple pointer",
                    tid.block_number, tid.offset_number
                ));
            }
            let tuple = std::slice::from_raw_parts_mut(tuple_ptr, tuple_len);
            visit(tuple)
        }
    }
}

impl Drop for GenericXLogTxn {
    fn drop(&mut self) {
        if !self.finished && !self.state.is_null() {
            // SAFETY: Aborting an unfinished GenericXLog state is PostgreSQL's
            // required cleanup path.
            unsafe { pg_sys::GenericXLogAbort(self.state) };
        }
    }
}

#[allow(dead_code)] // Consumed by HNSW migration in Task 54 packets 003/004.
impl RegisteredBufferPage<'_, '_> {
    pub(crate) fn page_ptr(&self) -> pg_sys::Page {
        self.page
    }

    /// Initialize this WAL-registered page (PageInit) with the given special
    /// area size in bytes. Use `0` for a normal page with no special area.
    pub(crate) fn init(&mut self, special_size: usize) {
        // SAFETY: `self.page` is the WAL-registered writable image obtained
        // from `GenericXLogRegisterBuffer` for the borrowed `LockedBufferGuard`.
        // `page_size` is the size of the underlying locked buffer's page.
        unsafe { pg_sys::PageInit(self.page, self.buffer.page_size(), special_size) };
    }

    /// Append `payload` to this WAL-registered page via `PageAddItemExtended`.
    ///
    /// Returns the new offset number on success, or `PageAddItemError` (with
    /// the buffer's block number for caller-side diagnostics) when PostgreSQL
    /// returns `InvalidOffsetNumber`.
    pub(crate) fn add_item(
        &mut self,
        payload: &[u8],
    ) -> Result<pg_sys::OffsetNumber, PageAddItemError> {
        // SAFETY: `self.page` is the WAL-registered writable image. PostgreSQL
        // copies `payload` bytes during `PageAddItemExtended`, so the slice
        // needs only to remain live for the duration of this call.
        let offset = unsafe {
            pg_sys::PageAddItemExtended(
                self.page,
                payload.as_ptr().cast_mut().cast(),
                payload.len(),
                pg_sys::InvalidOffsetNumber,
                0,
            )
        };
        if offset == pg_sys::InvalidOffsetNumber {
            Err(PageAddItemError {
                block_number: self.buffer.block_number(),
            })
        } else {
            Ok(offset)
        }
    }
}

/// Error returned by [`RegisteredBufferPage::add_item`] when PostgreSQL's
/// `PageAddItemExtended` rejects the tuple (returns `InvalidOffsetNumber`).
///
/// Carries the target block number for caller-side error messages.
#[allow(dead_code)] // Consumed by HNSW migration in Task 54 packets 003/004.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PageAddItemError {
    pub(crate) block_number: pg_sys::BlockNumber,
}

/// Typed RAII scope around a PostgreSQL `GenericXLog` transaction.
///
/// `WalTxnScope` is the safe-surface name for [`GenericXLogTxn`]; it wraps
/// the same underlying state and exposes safe `register_page` / `finish`
/// operations so consumers never need a per-call `unsafe { GenericXLogStart }`
/// or `unsafe { GenericXLogFinish }` block.
///
/// The `'rel` lifetime marker carries the relation-liveness invariant from the
/// `unsafe fn start` constructor through to every method on the scope.
#[allow(dead_code)] // Consumed by HNSW migration in Task 54 packets 003/004.
pub(crate) struct WalTxnScope<'rel> {
    txn: GenericXLogTxn,
    _rel: PhantomData<&'rel ()>,
}

#[allow(dead_code)] // Consumed by HNSW migration in Task 54 packets 003/004.
impl<'rel> WalTxnScope<'rel> {
    /// Start a new GenericXLog transaction for `relation`.
    ///
    /// # Safety
    ///
    /// `relation` must be a live PostgreSQL relation pointer; the `'rel`
    /// lifetime must outlive every safe operation on the returned scope.
    pub(crate) unsafe fn start(relation: pg_sys::Relation) -> Self {
        // SAFETY: caller guarantees `relation` is a live relation pointer.
        Self {
            txn: unsafe { GenericXLogTxn::start(relation) },
            _rel: PhantomData,
        }
    }

    /// Start a new GenericXLog transaction for the relation behind `handle`.
    ///
    /// Safe variant of [`start`](Self::start) — callers that already hold a
    /// validated [`RelationHandle`] pass it through this entry point so the
    /// WAL scope inherits the handle's "live relation" SAFETY contract
    /// instead of re-establishing it at every call site.
    pub(crate) fn start_handle(handle: RelationHandle) -> Self {
        // SAFETY: `RelationHandle` is a non-null pointer whose construction
        // contract requires a live opened PostgreSQL relation; that contract
        // satisfies `GenericXLogStart`'s precondition.
        unsafe { Self::start(handle.as_ptr()) }
    }

    /// Register a locked buffer as a full-page image and return the writable
    /// page wrapper. The returned [`RegisteredBufferPage`] exposes safe
    /// `init` / `add_item` operations.
    pub(crate) fn register_page<'txn, 'buffer>(
        &'txn mut self,
        buffer: &'buffer LockedBufferGuard,
    ) -> RegisteredBufferPage<'txn, 'buffer> {
        self.txn.register_locked_buffer_full_image_page(buffer)
    }

    /// Finish the underlying GenericXLog transaction and return the WAL
    /// pointer.
    pub(crate) fn finish(self) -> pg_sys::XLogRecPtr {
        self.txn.finish()
    }
}
