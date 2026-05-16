#[cfg(feature = "pg18")]
use pgrx::pg_sys;

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
    // SAFETY: ReadStream callbacks are invoked by PostgreSQL through the C
    // callback ABI; the guard converts Rust panics into PostgreSQL errors.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
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
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn linear_prefetch_cb(
    _stream: *mut pg_sys::ReadStream,
    callback_private_data: *mut std::ffi::c_void,
    per_buffer_data: *mut std::ffi::c_void,
) -> pg_sys::BlockNumber {
    // SAFETY: ReadStream callbacks are invoked by PostgreSQL through the C
    // callback ABI; the guard converts Rust panics into PostgreSQL errors.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
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
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn block_sequence_prefetch_cb(
    _stream: *mut pg_sys::ReadStream,
    callback_private_data: *mut std::ffi::c_void,
    per_buffer_data: *mut std::ffi::c_void,
) -> pg_sys::BlockNumber {
    // SAFETY: ReadStream callbacks are invoked by PostgreSQL through the C
    // callback ABI; the guard converts Rust panics into PostgreSQL errors.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if callback_private_data.is_null() {
                return pg_sys::InvalidBlockNumber;
            }
            // SAFETY: The block-sequence stream is registered with a
            // `BlockSequencePrefetchState` callback-private pointer that
            // outlives the callback invocation.
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
