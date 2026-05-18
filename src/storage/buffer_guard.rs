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
        // SAFETY: caller supplies a live PostgreSQL relation and block number.
        // The returned buffer pin is owned by this guard.
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                mode,
                ptr::null_mut(),
            )
        };
        // SAFETY: `buffer` is the direct result from `ReadBufferExtended`,
        // which pins valid buffers for the caller.
        unsafe { Self::from_pinned(buffer) }
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

impl LockedBufferGuard {
    pub(crate) unsafe fn read_main(
        relation: pg_sys::Relation,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
        lockmode: i32,
    ) -> Option<Self> {
        // SAFETY: caller supplies a live PostgreSQL relation and block number.
        // The returned buffer pin is owned by this guard.
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                mode,
                ptr::null_mut(),
            )
        };
        // SAFETY: `buffer` is the result from `ReadBufferExtended`.
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return None;
        }

        // SAFETY: `buffer` is valid and pinned; this guard owns the matching
        // `UnlockReleaseBuffer`.
        unsafe { pg_sys::LockBuffer(buffer, lockmode) };
        Some(Self { buffer })
    }

    pub(crate) unsafe fn read_main_locked(
        relation: pg_sys::Relation,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
    ) -> Option<Self> {
        // SAFETY: caller supplies a live PostgreSQL relation and a read mode
        // that returns the buffer already locked, such as `RBM_ZERO_AND_LOCK`.
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                mode,
                ptr::null_mut(),
            )
        };
        // SAFETY: `buffer` is the result from `ReadBufferExtended`.
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return None;
        }
        Some(Self { buffer })
    }

    pub(crate) unsafe fn lock_pinned(buffer: pg_sys::Buffer, lockmode: i32) -> Option<Self> {
        // SAFETY: `buffer` is supplied by a PostgreSQL API that pins buffers
        // for the caller, such as `read_stream_next_buffer`.
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return None;
        }

        // SAFETY: `buffer` is valid and pinned; this guard owns the matching
        // `UnlockReleaseBuffer`.
        unsafe { pg_sys::LockBuffer(buffer, lockmode) };
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
