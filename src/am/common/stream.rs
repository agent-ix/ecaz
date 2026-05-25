use pgrx::pg_sys;

use super::callback::pg_callback;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphPrefetchState {
    blocks: Vec<u32>,
    index: usize,
}

impl GraphPrefetchState {
    pub(crate) fn new(blocks: Vec<u32>) -> Self {
        Self { blocks, index: 0 }
    }

    pub(crate) fn reset(&mut self, blocks: Vec<u32>) {
        self.blocks.clear();
        self.blocks.extend(blocks);
        self.index = 0;
    }

    pub(crate) fn next_block(&mut self) -> Option<u32> {
        let block = self.blocks.get(self.index).copied()?;
        self.index += 1;
        Some(block)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockSequencePrefetchState {
    blocks: Vec<u32>,
    index: usize,
}

impl BlockSequencePrefetchState {
    pub(crate) fn new(blocks: Vec<u32>) -> Self {
        Self { blocks, index: 0 }
    }

    pub(crate) fn reset(&mut self, blocks: Vec<u32>) {
        self.blocks.clear();
        self.blocks.extend(blocks);
        self.index = 0;
    }

    pub(crate) fn next_block(&mut self) -> Option<u32> {
        let block = self.blocks.get(self.index).copied()?;
        self.index += 1;
        Some(block)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LinearPrefetchState {
    next_block: u32,
    max_block: u32,
}

impl LinearPrefetchState {
    pub(crate) fn new(next_block: u32, max_block: u32) -> Self {
        Self {
            next_block,
            max_block,
        }
    }

    pub(crate) fn reset(&mut self, next_block: u32, max_block: u32) {
        self.next_block = next_block;
        self.max_block = max_block;
    }

    pub(crate) fn next_block(&mut self) -> Option<u32> {
        if self.next_block > self.max_block {
            return None;
        }

        let block = self.next_block;
        self.next_block += 1;
        Some(block)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadStreamCallbackSignature {
    pub callback_name: &'static str,
    pub stream_mode: &'static str,
    pub access_pattern: &'static str,
    pub state_type: &'static str,
    pub end_of_stream_sentinel: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadStreamCallbackResult {
    Block(u32),
    EndOfStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadStreamSnapshot {
    pub graph_stream_mode: &'static str,
    pub linear_stream_mode: &'static str,
    pub graph_stream_access_pattern: &'static str,
    pub linear_stream_access_pattern: &'static str,
    pub pg18_callback_surface_ready: bool,
    pub pg18_scan_wiring_ready: bool,
    pub pg18_vacuum_wiring_ready: bool,
}

pub(crate) fn graph_callback_signature() -> ReadStreamCallbackSignature {
    ReadStreamCallbackSignature {
        callback_name: "graph_prefetch_cb",
        stream_mode: "READ_STREAM_DEFAULT",
        access_pattern: "random",
        state_type: "GraphPrefetchState",
        end_of_stream_sentinel: "InvalidBlockNumber",
    }
}

pub(crate) fn graph_prefetch_callback(state: &mut GraphPrefetchState) -> ReadStreamCallbackResult {
    match state.next_block() {
        Some(block) => ReadStreamCallbackResult::Block(block),
        None => ReadStreamCallbackResult::EndOfStream,
    }
}

pub(crate) fn linear_callback_signature() -> ReadStreamCallbackSignature {
    ReadStreamCallbackSignature {
        callback_name: "linear_prefetch_cb",
        stream_mode: "READ_STREAM_SEQUENTIAL",
        access_pattern: "sequential",
        state_type: "LinearPrefetchState",
        end_of_stream_sentinel: "InvalidBlockNumber",
    }
}

pub(crate) fn linear_prefetch_callback(
    state: &mut LinearPrefetchState,
) -> ReadStreamCallbackResult {
    match state.next_block() {
        Some(block) => ReadStreamCallbackResult::Block(block),
        None => ReadStreamCallbackResult::EndOfStream,
    }
}

pub(crate) fn block_sequence_prefetch_callback(
    state: &mut BlockSequencePrefetchState,
) -> ReadStreamCallbackResult {
    match state.next_block() {
        Some(block) => ReadStreamCallbackResult::Block(block),
        None => ReadStreamCallbackResult::EndOfStream,
    }
}

#[cfg(feature = "pg18")]
pub(crate) fn prefetch_relation_blocks(
    relation: pg_sys::Relation,
    block_numbers: Vec<pg_sys::BlockNumber>,
    context: &str,
) {
    if block_numbers.is_empty() {
        return;
    }

    let mut state = BlockSequencePrefetchState::new(block_numbers);
    // SAFETY: `relation` is open for the caller's scan; `state` lives until
    // the returned scope drops; the registered callback only consumes a
    // `BlockSequencePrefetchState` callback-private pointer.
    let mut stream = unsafe {
        ReadStreamScope::open(
            pg_sys::READ_STREAM_DEFAULT as i32,
            relation,
            block_sequence_prefetch_cb,
            (&mut state as *mut BlockSequencePrefetchState).cast(),
        )
    };

    while let Some(entry) = stream.next_pinned() {
        let (_pinned, _block) = match entry {
            Ok(pair) => pair,
            Err(err) => pgrx::error!("{context} {err}"),
        };
        // The pinned buffer guard releases on drop; the loop discards the
        // guard so each pin is acquired-and-released in order.
    }
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn prefetch_relation_blocks(
    relation: pg_sys::Relation,
    block_numbers: Vec<pg_sys::BlockNumber>,
    _context: &str,
) {
    for block_number in block_numbers {
        // SAFETY: relation is open for the caller's scan, and each block number
        // came from candidate heap TIDs for that relation.
        unsafe { pg_sys::PrefetchBuffer(relation, pg_sys::ForkNumber::MAIN_FORKNUM, block_number) };
    }
}

/// Typed RAII scope around `pg_sys::ReadStream` for the in-`stream.rs`
/// prefetch / visit helpers.
///
/// * Construction (`open`) wraps `read_stream_begin_relation`.
/// * `next_pinned` / `next_locked` consume one stream-yielded buffer per
///   call and return it as a typed `PinnedBufferGuard` / `LockedBufferGuard`
///   alongside the per-buffer-data block-number extraction.
/// * `Drop` calls `read_stream_end` once.
///
/// Per `feedback_view_operations_not_accessors`, no safe accessor leaks
/// the underlying `*mut pg_sys::ReadStream` or a raw `pg_sys::Buffer`.
///
/// Cross-AM consumers (`visit_scan_owned_read_stream_pinned/_locked`,
/// `reset_scan_owned_read_stream`) operate on the raw `*mut
/// pg_sys::ReadStream` held by the AM scan opaque; threading those
/// through `ReadStreamScope` requires AM scan-opaque migration that is
/// out of scope for Task 59 §Non-Goals.
#[cfg(feature = "pg18")]
pub(crate) struct ReadStreamScope<'rel> {
    stream: *mut pg_sys::ReadStream,
    _marker: std::marker::PhantomData<&'rel mut pg_sys::ReadStream>,
}

#[cfg(feature = "pg18")]
impl<'rel> ReadStreamScope<'rel> {
    /// Open a read stream over `relation` using `callback` for block
    /// supply. The returned scope owns the single `read_stream_end` call
    /// via `Drop`.
    ///
    /// # Safety
    ///
    /// `relation` must be open for the caller's scan with the appropriate
    /// lock and remain open for the returned scope's lifetime.
    /// `callback_private_data` must point at the state structure
    /// `callback` expects, must remain live until the scope drops, and
    /// must not be aliased outside the callback. The registered
    /// `callback` must match the layout of the state.
    pub(crate) unsafe fn open(
        mode: i32,
        relation: pg_sys::Relation,
        callback: unsafe extern "C-unwind" fn(
            *mut pg_sys::ReadStream,
            *mut std::ffi::c_void,
            *mut std::ffi::c_void,
        ) -> pg_sys::BlockNumber,
        callback_private_data: *mut std::ffi::c_void,
    ) -> Self {
        // SAFETY: caller asserts relation, callback, and callback-private
        // contract per the doc above.
        let stream = unsafe {
            pg_sys::read_stream_begin_relation(
                mode,
                std::ptr::null_mut(),
                relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                Some(callback),
                callback_private_data,
                std::mem::size_of::<pg_sys::BlockNumber>(),
            )
        };
        Self {
            stream,
            _marker: std::marker::PhantomData,
        }
    }

    /// Yield the next pinned buffer along with its per-buffer-data block
    /// number (if the callback supplied one).
    ///
    /// Returns:
    /// * `None` when the stream is exhausted (PG returns `InvalidBuffer`).
    /// * `Some(Err)` if PG returned a buffer but the typed `PinnedBufferGuard`
    ///   construction rejected it (defensive; should not happen in normal
    ///   operation).
    /// * `Some(Ok((guard, block)))` on a successful yield; the guard owns
    ///   the pin release on drop.
    pub(crate) fn next_pinned(
        &mut self,
    ) -> Option<
        Result<
            (
                crate::storage::buffer_guard::PinnedBufferGuard,
                Option<pg_sys::BlockNumber>,
            ),
            String,
        >,
    > {
        let mut per_buffer_data = std::ptr::null_mut();
        // SAFETY: `self.stream` was opened by `Self::open` and the scope
        // owns it for `'rel`; PG returns either a pinned buffer or
        // `InvalidBuffer` to signal end-of-stream.
        let pinned = unsafe {
            let buffer = pg_sys::read_stream_next_buffer(self.stream, &mut per_buffer_data);
            if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
                return None;
            }
            crate::storage::buffer_guard::PinnedBufferGuard::from_pinned(buffer)
        };
        let block_number = read_stream_per_buffer_block_number(per_buffer_data);
        Some(
            pinned
                .map(|p| (p, block_number))
                .ok_or_else(|| "read stream returned an invalid buffer".to_owned()),
        )
    }

    /// Yield the next buffer locked at `lockmode` along with the
    /// callback-supplied block number (falling back to the buffer's own
    /// block number when the callback did not write one).
    ///
    /// Returns:
    /// * `None` when the stream is exhausted.
    /// * `Some(Err)` if the lock acquisition rejected the pin (defensive).
    /// * `Some(Ok((guard, block_number)))` on a successful yield.
    pub(crate) fn next_locked(
        &mut self,
        lockmode: i32,
    ) -> Option<
        Result<
            (
                crate::storage::buffer_guard::LockedBufferGuard,
                pg_sys::BlockNumber,
            ),
            String,
        >,
    > {
        let mut per_buffer_data = std::ptr::null_mut();
        // SAFETY: `self.stream` is live for the scope's lifetime; PG
        // returns either a pinned buffer or `InvalidBuffer` for EOS, and
        // the typed `LockedBufferGuard` wraps the pin + acquired lock.
        let locked = unsafe {
            let buffer = pg_sys::read_stream_next_buffer(self.stream, &mut per_buffer_data);
            if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
                return None;
            }
            crate::storage::buffer_guard::LockedBufferGuard::lock_pinned(buffer, lockmode)
        };
        let Some(locked) = locked else {
            return Some(Err("read stream returned an invalid buffer".to_owned()));
        };
        let block_number = read_stream_per_buffer_block_number(per_buffer_data)
            .unwrap_or_else(|| locked.block_number());
        Some(Ok((locked, block_number)))
    }
}

#[cfg(feature = "pg18")]
impl Drop for ReadStreamScope<'_> {
    fn drop(&mut self) {
        // SAFETY: `self.stream` was opened by `read_stream_begin_relation`
        // inside `Self::open`; the scope owns exactly one `read_stream_end`
        // call.
        unsafe { pg_sys::read_stream_end(self.stream) };
    }
}

#[cfg(feature = "pg18")]
fn read_stream_per_buffer_block_number(
    per_buffer_data: *mut std::ffi::c_void,
) -> Option<pg_sys::BlockNumber> {
    if per_buffer_data.is_null() {
        None
    } else {
        // SAFETY: registered read-stream callbacks store one BlockNumber in
        // per-buffer data when PostgreSQL supplies the slot.
        Some(unsafe { *per_buffer_data.cast::<pg_sys::BlockNumber>() })
    }
}

#[cfg(feature = "pg18")]
fn visit_read_stream<F>(
    stream: &mut ReadStreamScope<'_>,
    context: &str,
    mut visitor: F,
) -> Result<(), String>
where
    F: FnMut(
        &crate::storage::buffer_guard::LockedBufferGuard,
        pg_sys::BlockNumber,
    ) -> Result<(), String>,
{
    while let Some(entry) = stream.next_locked(pg_sys::BUFFER_LOCK_SHARE as i32) {
        let (buffer, block_number) = entry.map_err(|err| format!("{context} {err}"))?;
        visitor(&buffer, block_number)?;
    }
    Ok(())
}

#[cfg(feature = "pg18")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScanOwnedReadStreamControl {
    Continue,
    Stop,
}

/// Yield the next pinned buffer from a scan-owned `pg_sys::ReadStream`,
/// or `None` at end-of-stream. Inner `None` indicates PG returned a
/// buffer but `PinnedBufferGuard::from_pinned` rejected it (defensive).
#[cfg(feature = "pg18")]
fn next_scan_owned_pinned(
    stream: *mut pg_sys::ReadStream,
) -> Option<
    Option<(
        crate::storage::buffer_guard::PinnedBufferGuard,
        Option<pg_sys::BlockNumber>,
    )>,
> {
    let mut per_buffer_data = std::ptr::null_mut();
    // SAFETY: `stream` is a live scan-owned read stream retained by the
    // AM scan opaque; the typed `PinnedBufferGuard` wraps the pin.
    let guard = unsafe {
        let buffer = pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data);
        if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
            return None;
        }
        crate::storage::buffer_guard::PinnedBufferGuard::from_pinned(buffer)
    };
    let block_number = read_stream_per_buffer_block_number(per_buffer_data);
    Some(guard.map(|g| (g, block_number)))
}

/// Yield the next buffer from a scan-owned `pg_sys::ReadStream` locked
/// at `lockmode`, along with the callback-supplied block number (falling
/// back to the buffer's own block number when none was supplied). Inner
/// `None` indicates PG returned a buffer but the lock acquisition
/// rejected it.
#[cfg(feature = "pg18")]
fn next_scan_owned_locked(
    stream: *mut pg_sys::ReadStream,
    lockmode: i32,
) -> Option<
    Option<(
        crate::storage::buffer_guard::LockedBufferGuard,
        pg_sys::BlockNumber,
    )>,
> {
    let mut per_buffer_data = std::ptr::null_mut();
    // SAFETY: `stream` is a live scan-owned read stream retained by the
    // AM scan opaque; the typed `LockedBufferGuard` wraps the pin + lock.
    let locked = unsafe {
        let buffer = pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data);
        if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
            return None;
        }
        crate::storage::buffer_guard::LockedBufferGuard::lock_pinned(buffer, lockmode)
    };
    let Some(locked) = locked else {
        return Some(None);
    };
    let block_number = read_stream_per_buffer_block_number(per_buffer_data)
        .unwrap_or_else(|| locked.block_number());
    Some(Some((locked, block_number)))
}

#[cfg(feature = "pg18")]
pub(crate) fn reset_scan_owned_read_stream(
    stream: *mut pg_sys::ReadStream,
    context: &str,
) -> Result<(), String> {
    if stream.is_null() {
        return Err(format!("{context} read stream is not initialized"));
    }
    // SAFETY: callers pass a scan-owned read stream opened by
    // read_stream_begin_relation and retained by the scan opaque.
    unsafe { pg_sys::read_stream_reset(stream) };
    Ok(())
}

#[cfg(feature = "pg18")]
pub(crate) fn visit_scan_owned_read_stream_pinned<F>(
    stream: *mut pg_sys::ReadStream,
    context: &str,
    mut visitor: F,
) -> Result<(), String>
where
    F: FnMut(
        crate::storage::buffer_guard::PinnedBufferGuard,
        Option<pg_sys::BlockNumber>,
    ) -> Result<ScanOwnedReadStreamControl, String>,
{
    if stream.is_null() {
        return Err(format!("{context} read stream is not initialized"));
    }
    while let Some(entry) = next_scan_owned_pinned(stream) {
        let (buffer, block_number) =
            entry.ok_or_else(|| format!("{context} read stream returned an invalid buffer"))?;
        if visitor(buffer, block_number)? == ScanOwnedReadStreamControl::Stop {
            break;
        }
    }
    Ok(())
}

#[cfg(feature = "pg18")]
pub(crate) fn visit_scan_owned_read_stream_locked<F>(
    stream: *mut pg_sys::ReadStream,
    lockmode: i32,
    context: &str,
    mut visitor: F,
) -> Result<(), String>
where
    F: FnMut(
        crate::storage::buffer_guard::LockedBufferGuard,
        Option<pg_sys::BlockNumber>,
    ) -> Result<ScanOwnedReadStreamControl, String>,
{
    if stream.is_null() {
        return Err(format!("{context} read stream is not initialized"));
    }
    while let Some(entry) = next_scan_owned_locked(stream, lockmode) {
        let (buffer, block_number) =
            entry.ok_or_else(|| format!("{context} read stream returned an invalid buffer"))?;
        if visitor(buffer, Some(block_number))? == ScanOwnedReadStreamControl::Stop {
            break;
        }
    }
    Ok(())
}

#[cfg(feature = "pg18")]
pub(crate) fn visit_relation_linear_read_stream<F>(
    relation: pg_sys::Relation,
    first_block: pg_sys::BlockNumber,
    last_block: pg_sys::BlockNumber,
    context: &str,
    visitor: F,
) -> Result<(), String>
where
    F: FnMut(
        &crate::storage::buffer_guard::LockedBufferGuard,
        pg_sys::BlockNumber,
    ) -> Result<(), String>,
{
    let mut state = LinearPrefetchState::new(first_block, last_block);
    // SAFETY: `relation` is open for the caller's scan; `state` lives
    // until the scope drops; `linear_prefetch_cb` matches the
    // `LinearPrefetchState` layout.
    let mut stream = unsafe {
        ReadStreamScope::open(
            pg_sys::READ_STREAM_SEQUENTIAL as i32,
            relation,
            linear_prefetch_cb,
            (&mut state as *mut LinearPrefetchState).cast(),
        )
    };
    visit_read_stream(&mut stream, context, visitor)
}

#[cfg(feature = "pg18")]
pub(crate) fn visit_relation_block_sequence_read_stream<F>(
    relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
    context: &str,
    visitor: F,
) -> Result<(), String>
where
    F: FnMut(
        &crate::storage::buffer_guard::LockedBufferGuard,
        pg_sys::BlockNumber,
    ) -> Result<(), String>,
{
    if block_numbers.is_empty() {
        return Ok(());
    }
    let mut state = BlockSequencePrefetchState::new(block_numbers.to_vec());
    // SAFETY: `relation` is open for the caller's scan; `state` lives
    // until the scope drops; `block_sequence_prefetch_cb` matches the
    // `BlockSequencePrefetchState` layout.
    let mut stream = unsafe {
        ReadStreamScope::open(
            pg_sys::READ_STREAM_SEQUENTIAL as i32,
            relation,
            block_sequence_prefetch_cb,
            (&mut state as *mut BlockSequencePrefetchState).cast(),
        )
    };
    visit_read_stream(&mut stream, context, visitor)
}

pub(crate) fn stream_snapshot() -> ReadStreamSnapshot {
    let graph = graph_callback_signature();
    let linear = linear_callback_signature();
    ReadStreamSnapshot {
        graph_stream_mode: graph.stream_mode,
        linear_stream_mode: linear.stream_mode,
        graph_stream_access_pattern: graph.access_pattern,
        linear_stream_access_pattern: linear.access_pattern,
        pg18_callback_surface_ready: cfg!(feature = "pg18"),
        pg18_scan_wiring_ready: cfg!(feature = "pg18"),
        pg18_vacuum_wiring_ready: cfg!(feature = "pg18"),
    }
}

#[cfg(feature = "pg18")]
fn write_stream_block(per_buffer_data: *mut std::ffi::c_void, block_number: u32) {
    let block_slot = per_buffer_data.cast::<pg_sys::BlockNumber>();
    if !block_slot.is_null() {
        // SAFETY: PostgreSQL ReadStream passes `per_buffer_data` as either null
        // or writable storage for one `BlockNumber`; the null case is handled
        // above before writing the selected block.
        unsafe {
            *block_slot = block_number;
        }
    }
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn graph_prefetch_cb(
    _stream: *mut pg_sys::ReadStream,
    callback_private_data: *mut std::ffi::c_void,
    per_buffer_data: *mut std::ffi::c_void,
) -> pg_sys::BlockNumber {
    pg_callback!({
        if callback_private_data.is_null() {
            return pg_sys::InvalidBlockNumber;
        }
        // SAFETY: The graph stream is registered with a `GraphPrefetchState`
        // callback-private pointer that outlives the callback invocation.
        let state = &mut *callback_private_data.cast::<GraphPrefetchState>();
        match graph_prefetch_callback(state) {
            ReadStreamCallbackResult::Block(block_number) => {
                write_stream_block(per_buffer_data, block_number);
                block_number
            }
            ReadStreamCallbackResult::EndOfStream => pg_sys::InvalidBlockNumber,
        }
    })
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn linear_prefetch_cb(
    _stream: *mut pg_sys::ReadStream,
    callback_private_data: *mut std::ffi::c_void,
    per_buffer_data: *mut std::ffi::c_void,
) -> pg_sys::BlockNumber {
    pg_callback!({
        if callback_private_data.is_null() {
            return pg_sys::InvalidBlockNumber;
        }
        // SAFETY: The linear stream is registered with a `LinearPrefetchState`
        // callback-private pointer that outlives the callback invocation.
        let state = &mut *callback_private_data.cast::<LinearPrefetchState>();
        match linear_prefetch_callback(state) {
            ReadStreamCallbackResult::Block(block_number) => {
                write_stream_block(per_buffer_data, block_number);
                block_number
            }
            ReadStreamCallbackResult::EndOfStream => pg_sys::InvalidBlockNumber,
        }
    })
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn block_sequence_prefetch_cb(
    _stream: *mut pg_sys::ReadStream,
    callback_private_data: *mut std::ffi::c_void,
    per_buffer_data: *mut std::ffi::c_void,
) -> pg_sys::BlockNumber {
    pg_callback!({
        if callback_private_data.is_null() {
            return pg_sys::InvalidBlockNumber;
        }
        // SAFETY: The block-sequence stream is registered with a
        // `BlockSequencePrefetchState` callback-private pointer that outlives
        // the callback invocation.
        let state = &mut *callback_private_data.cast::<BlockSequencePrefetchState>();
        match block_sequence_prefetch_callback(state) {
            ReadStreamCallbackResult::Block(block_number) => {
                write_stream_block(per_buffer_data, block_number);
                block_number
            }
            ReadStreamCallbackResult::EndOfStream => pg_sys::InvalidBlockNumber,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        block_sequence_prefetch_callback, graph_callback_signature, graph_prefetch_callback,
        linear_callback_signature, linear_prefetch_callback, stream_snapshot,
        BlockSequencePrefetchState, GraphPrefetchState, LinearPrefetchState,
        ReadStreamCallbackResult, ReadStreamCallbackSignature, ReadStreamSnapshot,
    };

    #[test]
    fn stream_snapshot_matches_build_target() {
        assert_eq!(
            stream_snapshot(),
            ReadStreamSnapshot {
                graph_stream_mode: "READ_STREAM_DEFAULT",
                linear_stream_mode: "READ_STREAM_SEQUENTIAL",
                graph_stream_access_pattern: "random",
                linear_stream_access_pattern: "sequential",
                pg18_callback_surface_ready: cfg!(feature = "pg18"),
                pg18_scan_wiring_ready: cfg!(feature = "pg18"),
                pg18_vacuum_wiring_ready: cfg!(feature = "pg18"),
            }
        );
    }

    #[test]
    fn graph_callback_signature_matches_fr019_contract() {
        assert_eq!(
            graph_callback_signature(),
            ReadStreamCallbackSignature {
                callback_name: "graph_prefetch_cb",
                stream_mode: "READ_STREAM_DEFAULT",
                access_pattern: "random",
                state_type: "GraphPrefetchState",
                end_of_stream_sentinel: "InvalidBlockNumber",
            }
        );
    }

    #[test]
    fn linear_callback_signature_matches_fr019_contract() {
        assert_eq!(
            linear_callback_signature(),
            ReadStreamCallbackSignature {
                callback_name: "linear_prefetch_cb",
                stream_mode: "READ_STREAM_SEQUENTIAL",
                access_pattern: "sequential",
                state_type: "LinearPrefetchState",
                end_of_stream_sentinel: "InvalidBlockNumber",
            }
        );
    }

    #[test]
    fn graph_prefetch_state_advances_until_exhausted() {
        let mut state = GraphPrefetchState::new(vec![11, 14, 18]);

        assert_eq!(state.next_block(), Some(11));
        assert_eq!(state.next_block(), Some(14));
        assert_eq!(state.next_block(), Some(18));
        assert_eq!(state.next_block(), None);
    }

    #[test]
    fn graph_prefetch_state_reset_restarts_with_new_batch() {
        let mut state = GraphPrefetchState::new(vec![11, 14, 18]);

        assert_eq!(state.next_block(), Some(11));
        assert_eq!(state.next_block(), Some(14));

        state.reset(vec![21, 22]);

        assert_eq!(state.next_block(), Some(21));
        assert_eq!(state.next_block(), Some(22));
        assert_eq!(state.next_block(), None);
    }

    #[test]
    fn graph_prefetch_callback_returns_blocks_then_end_of_stream() {
        let mut state = GraphPrefetchState::new(vec![11, 14]);

        assert_eq!(
            graph_prefetch_callback(&mut state),
            ReadStreamCallbackResult::Block(11)
        );
        assert_eq!(
            graph_prefetch_callback(&mut state),
            ReadStreamCallbackResult::Block(14)
        );
        assert_eq!(
            graph_prefetch_callback(&mut state),
            ReadStreamCallbackResult::EndOfStream
        );
    }

    #[test]
    fn linear_prefetch_state_advances_sequentially_until_exhausted() {
        let mut state = LinearPrefetchState::new(21, 23);

        assert_eq!(state.next_block(), Some(21));
        assert_eq!(state.next_block(), Some(22));
        assert_eq!(state.next_block(), Some(23));
        assert_eq!(state.next_block(), None);
    }

    #[test]
    fn linear_prefetch_state_reset_restarts_range() {
        let mut state = LinearPrefetchState::new(21, 23);

        assert_eq!(state.next_block(), Some(21));
        assert_eq!(state.next_block(), Some(22));

        state.reset(30, 31);

        assert_eq!(state.next_block(), Some(30));
        assert_eq!(state.next_block(), Some(31));
        assert_eq!(state.next_block(), None);
    }

    #[test]
    fn block_sequence_prefetch_state_advances_until_exhausted() {
        let mut state = BlockSequencePrefetchState::new(vec![4, 7, 8, 12]);

        assert_eq!(state.next_block(), Some(4));
        assert_eq!(state.next_block(), Some(7));
        assert_eq!(state.next_block(), Some(8));
        assert_eq!(state.next_block(), Some(12));
        assert_eq!(state.next_block(), None);
    }

    #[test]
    fn block_sequence_prefetch_state_reset_restarts_with_new_blocks() {
        let mut state = BlockSequencePrefetchState::new(vec![4, 7, 8]);

        assert_eq!(state.next_block(), Some(4));
        assert_eq!(state.next_block(), Some(7));

        state.reset(vec![20, 21]);

        assert_eq!(state.next_block(), Some(20));
        assert_eq!(state.next_block(), Some(21));
        assert_eq!(state.next_block(), None);
    }

    #[test]
    fn block_sequence_prefetch_callback_reports_end_of_stream() {
        let mut state = BlockSequencePrefetchState::new(vec![31, 33]);

        assert_eq!(
            block_sequence_prefetch_callback(&mut state),
            ReadStreamCallbackResult::Block(31)
        );
        assert_eq!(
            block_sequence_prefetch_callback(&mut state),
            ReadStreamCallbackResult::Block(33)
        );
        assert_eq!(
            block_sequence_prefetch_callback(&mut state),
            ReadStreamCallbackResult::EndOfStream
        );
    }

    #[test]
    fn linear_prefetch_callback_returns_blocks_then_end_of_stream() {
        let mut state = LinearPrefetchState::new(21, 22);

        assert_eq!(
            linear_prefetch_callback(&mut state),
            ReadStreamCallbackResult::Block(21)
        );
        assert_eq!(
            linear_prefetch_callback(&mut state),
            ReadStreamCallbackResult::Block(22)
        );
        assert_eq!(
            linear_prefetch_callback(&mut state),
            ReadStreamCallbackResult::EndOfStream
        );
    }
}
