//! PostgreSQL buffer ownership guards.
//!
//! Use `PinnedBufferGuard` when the caller owns only a buffer pin:
//! `read_main` wraps `ReadBufferExtended`, and `from_pinned` adopts buffers
//! returned by APIs such as `read_stream_next_buffer`.
//!
//! Use `PinnedBufferLockGuard` when the caller needs a temporary lock on a
//! separately owned pin; it unlocks but does not release the pin.
//!
//! Use `LockedBufferGuard` when the caller owns both pin and lock:
//! `read_main` wraps `ReadBufferExtended` plus `LockBuffer`,
//! `read_main_locked` wraps modes that return an already-locked buffer such as
//! `RBM_ZERO_AND_LOCK`, and `lock_pinned` adopts a pre-pinned buffer before
//! locking it.

use std::marker::PhantomData;
use std::ptr;

use pgrx::pg_sys;

use crate::storage::page::ItemPointer;
use crate::storage::relation::RelationHandle;

pub(crate) struct PinnedBufferGuard {
    buffer: pg_sys::Buffer,
}

impl PinnedBufferGuard {
    #[cfg(not(feature = "pg18"))]
    pub(crate) unsafe fn read_main(
        relation: pg_sys::Relation,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
    ) -> Option<Self> {
        // SAFETY: caller supplies a live PostgreSQL relation and block number;
        // `ReadBufferExtended` pins valid buffers and `from_pinned` validates
        // before constructing the owning guard.
        unsafe {
            let buffer = pg_sys::ReadBufferExtended(
                relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                mode,
                ptr::null_mut(),
            );
            Self::from_pinned(buffer)
        }
    }

    pub(crate) unsafe fn from_pinned(buffer: pg_sys::Buffer) -> Option<Self> {
        // SAFETY: `buffer` is supplied by a PostgreSQL API that pins buffers
        // for the caller, such as `read_stream_next_buffer`.
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return None;
        }
        Some(Self { buffer })
    }

    #[allow(dead_code)]
    pub(crate) fn buffer(&self) -> pg_sys::Buffer {
        self.buffer
    }

    #[allow(dead_code)]
    pub(crate) fn block_number(&self) -> pg_sys::BlockNumber {
        // SAFETY: this guard owns a valid pinned buffer.
        unsafe { pg_sys::BufferGetBlockNumber(self.buffer) }
    }

    pub(crate) fn lock(&self, lockmode: i32) -> PinnedBufferLockGuard<'_> {
        // SAFETY: this guard owns a valid pinned buffer, so a short-lived
        // lock-only guard may borrow that pin and unlock without releasing it.
        unsafe { pg_sys::LockBuffer(self.buffer, lockmode) };
        PinnedBufferLockGuard {
            buffer: self.buffer,
            _pin: PhantomData,
        }
    }
}

impl Drop for PinnedBufferGuard {
    fn drop(&mut self) {
        // SAFETY: `buffer` was handed to this guard as an owned pin; this guard
        // owns the matching release.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::ReleaseBuffer(self.buffer) };
    }
}

pub(crate) struct PinnedBufferLockGuard<'a> {
    buffer: pg_sys::Buffer,
    _pin: PhantomData<&'a PinnedBufferGuard>,
}

impl PinnedBufferLockGuard<'_> {
    pub(crate) fn page(&self) -> pg_sys::Page {
        // SAFETY: this guard owns a valid lock on a pinned buffer borrowed
        // from `PinnedBufferGuard`.
        unsafe { pg_sys::BufferGetPage(self.buffer) }
    }

    pub(crate) fn page_size(&self) -> usize {
        // SAFETY: this guard owns a valid lock on a pinned buffer borrowed
        // from `PinnedBufferGuard`.
        unsafe { pg_sys::BufferGetPageSize(self.buffer) as usize }
    }
}

impl Drop for PinnedBufferLockGuard<'_> {
    fn drop(&mut self) {
        // SAFETY: this guard locked a buffer whose pin is owned by the
        // borrowed `PinnedBufferGuard`; only the lock is released here.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::LockBuffer(self.buffer, pg_sys::BUFFER_LOCK_UNLOCK as i32) };
    }
}

pub(crate) struct LockedBufferGuard {
    buffer: pg_sys::Buffer,
}

pub(crate) enum LockedPageTupleVisit<R> {
    Unused,
    Present(R),
}

impl LockedBufferGuard {
    pub(crate) unsafe fn read_main(
        relation: pg_sys::Relation,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
        lockmode: i32,
    ) -> Option<Self> {
        // SAFETY: caller supplies a live PostgreSQL relation and block number;
        // `ReadBufferExtended` pins valid buffers, `BufferIsValid` filters
        // invalid handles, and the matched `LockBuffer` pairs with this
        // guard's `UnlockReleaseBuffer` in Drop.
        let buffer = pg_sys::ReadBufferExtended(
            relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            block_number,
            mode,
            ptr::null_mut(),
        );
        if !pg_sys::BufferIsValid(buffer) {
            return None;
        }
        pg_sys::LockBuffer(buffer, lockmode);
        Some(Self { buffer })
    }

    pub(crate) unsafe fn read_main_locked(
        relation: pg_sys::Relation,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
    ) -> Option<Self> {
        // SAFETY: caller supplies a live PostgreSQL relation and a read mode
        // that returns the buffer already locked, such as `RBM_ZERO_AND_LOCK`;
        // `BufferIsValid` filters invalid handles.
        let buffer = pg_sys::ReadBufferExtended(
            relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            block_number,
            mode,
            ptr::null_mut(),
        );
        if !pg_sys::BufferIsValid(buffer) {
            return None;
        }
        Some(Self { buffer })
    }

    /// Safe variant of [`read_main`](Self::read_main) — callers that already
    /// hold a validated [`RelationHandle`] pass it through this entry point so
    /// the buffer guard inherits the handle's "live relation" SAFETY contract
    /// instead of re-establishing it at every call site.
    #[allow(dead_code)] // Consumed by HNSW migration in Task 54 packets 003/004.
    pub(crate) fn read_main_handle(
        handle: RelationHandle,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
        lockmode: i32,
    ) -> Option<Self> {
        // SAFETY: `RelationHandle` is a non-null pointer whose construction
        // contract requires a live opened PostgreSQL relation; that contract
        // satisfies `ReadBufferExtended` and `LockBuffer`.
        unsafe { Self::read_main(handle.as_ptr(), block_number, mode, lockmode) }
    }

    /// Safe variant of [`read_main_locked`](Self::read_main_locked) — see
    /// [`read_main_handle`](Self::read_main_handle) for the safety contract.
    #[allow(dead_code)] // Consumed by HNSW migration in Task 54 packets 003/004.
    pub(crate) fn read_main_locked_handle(
        handle: RelationHandle,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
    ) -> Option<Self> {
        // SAFETY: `RelationHandle` carries a live relation pointer; the
        // requested `mode` (e.g. `RBM_ZERO_AND_LOCK`) returns the buffer
        // already locked, so no follow-up `LockBuffer` is needed.
        unsafe { Self::read_main_locked(handle.as_ptr(), block_number, mode) }
    }

    pub(crate) unsafe fn lock_pinned(buffer: pg_sys::Buffer, lockmode: i32) -> Option<Self> {
        // SAFETY: `buffer` is supplied by a PostgreSQL API that pins buffers
        // for the caller (e.g. `read_stream_next_buffer`); `BufferIsValid`
        // filters invalid handles before the matched `LockBuffer`/Drop pair.
        if !pg_sys::BufferIsValid(buffer) {
            return None;
        }
        pg_sys::LockBuffer(buffer, lockmode);
        Some(Self { buffer })
    }

    pub(crate) fn page(&self) -> pg_sys::Page {
        // SAFETY: this guard owns a valid locked buffer.
        unsafe { pg_sys::BufferGetPage(self.buffer) }
    }

    pub(crate) fn buffer(&self) -> pg_sys::Buffer {
        self.buffer
    }

    pub(crate) fn page_size(&self) -> usize {
        // SAFETY: this guard owns a valid locked buffer.
        unsafe { pg_sys::BufferGetPageSize(self.buffer) as usize }
    }

    pub(crate) fn block_number(&self) -> pg_sys::BlockNumber {
        // SAFETY: this guard owns a valid locked buffer.
        unsafe { pg_sys::BufferGetBlockNumber(self.buffer) }
    }

    pub(crate) fn max_offset_number(&self) -> pg_sys::OffsetNumber {
        // SAFETY: this guard owns a valid locked buffer.
        unsafe { pg_sys::PageGetMaxOffsetNumber(self.page()) }
    }

    pub(crate) fn visit_tuple_bytes<R, F>(
        &self,
        tid: ItemPointer,
        tuple_kind: &str,
        visit: F,
    ) -> Result<LockedPageTupleVisit<R>, String>
    where
        F: FnOnce(&[u8]) -> Result<R, String>,
    {
        let page = self.page();
        let page_size = self.page_size();
        let block_number = self.block_number();

        // SAFETY: this guard owns a valid locked buffer. The requested line
        // pointer is range checked, tuple bounds are checked against the locked
        // page size, and the borrowed tuple bytes do not escape the visitor.
        unsafe {
            let max_offset = self.max_offset_number();
            if tid.offset_number == pg_sys::InvalidOffsetNumber || tid.offset_number > max_offset {
                return Err(format!(
                    "{tuple_kind} tuple offset {} out of range on block {} (max {})",
                    tid.offset_number, block_number, max_offset
                ));
            }

            let item_id = pg_sys::PageGetItemId(page, tid.offset_number);
            if item_id.is_null() {
                return Err(format!(
                    "{tuple_kind} tuple ({},{}) returned a null item id",
                    tid.block_number, tid.offset_number
                ));
            }
            let item_id_ref = &*item_id;
            if item_id_ref.lp_flags() == 0 {
                return Ok(LockedPageTupleVisit::Unused);
            }

            let tuple_offset = item_id_ref.lp_off() as usize;
            let tuple_len = item_id_ref.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                return Err(format!(
                    "{tuple_kind} tuple ({},{}) has invalid bounds",
                    tid.block_number, tid.offset_number
                ));
            }

            let tuple_ptr = pg_sys::PageGetItem(page, item_id).cast::<u8>();
            if tuple_ptr.is_null() {
                return Err(format!(
                    "{tuple_kind} tuple ({},{}) returned a null tuple pointer",
                    tid.block_number, tid.offset_number
                ));
            }
            let tuple_bytes = std::slice::from_raw_parts(tuple_ptr.cast_const(), tuple_len);
            visit(tuple_bytes).map(LockedPageTupleVisit::Present)
        }
    }
}

impl Drop for LockedBufferGuard {
    fn drop(&mut self) {
        // SAFETY: `buffer` was locked by a `LockedBufferGuard` constructor;
        // this guard owns the matching unlock and release.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::UnlockReleaseBuffer(self.buffer) };
    }
}
