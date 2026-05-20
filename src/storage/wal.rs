//! WAL helpers shared across access methods.
//!
//! ECAZ currently writes PostgreSQL GenericXLog records only. Those records
//! carry page images/deltas rather than an extension-owned record body, so the
//! versioned byte contract lives in the page payloads themselves.

use pgrx::pg_sys;

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
    pub unsafe fn register_buffer(&mut self, buffer: pg_sys::Buffer, flags: i32) -> pg_sys::Page {
        assert!(
            !self.finished,
            "cannot register buffer after GenericXLogFinish"
        );
        // SAFETY: The caller guarantees `buffer` and `flags` are valid.
        unsafe { pg_sys::GenericXLogRegisterBuffer(self.state, buffer, flags) }
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

impl Drop for GenericXLogTxn {
    fn drop(&mut self) {
        if !self.finished && !self.state.is_null() {
            // SAFETY: Aborting an unfinished GenericXLog state is PostgreSQL's
            // required cleanup path.
            unsafe { pg_sys::GenericXLogAbort(self.state) };
        }
    }
}
