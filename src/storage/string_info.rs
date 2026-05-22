//! PostgreSQL `StringInfo` receive-buffer helpers.

use std::marker::PhantomData;

use pgrx::{pg_sys, Internal};

pub(crate) struct StringInfoReader<'msg> {
    msg: pg_sys::StringInfo,
    _marker: PhantomData<&'msg mut pg_sys::StringInfoData>,
}

impl<'msg> StringInfoReader<'msg> {
    pub(crate) fn from_internal(input: Internal, label: &str) -> Result<Self, String> {
        // SAFETY: PostgreSQL type receive functions pass their `internal`
        // argument as a live `StringInfoData` input buffer for the duration of
        // the receive call.
        let msg = unsafe {
            input
                .get::<pg_sys::StringInfoData>()
                .ok_or_else(|| format!("{label}: missing input buffer"))?
                as *const pg_sys::StringInfoData as pg_sys::StringInfo
        };
        // SAFETY: `msg` is the live receive buffer extracted from `input`
        // above and the returned reader is scoped to this receive call.
        unsafe { Self::from_raw(msg, label) }
    }

    /// Create a reader for PostgreSQL's live type-receive buffer.
    ///
    /// # Safety
    ///
    /// `msg` must be the live `StringInfo` pointer PostgreSQL passed to the
    /// current receive function, and this reader must not outlive that call.
    pub(crate) unsafe fn from_raw(msg: pg_sys::StringInfo, label: &str) -> Result<Self, String> {
        if msg.is_null() {
            return Err(format!("{label}: missing input buffer"));
        }
        Ok(Self {
            msg,
            _marker: PhantomData,
        })
    }

    pub(crate) fn remaining_len(&self, label: &str) -> Result<usize, String> {
        // SAFETY: `StringInfoReader` is constructed only for PostgreSQL's live
        // receive buffer; len and cursor are read-only fields copied by value.
        let total_len = usize::try_from(unsafe { (*self.msg).len })
            .map_err(|_| format!("{label}: invalid binary length"))?;
        // SAFETY: same live StringInfo; reading cursor does not mutate the buffer.
        let cursor = usize::try_from(unsafe { (*self.msg).cursor })
            .map_err(|_| format!("{label}: invalid binary cursor"))?;
        if cursor > total_len {
            return Err(format!("{label}: invalid binary cursor state"));
        }
        Ok(total_len - cursor)
    }

    pub(crate) fn read_bytes(&mut self, len: usize, label: &str) -> Result<Vec<u8>, String> {
        if len > i32::MAX as usize {
            return Err(format!("{label}: binary payload too large"));
        }
        if len == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: `StringInfoReader` owns the live receive-buffer cursor for
        // this call. PostgreSQL advances the cursor and returns a pointer to
        // the requested byte range.
        let ptr = unsafe { pg_sys::pq_getmsgbytes(self.msg, len as i32) as *const u8 };
        // SAFETY: PostgreSQL returned a pointer to exactly len bytes above;
        // copy them before returning so no borrowed message-buffer lifetime escapes.
        Ok(unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec())
    }

    pub(crate) fn finish(self) -> Result<(), String> {
        // SAFETY: caller consumed and validated the message payload;
        // PostgreSQL verifies the cursor is at the end of the live buffer.
        unsafe { pg_sys::pq_getmsgend(self.msg) };
        Ok(())
    }
}
