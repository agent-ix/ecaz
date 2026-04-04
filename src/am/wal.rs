//! GenericXLog helpers for tqhnsw page writes.

use pgrx::pg_sys;

/// RAII wrapper around PostgreSQL's GenericXLog state.
///
/// If the transaction is dropped without calling [`finish`](Self::finish),
/// PostgreSQL aborts the pending generic WAL record.
pub struct GenericXLogTxn {
    state: *mut pg_sys::GenericXLogState,
    finished: bool,
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

    /// Register a buffer for modification and return the writable page pointer.
    ///
    /// # Safety
    ///
    /// `buffer` must be a valid buffer belonging to the relation used to start
    /// this transaction. `flags` must follow PostgreSQL's GenericXLog contract.
    pub unsafe fn register_buffer(
        &mut self,
        buffer: pg_sys::Buffer,
        flags: i32,
    ) -> pg_sys::Page {
        assert!(!self.finished, "cannot register buffer after GenericXLogFinish");
        // SAFETY: The caller guarantees `buffer` and `flags` are valid.
        unsafe { pg_sys::GenericXLogRegisterBuffer(self.state, buffer, flags) }
    }

    /// Finish the GenericXLog transaction and return the written WAL pointer.
    ///
    /// # Safety
    ///
    /// All registered pages must already contain their final intended contents.
    pub unsafe fn finish(mut self) -> pg_sys::XLogRecPtr {
        self.finished = true;
        // SAFETY: `self.state` came from `GenericXLogStart` and has not yet been finished/aborted.
        unsafe { pg_sys::GenericXLogFinish(self.state) }
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
