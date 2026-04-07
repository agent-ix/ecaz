#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphPrefetchState {
    blocks: Vec<u32>,
    index: usize,
}

impl GraphPrefetchState {
    pub(crate) fn new(blocks: Vec<u32>) -> Self {
        Self { blocks, index: 0 }
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

pub(crate) fn stream_snapshot() -> ReadStreamSnapshot {
    let graph = graph_callback_signature();
    let linear = linear_callback_signature();
    ReadStreamSnapshot {
        graph_stream_mode: graph.stream_mode,
        linear_stream_mode: linear.stream_mode,
        graph_stream_access_pattern: graph.access_pattern,
        linear_stream_access_pattern: linear.access_pattern,
        pg18_callback_surface_ready: false,
        pg18_scan_wiring_ready: false,
        pg18_vacuum_wiring_ready: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        graph_callback_signature, graph_prefetch_callback, linear_callback_signature,
        linear_prefetch_callback, stream_snapshot, GraphPrefetchState, LinearPrefetchState,
        ReadStreamCallbackResult, ReadStreamCallbackSignature, ReadStreamSnapshot,
    };

    #[test]
    fn stream_snapshot_stays_explicitly_unwired_until_pg18_support_exists() {
        assert_eq!(
            stream_snapshot(),
            ReadStreamSnapshot {
                graph_stream_mode: "READ_STREAM_DEFAULT",
                linear_stream_mode: "READ_STREAM_SEQUENTIAL",
                graph_stream_access_pattern: "random",
                linear_stream_access_pattern: "sequential",
                pg18_callback_surface_ready: false,
                pg18_scan_wiring_ready: false,
                pg18_vacuum_wiring_ready: false,
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
