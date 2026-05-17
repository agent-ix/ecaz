use std::ptr;

use pgrx::pg_sys;

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

    pub(crate) fn page(&self) -> pg_sys::Page {
        // SAFETY: this guard owns a valid locked buffer.
        unsafe { pg_sys::BufferGetPage(self.buffer) }
    }

    pub(crate) fn page_size(&self) -> usize {
        // SAFETY: this guard owns a valid locked buffer.
        unsafe { pg_sys::BufferGetPageSize(self.buffer) as usize }
    }
}

impl Drop for LockedBufferGuard {
    fn drop(&mut self) {
        // SAFETY: `buffer` was locked by `LockedBufferGuard::read_main`; this
        // guard owns the matching unlock and release.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::UnlockReleaseBuffer(self.buffer) };
    }
}
