//! PostgreSQL `StringInfo` receive-buffer helpers.

use pgrx::pg_sys;

pub(crate) fn remaining_len(msg: pg_sys::StringInfo, label: &str) -> Result<usize, String> {
    if msg.is_null() {
        return Err(format!("{label}: missing input buffer"));
    }
    // SAFETY: callers pass PostgreSQL's live StringInfo for the current type
    // receive call; len and cursor are read-only fields copied by value.
    let total_len = usize::try_from(unsafe { (*msg).len })
        .map_err(|_| format!("{label}: invalid binary length"))?;
    // SAFETY: same live StringInfo; reading cursor does not mutate the buffer.
    let cursor = usize::try_from(unsafe { (*msg).cursor })
        .map_err(|_| format!("{label}: invalid binary cursor"))?;
    if cursor > total_len {
        return Err(format!("{label}: invalid binary cursor state"));
    }
    Ok(total_len - cursor)
}

pub(crate) fn read_bytes(
    msg: pg_sys::StringInfo,
    len: usize,
    label: &str,
) -> Result<Vec<u8>, String> {
    if msg.is_null() {
        return Err(format!("{label}: missing input buffer"));
    }
    if len > i32::MAX as usize {
        return Err(format!("{label}: binary payload too large"));
    }
    if len == 0 {
        return Ok(Vec::new());
    }
    // SAFETY: callers request bytes from a live PostgreSQL StringInfo. PostgreSQL
    // advances the cursor and returns a pointer to the requested byte range.
    let ptr = unsafe { pg_sys::pq_getmsgbytes(msg, len as i32) as *const u8 };
    // SAFETY: PostgreSQL returned a pointer to exactly len bytes above; copy
    // them before returning so no borrowed message-buffer lifetime escapes.
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec())
}

pub(crate) fn finish(msg: pg_sys::StringInfo, label: &str) -> Result<(), String> {
    if msg.is_null() {
        return Err(format!("{label}: missing input buffer"));
    }
    // SAFETY: caller consumed and validated the message payload; PostgreSQL
    // verifies the cursor is at the end of the live StringInfo buffer.
    unsafe { pg_sys::pq_getmsgend(msg) };
    Ok(())
}
