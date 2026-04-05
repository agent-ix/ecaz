use std::ptr;

use pgrx::pg_sys;

use super::page;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphElement {
    pub tid: page::ItemPointer,
    pub level: u8,
    pub deleted: bool,
    pub heaptids: Vec<page::ItemPointer>,
    pub neighbortid: page::ItemPointer,
    pub code: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphNeighbors {
    pub tid: page::ItemPointer,
    pub count: usize,
    pub tids: Vec<page::ItemPointer>,
}

pub(crate) unsafe fn load_graph_element(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
) -> GraphElement {
    let tuple_bytes = unsafe { read_page_tuple_bytes(index_relation, element_tid, "element") };
    let element = page::TqElementTuple::decode(&tuple_bytes, code_len)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode graph element tuple: {e}"));
    GraphElement {
        tid: element_tid,
        level: element.level,
        deleted: element.deleted,
        heaptids: element.heaptids,
        neighbortid: element.neighbortid,
        code: element.code,
    }
}

pub(crate) unsafe fn load_graph_neighbors(
    index_relation: pg_sys::Relation,
    neighbor_tid: page::ItemPointer,
) -> GraphNeighbors {
    if neighbor_tid == page::ItemPointer::INVALID {
        return GraphNeighbors {
            tid: neighbor_tid,
            count: 0,
            tids: Vec::new(),
        };
    }

    let tuple_bytes = unsafe { read_page_tuple_bytes(index_relation, neighbor_tid, "neighbor") };
    let neighbor = page::TqNeighborTuple::decode(&tuple_bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode graph neighbor tuple: {e}"));
    let count = neighbor.count as usize;
    if count > neighbor.tids.len() {
        pgrx::error!(
            "tqhnsw neighbor tuple count {} exceeds payload tid count {}",
            neighbor.count,
            neighbor.tids.len()
        );
    }
    GraphNeighbors {
        tid: neighbor_tid,
        count,
        tids: neighbor.tids[..count].to_vec(),
    }
}

pub(crate) unsafe fn load_graph_adjacency(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
) -> (GraphElement, GraphNeighbors) {
    let element = unsafe { load_graph_element(index_relation, element_tid, code_len) };
    let neighbors = unsafe { load_graph_neighbors(index_relation, element.neighbortid) };
    (element, neighbors)
}

unsafe fn read_page_tuple_bytes(
    index_relation: pg_sys::Relation,
    tuple_tid: page::ItemPointer,
    tuple_kind: &str,
) -> Vec<u8> {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            tuple_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let line_pointer_count = super::page_line_pointer_count(page_ptr);
    if tuple_tid.offset_number == 0 || tuple_tid.offset_number > line_pointer_count {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        pgrx::error!(
            "tqhnsw graph read found {tuple_kind} tuple offset {} out of range on block {}",
            tuple_tid.offset_number,
            tuple_tid.block_number
        );
    }

    let item_id = unsafe { &*super::page_item_id(page_ptr, tuple_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        pgrx::error!("tqhnsw graph read found unused {tuple_kind} tuple slot");
    }

    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        pgrx::error!(
            "tqhnsw found invalid {tuple_kind} tuple bounds on block {}",
            tuple_tid.block_number
        );
    }

    let tuple_bytes =
        unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) }.to_vec();
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    tuple_bytes
}
