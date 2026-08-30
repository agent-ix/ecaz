//! PostgreSQL raw-relation storage for Task 231 fixed-stride nodes.
//!
//! The relation is catalogued as an auxiliary heap only so PostgreSQL owns its
//! lifecycle. Its main fork is never interpreted through heapam: block zero is
//! an EFM1 admission record and every subsequent block is an EFS1 page written
//! through GenericXLog full-page images.

use pgrx::pg_sys;

use super::fixed_stride::{
    DistannFixedStrideLayoutDescriptorV1, FixedStrideMetadataV1, FixedStrideNodeV1,
    FixedStridePageEnvelopeV1, FixedStridePageKind, FIXED_STRIDE_METADATA_BYTES,
    FIXED_STRIDE_PAGE_HEADER_BYTES,
};
use crate::storage::{
    buffer_guard::LockedBufferGuard,
    relation::{main_fork_block_count_handle, RelationHandle},
    wal::WalTxnScope,
};

const P_NEW: pg_sys::BlockNumber = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FixedStrideReadTelemetry {
    pub(crate) blocks_read: u32,
    pub(crate) bytes_read: u64,
}

pub(crate) fn initialize(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
) -> Result<(), String> {
    metadata.layout.validate()?;
    if main_fork_block_count_handle(relation) != 0 {
        return Err(
            "EC_FIXED_STRIDE_FORMAT: node store initialization requires an empty relation"
                .to_owned(),
        );
    }
    let buffer = LockedBufferGuard::read_main_locked_handle(
        relation,
        P_NEW,
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
    )
    .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: failed to allocate metadata block".to_owned())?;
    if buffer.block_number() != 0 {
        return Err(
            "EC_FIXED_STRIDE_FORMAT: metadata allocation did not return block zero".to_owned(),
        );
    }
    ensure_page_size(&buffer, &metadata.layout)?;
    let encoded = metadata.encode()?;
    let mut wal = WalTxnScope::start_handle(relation);
    {
        let mut page = wal.register_page(&buffer);
        page.init(0);
        write_page_region(
            page.page_ptr(),
            buffer.page_size(),
            usize::from(metadata.layout.pg_page_header_bytes),
            &encoded,
        )?;
        set_page_lower(
            page.page_ptr(),
            usize::from(metadata.layout.pg_page_header_bytes) + encoded.len(),
        )?;
    }
    wal.finish();
    Ok(())
}

pub(crate) fn read_metadata(
    relation: RelationHandle,
    expected: &FixedStrideMetadataV1,
) -> Result<FixedStrideMetadataV1, String> {
    if main_fork_block_count_handle(relation) == 0 {
        return Err("EC_FIXED_STRIDE_FORMAT: node store metadata block is missing".to_owned());
    }
    let buffer = LockedBufferGuard::read_main_handle(
        relation,
        0,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: failed to read metadata block".to_owned())?;
    ensure_page_size(&buffer, &expected.layout)?;
    let offset = usize::from(expected.layout.pg_page_header_bytes);
    let encoded = read_page_region(&buffer, offset, FIXED_STRIDE_METADATA_BYTES)?;
    let admitted = FixedStrideMetadataV1::decode(encoded)?;
    if &admitted != expected {
        return Err("EC_FIXED_STRIDE_FORMAT: node store metadata admission mismatch".to_owned());
    }
    Ok(admitted)
}

pub(crate) fn write_node(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    node: &FixedStrideNodeV1,
) -> Result<(), String> {
    read_metadata(relation, metadata)?;
    let encoded = node.encode(&metadata.layout)?;
    let address = metadata.layout.address(node.node_ordinal)?;
    if metadata.layout.is_packed() {
        write_packed_node(
            relation,
            metadata,
            node.node_ordinal,
            address.slot_index,
            &encoded,
        )
    } else {
        write_multiblock_node(
            relation,
            metadata,
            node.node_ordinal,
            address.first_block,
            &encoded,
        )
    }
}

pub(crate) fn read_node(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    expected_ordinal: u64,
    expected_vec_id: u64,
    out: &mut FixedStrideNodeV1,
) -> Result<FixedStrideReadTelemetry, String> {
    read_metadata(relation, metadata)?;
    let layout = &metadata.layout;
    let address = layout.address(expected_ordinal)?;
    let mut encoded = Vec::with_capacity(layout.node_stride_bytes as usize);
    if layout.is_packed() {
        let buffer = read_data_block(relation, layout, address.first_block)?;
        let (envelope, payload) = decode_locked_page(&buffer, metadata)?;
        if envelope.kind != FixedStridePageKind::Packed || envelope.slot_count <= address.slot_index
        {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: packed node ordinal is unpublished on page".to_owned(),
            );
        }
        let start = usize::from(address.slot_index) * layout.node_stride_bytes as usize;
        let end = start + layout.node_stride_bytes as usize;
        encoded.extend_from_slice(
            payload
                .get(start..end)
                .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: packed node bounds mismatch".to_owned())?,
        );
        FixedStrideNodeV1::decode_into(&encoded, layout, expected_ordinal, expected_vec_id, out)?;
        Ok(FixedStrideReadTelemetry {
            blocks_read: 1,
            bytes_read: u64::from(layout.block_size),
        })
    } else {
        for segment in 0..layout.extent_blocks {
            let block = address
                .first_block
                .checked_add(segment)
                .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: segment block overflow".to_owned())?;
            let buffer = read_data_block(relation, layout, block)?;
            let (envelope, payload) = decode_locked_page(&buffer, metadata)?;
            if envelope.kind != FixedStridePageKind::MultiBlock
                || u32::from(envelope.segment_index) != segment
                || envelope.base_ordinal != expected_ordinal
            {
                return Err("EC_FIXED_STRIDE_FORMAT: multi-block node segment mismatch".to_owned());
            }
            encoded.extend_from_slice(payload);
        }
        if encoded.len() != layout.node_stride_bytes as usize {
            return Err("EC_FIXED_STRIDE_FORMAT: multi-block node length mismatch".to_owned());
        }
        FixedStrideNodeV1::decode_into(&encoded, layout, expected_ordinal, expected_vec_id, out)?;
        Ok(FixedStrideReadTelemetry {
            blocks_read: layout.extent_blocks,
            bytes_read: u64::from(layout.block_size) * u64::from(layout.extent_blocks),
        })
    }
}

fn write_packed_node(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    ordinal: u64,
    slot_index: u16,
    encoded_node: &[u8],
) -> Result<(), String> {
    let layout = &metadata.layout;
    let address = layout.address(ordinal)?;
    let (buffer, is_new) = open_data_block_for_write(relation, layout, address.first_block)?;
    let base_ordinal = ordinal - u64::from(slot_index);
    let stride = layout.node_stride_bytes as usize;
    let mut payload = if is_new {
        if slot_index != 0 {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: new packed page starts after slot zero".to_owned(),
            );
        }
        Vec::new()
    } else {
        let (envelope, bytes) = decode_locked_page(&buffer, metadata)?;
        if envelope.kind != FixedStridePageKind::Packed
            || envelope.base_ordinal != base_ordinal
            || !(envelope.slot_count == slot_index
                || envelope.slot_count == slot_index.saturating_add(1))
        {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: packed append is not the next or retry slot".to_owned(),
            );
        }
        bytes.to_vec()
    };
    let required = usize::from(slot_index)
        .checked_add(1)
        .and_then(|slots| slots.checked_mul(stride))
        .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: packed payload length overflow".to_owned())?;
    payload.resize(required, 0);
    let start = usize::from(slot_index) * stride;
    payload[start..start + stride].copy_from_slice(encoded_node);
    let slot_count = slot_index
        .checked_add(1)
        .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: packed slot count overflow".to_owned())?;
    let envelope = FixedStridePageEnvelopeV1 {
        kind: FixedStridePageKind::Packed,
        base_ordinal,
        slot_count,
        segment_index: 0,
        segment_count: 1,
        content_bytes: u16::try_from(payload.len())
            .map_err(|_| "EC_FIXED_STRIDE_FORMAT: packed payload exceeds u16".to_owned())?,
        generation_tag: metadata.generation_tag,
    };
    write_data_page(relation, layout, &buffer, is_new, &envelope, &payload)
}

fn write_multiblock_node(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    ordinal: u64,
    first_block: u32,
    encoded_node: &[u8],
) -> Result<(), String> {
    let layout = &metadata.layout;
    let segment_count = u16::try_from(layout.extent_blocks)
        .map_err(|_| "EC_FIXED_STRIDE_FORMAT: extent count exceeds u16".to_owned())?;
    for segment in 0..layout.extent_blocks {
        let block = first_block
            .checked_add(segment)
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: segment block overflow".to_owned())?;
        let start = segment as usize * layout.page_payload_bytes as usize;
        let end = (start + layout.page_payload_bytes as usize).min(encoded_node.len());
        let payload = &encoded_node[start..end];
        let (buffer, is_new) = open_data_block_for_write(relation, layout, block)?;
        if !is_new {
            let (existing, _) = decode_locked_page(&buffer, metadata)?;
            if existing.kind != FixedStridePageKind::MultiBlock
                || existing.base_ordinal != ordinal
                || u32::from(existing.segment_index) != segment
            {
                return Err(
                    "EC_FIXED_STRIDE_FORMAT: multi-block retry identity mismatch".to_owned(),
                );
            }
        }
        let envelope = FixedStridePageEnvelopeV1 {
            kind: FixedStridePageKind::MultiBlock,
            base_ordinal: ordinal,
            slot_count: 1,
            segment_index: u16::try_from(segment)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: segment index exceeds u16".to_owned())?,
            segment_count,
            content_bytes: u16::try_from(payload.len())
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: segment payload exceeds u16".to_owned())?,
            generation_tag: metadata.generation_tag,
        };
        write_data_page(relation, layout, &buffer, is_new, &envelope, payload)?;
    }
    Ok(())
}

fn open_data_block_for_write(
    relation: RelationHandle,
    layout: &DistannFixedStrideLayoutDescriptorV1,
    target: u32,
) -> Result<(LockedBufferGuard, bool), String> {
    let blocks = main_fork_block_count_handle(relation);
    if blocks < target {
        return Err(format!(
            "EC_FIXED_STRIDE_FORMAT: node store has a block gap before target {target}"
        ));
    }
    let is_new = blocks == target;
    let buffer = if is_new {
        LockedBufferGuard::read_main_locked_handle(
            relation,
            P_NEW,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
        )
    } else {
        LockedBufferGuard::read_main_handle(
            relation,
            target,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .ok_or_else(|| format!("EC_FIXED_STRIDE_FORMAT: failed to open data block {target}"))?;
    if buffer.block_number() != target {
        return Err(format!(
            "EC_FIXED_STRIDE_FORMAT: allocated block {}, expected {target}",
            buffer.block_number()
        ));
    }
    ensure_page_size(&buffer, layout)?;
    Ok((buffer, is_new))
}

fn read_data_block(
    relation: RelationHandle,
    layout: &DistannFixedStrideLayoutDescriptorV1,
    block: u32,
) -> Result<LockedBufferGuard, String> {
    if block >= main_fork_block_count_handle(relation) {
        return Err(format!(
            "EC_FIXED_STRIDE_FORMAT: data block {block} is missing"
        ));
    }
    let buffer = LockedBufferGuard::read_main_handle(
        relation,
        block,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .ok_or_else(|| format!("EC_FIXED_STRIDE_FORMAT: failed to read data block {block}"))?;
    ensure_page_size(&buffer, layout)?;
    Ok(buffer)
}

fn decode_locked_page<'a>(
    buffer: &'a LockedBufferGuard,
    metadata: &FixedStrideMetadataV1,
) -> Result<(FixedStridePageEnvelopeV1, &'a [u8]), String> {
    let layout = &metadata.layout;
    ensure_page_size(buffer, layout)?;
    let header_offset = usize::from(layout.pg_page_header_bytes);
    let header = read_page_region(buffer, header_offset, FIXED_STRIDE_PAGE_HEADER_BYTES)?;
    let content_bytes = FixedStridePageEnvelopeV1::encoded_content_bytes(header)?;
    let payload = read_page_region(buffer, usize::from(layout.data_offset()?), content_bytes)?;
    let envelope = FixedStridePageEnvelopeV1::decode(
        header,
        payload,
        layout,
        &metadata.generation_tag,
        buffer.block_number(),
    )?;
    Ok((envelope, payload))
}

fn write_data_page(
    relation: RelationHandle,
    layout: &DistannFixedStrideLayoutDescriptorV1,
    buffer: &LockedBufferGuard,
    is_new: bool,
    envelope: &FixedStridePageEnvelopeV1,
    payload: &[u8],
) -> Result<(), String> {
    let header = envelope.encode(layout, payload, buffer.block_number())?;
    let mut wal = WalTxnScope::start_handle(relation);
    {
        let mut page = wal.register_page(buffer);
        if is_new {
            page.init(0);
        }
        write_page_region(
            page.page_ptr(),
            buffer.page_size(),
            usize::from(layout.pg_page_header_bytes),
            &header,
        )?;
        write_page_region(
            page.page_ptr(),
            buffer.page_size(),
            usize::from(layout.data_offset()?),
            payload,
        )?;
        set_page_lower(
            page.page_ptr(),
            usize::from(layout.data_offset()?) + payload.len(),
        )?;
    }
    wal.finish();
    Ok(())
}

fn ensure_page_size(
    buffer: &LockedBufferGuard,
    layout: &DistannFixedStrideLayoutDescriptorV1,
) -> Result<(), String> {
    if buffer.page_size() != layout.block_size as usize {
        return Err(format!(
            "EC_FIXED_STRIDE_FORMAT: runtime page size {} does not match admitted {}",
            buffer.page_size(),
            layout.block_size
        ));
    }
    Ok(())
}

fn read_page_region(
    buffer: &LockedBufferGuard,
    offset: usize,
    len: usize,
) -> Result<&[u8], String> {
    let end = offset
        .checked_add(len)
        .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: page region overflow".to_owned())?;
    if end > buffer.page_size() {
        return Err("EC_FIXED_STRIDE_FORMAT: page region exceeds buffer bounds".to_owned());
    }
    // SAFETY: the caller owns a locked, pinned buffer for the duration of the
    // returned borrow. Bounds were checked against that buffer's page size.
    Ok(unsafe { std::slice::from_raw_parts(buffer.page().cast::<u8>().add(offset), len) })
}

fn write_page_region(
    page: pg_sys::Page,
    page_size: usize,
    offset: usize,
    bytes: &[u8],
) -> Result<(), String> {
    let end = offset
        .checked_add(bytes.len())
        .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: page write region overflow".to_owned())?;
    if end > page_size {
        return Err("EC_FIXED_STRIDE_FORMAT: page write exceeds buffer bounds".to_owned());
    }
    // SAFETY: the page is the writable image registered with GenericXLog;
    // bounds were checked against its owning buffer and the regions do not
    // overlap the source slices used by callers.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), page.cast::<u8>().add(offset), bytes.len());
    }
    Ok(())
}

fn set_page_lower(page: pg_sys::Page, lower: usize) -> Result<(), String> {
    let lower = u16::try_from(lower)
        .map_err(|_| "EC_FIXED_STRIDE_FORMAT: raw page lower bound exceeds u16".to_owned())?;
    // PostgreSQL WAL treats pd_lower..pd_upper as the reusable page hole and
    // may omit it from full-page images. The raw EFM1/EFS1 bytes begin directly
    // after PageHeaderData, so advance pd_lower through the initialized bytes
    // to make them durable. Heapam never opens this auxiliary relation.
    unsafe {
        let header = &mut *page.cast::<pg_sys::PageHeaderData>();
        if lower > header.pd_special {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: raw page lower bound exceeds special offset".to_owned(),
            );
        }
        header.pd_lower = lower;
    }
    Ok(())
}

#[cfg(feature = "pg_test")]
#[pgrx::pg_schema]
mod tests {
    use std::ptr::NonNull;

    use pgrx::{pg_sys, Spi};

    use super::*;
    use crate::storage::{page::ItemPointer, relation_guard::RelationGuard};

    fn relation_oid(name: &str) -> pg_sys::Oid {
        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{name}'::regclass::oid"))
            .expect("relation OID query should succeed")
            .expect("relation OID should be present")
    }

    fn sample_node(
        layout: &DistannFixedStrideLayoutDescriptorV1,
        ordinal: u64,
    ) -> FixedStrideNodeV1 {
        FixedStrideNodeV1 {
            tombstoned: false,
            node_ordinal: ordinal,
            vec_id: 10_000 + ordinal,
            row_tid: ItemPointer {
                block_number: u32::try_from(ordinal + 1).expect("test ordinal fits block number"),
                offset_number: 1,
            },
            neighbor_count: 0,
            exact_vector: vec![0.25; usize::from(layout.dimensions)],
            search_code: vec![0x5a; layout.code_len as usize],
            neighbor_vec_ids: vec![0; usize::from(layout.graph_degree)],
            neighbor_codes: vec![0; usize::from(layout.graph_degree) * layout.code_len as usize],
        }
    }

    fn exercise_relation(name: &str, layout: DistannFixedStrideLayoutDescriptorV1, count: u64) {
        Spi::run(&format!(
            "CREATE TABLE {name} (__ecdz_raw bytea) WITH (autovacuum_enabled=false)"
        ))
        .expect("raw relation should be created");
        let relation = RelationGuard::try_open(
            relation_oid(name),
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        )
        .expect("raw relation should open");
        assert_eq!(relation.std_rd_options_autovacuum_enabled(), Some(false));
        let handle = NonNull::new(relation.as_ptr()).expect("relation pointer should be non-null");
        let metadata = FixedStrideMetadataV1 {
            generation_tag: [0x33; 16],
            layout,
        };
        initialize(handle, &metadata).expect("metadata initialization should succeed");
        read_metadata(handle, &metadata).expect("metadata should admit");

        for ordinal in 0..count {
            let node = sample_node(&metadata.layout, ordinal);
            write_node(handle, &metadata, &node).expect("node write should succeed");
        }
        let retry = sample_node(&metadata.layout, count - 1);
        write_node(handle, &metadata, &retry).expect("tail retry should be idempotent");

        for ordinal in 0..count {
            let expected = sample_node(&metadata.layout, ordinal);
            let mut decoded = FixedStrideNodeV1::empty();
            let telemetry = read_node(handle, &metadata, ordinal, expected.vec_id, &mut decoded)
                .expect("node read should succeed");
            assert_eq!(decoded, expected);
            assert_eq!(
                telemetry.blocks_read,
                if metadata.layout.is_packed() {
                    1
                } else {
                    metadata.layout.extent_blocks
                }
            );
        }
        let mut decoded = FixedStrideNodeV1::empty();
        assert!(read_node(handle, &metadata, 0, u64::MAX, &mut decoded).is_err());
    }

    #[pgrx::pg_test]
    fn fixed_stride_store_round_trips_packed_and_multiblock_nodes() {
        exercise_relation(
            "ec_distann_fixed_stride_packed_test",
            DistannFixedStrideLayoutDescriptorV1::new(8, 4, 8).unwrap(),
            3,
        );
        exercise_relation(
            "ec_distann_fixed_stride_multiblock_test",
            DistannFixedStrideLayoutDescriptorV1::new(2048, 2, 8).unwrap(),
            2,
        );
    }
}
