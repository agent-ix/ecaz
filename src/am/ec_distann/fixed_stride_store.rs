//! PostgreSQL raw-relation storage for Task 231 fixed-stride nodes.
//!
//! The relation is catalogued as an auxiliary heap only so PostgreSQL owns its
//! lifecycle. Its main fork is never interpreted through heapam: block zero is
//! an EFM1 admission record and every subsequent block is an EFS1 page written
//! through GenericXLog full-page images.

use pgrx::pg_sys;
use sha2::{Digest, Sha256};

#[cfg(feature = "pg_test")]
use super::fixed_stride::NODE_DIGEST_OFFSET;
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
const COMMITTED_PAGE_DIGEST_DOMAIN: &[u8] = b"ec_distann_fixed_stride_committed_pages_v1\0";
const PAGE_DIGEST_OFFSET: usize = 48;
const PAGE_DIGEST_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FixedStrideReadTelemetry {
    pub(crate) logical_blocks_touched: u32,
    pub(crate) logical_bytes_touched: u64,
    pub(crate) shared_buffer_hits: u64,
    pub(crate) shared_buffer_reads: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FixedStrideReadRequest {
    pub(crate) node_ordinal: u64,
    pub(crate) vec_id: u64,
}

pub(crate) fn read_nodes(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    requests: &[FixedStrideReadRequest],
    out: &mut Vec<FixedStrideNodeV1>,
) -> Result<FixedStrideReadTelemetry, String> {
    // PostgreSQL maintains this accounting per backend. Sampling it around
    // the bounded raw-relation operation attributes shared-buffer behavior to
    // fixed-stride reads without waiting for asynchronous pg_stat flushes.
    // This function stays on one PostgreSQL backend and never crosses an
    // await or thread boundary.
    let before = unsafe { pg_sys::pgBufferUsage };
    let mut telemetry = read_nodes_inner(relation, metadata, requests, out)?;
    let after = unsafe { pg_sys::pgBufferUsage };
    telemetry.shared_buffer_hits = after
        .shared_blks_hit
        .saturating_sub(before.shared_blks_hit)
        .try_into()
        .unwrap_or(0);
    telemetry.shared_buffer_reads = after
        .shared_blks_read
        .saturating_sub(before.shared_blks_read)
        .try_into()
        .unwrap_or(0);
    Ok(telemetry)
}

fn read_nodes_inner(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    requests: &[FixedStrideReadRequest],
    out: &mut Vec<FixedStrideNodeV1>,
) -> Result<FixedStrideReadTelemetry, String> {
    out.clear();
    if requests.is_empty() {
        return Ok(FixedStrideReadTelemetry {
            logical_blocks_touched: 0,
            logical_bytes_touched: 0,
            shared_buffer_hits: 0,
            shared_buffer_reads: 0,
        });
    }
    let full_verification = super::options::debug_fixed_stride_full_verification();
    let layout = &metadata.layout;
    if !layout.is_packed() {
        let mut blocks = 0_u32;
        let mut bytes = 0_u64;
        for request in requests {
            let mut node = FixedStrideNodeV1::empty();
            let telemetry = read_node_with_verification(
                relation,
                metadata,
                request.node_ordinal,
                request.vec_id,
                &mut node,
                full_verification,
            )?;
            blocks = blocks
                .checked_add(telemetry.logical_blocks_touched)
                .ok_or_else(|| {
                    "EC_FIXED_STRIDE_FORMAT: logical block touches overflow".to_owned()
                })?;
            bytes = bytes
                .checked_add(telemetry.logical_bytes_touched)
                .ok_or_else(|| {
                    "EC_FIXED_STRIDE_FORMAT: logical byte touches overflow".to_owned()
                })?;
            out.push(node);
        }
        return Ok(FixedStrideReadTelemetry {
            logical_blocks_touched: blocks,
            logical_bytes_touched: bytes,
            shared_buffer_hits: 0,
            shared_buffer_reads: 0,
        });
    }

    let mut ordered = requests
        .iter()
        .copied()
        .enumerate()
        .map(|(request_index, request)| {
            Ok::<_, String>((
                layout.address_admitted(request.node_ordinal)?.first_block,
                request.node_ordinal,
                request_index,
                request,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    ordered.sort_unstable_by_key(|(block, ordinal, _, _)| (*block, *ordinal));
    let mut decoded = (0..requests.len())
        .map(|_| None)
        .collect::<Vec<Option<FixedStrideNodeV1>>>();
    let mut cursor = 0;
    let mut blocks_touched = 0_u32;
    while cursor < ordered.len() {
        let block = ordered[cursor].0;
        let buffer = read_data_block(relation, layout, block)?;
        let (envelope, payload) = decode_locked_page(&buffer, metadata, full_verification)?;
        if envelope.kind != FixedStridePageKind::Packed {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: packed request reached a non-packed page".to_owned(),
            );
        }
        blocks_touched = blocks_touched
            .checked_add(1)
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: logical block touches overflow".to_owned())?;
        while cursor < ordered.len() && ordered[cursor].0 == block {
            let (_, _, request_index, request) = ordered[cursor];
            let address = layout.address_admitted(request.node_ordinal)?;
            if envelope.slot_count <= address.slot_index {
                return Err(
                    "EC_FIXED_STRIDE_FORMAT: packed node ordinal is unpublished on page".to_owned(),
                );
            }
            let start = usize::from(address.slot_index) * layout.node_stride_bytes as usize;
            let end = start + layout.node_stride_bytes as usize;
            let bytes = payload
                .get(start..end)
                .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: packed node bounds mismatch".to_owned())?;
            let mut node = FixedStrideNodeV1::empty();
            FixedStrideNodeV1::decode_into_admitted(
                bytes,
                layout,
                request.node_ordinal,
                request.vec_id,
                &mut node,
                full_verification,
            )?;
            decoded[request_index] = Some(node);
            cursor += 1;
        }
    }
    out.extend(
        decoded
            .into_iter()
            .map(|node| node.expect("every fixed-stride read request is decoded exactly once")),
    );
    Ok(FixedStrideReadTelemetry {
        logical_blocks_touched: blocks_touched,
        logical_bytes_touched: u64::from(blocks_touched) * u64::from(layout.block_size),
        shared_buffer_hits: 0,
        shared_buffer_reads: 0,
    })
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
        seal_raw_page_header(
            page.page_ptr(),
            usize::from(metadata.layout.pg_page_header_bytes),
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

/// Return the first never-written ordinal from raw relation state. Callers
/// must hold the generation's transaction-scoped mutation lock. Unlike an
/// MVCC directory aggregate, this remains authoritative under REPEATABLE READ
/// and SERIALIZABLE snapshots. A transaction that aborts after its raw WAL
/// write leaves an unreachable physical ordinal; subsequent writers append
/// after it rather than risking reuse across a crash boundary.
pub(crate) fn next_append_ordinal(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    published_ordinal_floor: u64,
) -> Result<u64, String> {
    let blocks = main_fork_block_count_handle(relation);
    if blocks == 0 {
        return Err("EC_FIXED_STRIDE_FORMAT: node store metadata block is missing".to_owned());
    }
    let data_blocks = blocks - 1;
    let layout = &metadata.layout;
    let next = if data_blocks == 0 {
        0
    } else if layout.is_packed() {
        let last_block = blocks - 1;
        let buffer = read_data_block(relation, layout, last_block)?;
        let (envelope, _) = decode_locked_page(&buffer, metadata, true)?;
        if envelope.kind != FixedStridePageKind::Packed {
            return Err("EC_FIXED_STRIDE_FORMAT: packed tail reached a non-packed page".to_owned());
        }
        let expected_base = u64::from(data_blocks - 1)
            .checked_mul(u64::from(layout.nodes_per_page))
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: packed tail ordinal overflow".to_owned())?;
        if envelope.base_ordinal != expected_base {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: packed tail page is not ordinal-contiguous".to_owned(),
            );
        }
        expected_base
            .checked_add(u64::from(envelope.slot_count))
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: packed tail ordinal overflow".to_owned())?
    } else {
        if data_blocks % layout.extent_blocks != 0 {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: multi-block tail ends in a partial extent".to_owned(),
            );
        }
        u64::from(data_blocks / layout.extent_blocks)
    };
    if next < published_ordinal_floor {
        return Err(format!(
            "EC_FIXED_STRIDE_FORMAT: raw tail ordinal {next} precedes Ready floor {published_ordinal_floor}"
        ));
    }
    Ok(next)
}

pub(crate) fn write_node(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    node: &FixedStrideNodeV1,
    unpublished_tail_floor: u64,
) -> Result<(), String> {
    if node.node_ordinal < unpublished_tail_floor {
        return Err(
            "EC_FIXED_STRIDE_FORMAT: refusing to rewrite a published node ordinal".to_owned(),
        );
    }
    let encoded = node.encode(&metadata.layout)?;
    let address = metadata.layout.address_admitted(node.node_ordinal)?;
    if metadata.layout.is_packed() {
        write_packed_node(
            relation,
            metadata,
            node.node_ordinal,
            address.slot_index,
            &encoded,
            unpublished_tail_floor,
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
    read_node_with_verification(
        relation,
        metadata,
        expected_ordinal,
        expected_vec_id,
        out,
        super::options::debug_fixed_stride_full_verification(),
    )
}

pub(crate) fn read_node_verified(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    expected_ordinal: u64,
    expected_vec_id: u64,
    out: &mut FixedStrideNodeV1,
) -> Result<FixedStrideReadTelemetry, String> {
    read_node_with_verification(
        relation,
        metadata,
        expected_ordinal,
        expected_vec_id,
        out,
        true,
    )
}

fn read_node_with_verification(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    expected_ordinal: u64,
    expected_vec_id: u64,
    out: &mut FixedStrideNodeV1,
    full_verification: bool,
) -> Result<FixedStrideReadTelemetry, String> {
    let layout = &metadata.layout;
    let address = layout.address_admitted(expected_ordinal)?;
    let mut encoded = Vec::with_capacity(layout.node_stride_bytes as usize);
    if layout.is_packed() {
        let buffer = read_data_block(relation, layout, address.first_block)?;
        let (envelope, payload) = decode_locked_page(&buffer, metadata, full_verification)?;
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
        FixedStrideNodeV1::decode_into_admitted(
            &encoded,
            layout,
            expected_ordinal,
            expected_vec_id,
            out,
            full_verification,
        )?;
        Ok(FixedStrideReadTelemetry {
            logical_blocks_touched: 1,
            logical_bytes_touched: u64::from(layout.block_size),
            shared_buffer_hits: 0,
            shared_buffer_reads: 0,
        })
    } else {
        for segment in 0..layout.extent_blocks {
            let block = address
                .first_block
                .checked_add(segment)
                .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: segment block overflow".to_owned())?;
            let buffer = read_data_block(relation, layout, block)?;
            let (envelope, payload) = decode_locked_page(&buffer, metadata, full_verification)?;
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
        FixedStrideNodeV1::decode_into_admitted(
            &encoded,
            layout,
            expected_ordinal,
            expected_vec_id,
            out,
            full_verification,
        )?;
        Ok(FixedStrideReadTelemetry {
            logical_blocks_touched: layout.extent_blocks,
            logical_bytes_touched: u64::from(layout.block_size) * u64::from(layout.extent_blocks),
            shared_buffer_hits: 0,
            shared_buffer_reads: 0,
        })
    }
}

/// Validate every page that contains a committed ordinal and bind its stored
/// EFS1 digest in physical-block order. The committed count is part of the
/// canonical stream so an otherwise identical prefix cannot admit a longer or
/// shorter generation. Unreachable blocks beyond the committed prefix are not
/// published and therefore do not participate in Ready admission.
pub(crate) fn committed_page_digest(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    committed_node_count: u64,
) -> Result<[u8; 32], String> {
    read_metadata(relation, metadata)?;
    let layout = &metadata.layout;
    let data_blocks = if layout.is_packed() {
        committed_node_count
            .checked_add(u64::from(layout.nodes_per_page) - 1)
            .map(|count| count / u64::from(layout.nodes_per_page))
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: committed page count overflow".to_owned())?
    } else {
        committed_node_count
            .checked_mul(u64::from(layout.extent_blocks))
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: committed extent count overflow".to_owned())?
    };
    let available = u64::from(main_fork_block_count_handle(relation));
    let required = data_blocks
        .checked_add(1)
        .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: committed block count overflow".to_owned())?;
    if available < required {
        return Err(format!(
            "EC_FIXED_STRIDE_FORMAT: node store has {available} blocks, needs {required} for {committed_node_count} committed nodes"
        ));
    }

    let mut hasher = Sha256::new();
    hasher.update(COMMITTED_PAGE_DIGEST_DOMAIN);
    hasher.update(committed_node_count.to_le_bytes());
    hasher.update(data_blocks.to_le_bytes());
    for data_index in 0..data_blocks {
        let block = u32::try_from(data_index + 1)
            .map_err(|_| "EC_FIXED_STRIDE_FORMAT: committed block number exceeds u32".to_owned())?;
        let buffer = read_data_block(relation, layout, block)?;
        let (envelope, payload) = decode_locked_page(&buffer, metadata, true)?;
        hasher.update(block.to_le_bytes());
        if layout.is_packed() {
            let expected_slots = if data_index + 1 == data_blocks {
                let remainder = committed_node_count % u64::from(layout.nodes_per_page);
                if remainder == 0 {
                    layout.nodes_per_page
                } else {
                    u16::try_from(remainder).expect("remainder is below nodes_per_page")
                }
            } else {
                layout.nodes_per_page
            };
            if envelope.slot_count < expected_slots {
                return Err(
                    "EC_FIXED_STRIDE_FORMAT: committed packed page omits a published ordinal"
                        .to_owned(),
                );
            }
            let committed_bytes = usize::from(expected_slots)
                .checked_mul(layout.node_stride_bytes as usize)
                .ok_or_else(|| {
                    "EC_FIXED_STRIDE_FORMAT: committed packed prefix overflow".to_owned()
                })?;
            let committed_prefix = payload.get(..committed_bytes).ok_or_else(|| {
                "EC_FIXED_STRIDE_FORMAT: committed packed prefix exceeds page payload".to_owned()
            })?;
            // Bind exactly the committed slot prefix. An aborted later batch
            // may leave unreachable raw slots because GenericXLog pages are
            // not MVCC-undone; those bytes must not perturb Ready evidence for
            // the same committed generation prefix.
            hasher.update(b"packed-prefix\0");
            hasher.update(expected_slots.to_le_bytes());
            hasher.update(committed_prefix);
        } else {
            let header = read_page_region(
                &buffer,
                usize::from(layout.pg_page_header_bytes),
                FIXED_STRIDE_PAGE_HEADER_BYTES,
            )?;
            hasher.update(b"multiblock-page\0");
            hasher.update(
                header
                    .get(PAGE_DIGEST_OFFSET..PAGE_DIGEST_OFFSET + PAGE_DIGEST_BYTES)
                    .ok_or_else(|| {
                        "EC_FIXED_STRIDE_FORMAT: page digest bounds mismatch".to_owned()
                    })?,
            );
        }
    }
    Ok(hasher.finalize().into())
}

fn write_packed_node(
    relation: RelationHandle,
    metadata: &FixedStrideMetadataV1,
    ordinal: u64,
    slot_index: u16,
    encoded_node: &[u8],
    unpublished_tail_floor: u64,
) -> Result<(), String> {
    let layout = &metadata.layout;
    let address = layout.address_admitted(ordinal)?;
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
        let (envelope, bytes) = decode_locked_page(&buffer, metadata, true)?;
        if envelope.kind != FixedStridePageKind::Packed
            || envelope.base_ordinal != base_ordinal
            || envelope.slot_count < slot_index
        {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: packed append is not the next or retry slot".to_owned(),
            );
        }
        if envelope.slot_count > slot_index.saturating_add(1) {
            let first_truncated_ordinal = ordinal.checked_add(1).ok_or_else(|| {
                "EC_FIXED_STRIDE_FORMAT: packed truncated ordinal overflow".to_owned()
            })?;
            if first_truncated_ordinal < unpublished_tail_floor {
                return Err(
                    "EC_FIXED_STRIDE_FORMAT: packed retry would truncate a published slot"
                        .to_owned(),
                );
            }
        }
        bytes.to_vec()
    };
    // Re-encoding a packed page deliberately re-hashes its already-published
    // prefix. Filling one page is therefore O(nodes_per_page^2) in digest
    // bytes, but nodes_per_page is a layout-bounded constant and this happens
    // only on the generation build path, never on query reads.
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
            let (existing, _) = decode_locked_page(&buffer, metadata, true)?;
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
    // Publication is single-writer: the generation row is held FOR UPDATE by
    // the staging transaction before this helper is reached. That row lock is
    // the extension lock for this private relation, so the block-count check
    // and P_NEW allocation cannot race another fixed-stride writer. If that
    // invariant is violated, the post-allocation block-number check below
    // fails closed (and may leave only an unreachable zero tail block).
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
    full_verification: bool,
) -> Result<(FixedStridePageEnvelopeV1, &'a [u8]), String> {
    let layout = &metadata.layout;
    ensure_page_size(buffer, layout)?;
    let header_offset = usize::from(layout.pg_page_header_bytes);
    let header = read_page_region(buffer, header_offset, FIXED_STRIDE_PAGE_HEADER_BYTES)?;
    let content_bytes = FixedStridePageEnvelopeV1::encoded_content_bytes(header)?;
    let payload = read_page_region(buffer, usize::from(layout.data_offset()?), content_bytes)?;
    let envelope = FixedStridePageEnvelopeV1::decode_admitted(
        header,
        payload,
        layout,
        &metadata.generation_tag,
        buffer.block_number(),
        full_verification,
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
        seal_raw_page_header(page.page_ptr(), usize::from(layout.pg_page_header_bytes))?;
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

fn seal_raw_page_header(page: pg_sys::Page, header_bytes: usize) -> Result<(), String> {
    let header_bytes = u16::try_from(header_bytes)
        .map_err(|_| "EC_FIXED_STRIDE_FORMAT: raw page header exceeds u16".to_owned())?;
    // This catalogued auxiliary relation is relkind='r', so database-wide
    // VACUUM/ANALYZE, anti-wraparound vacuum, pg_dump, or an explicit SELECT
    // may still enter heapam even though production code never does. Present
    // zero heap line pointers by keeping pd_lower at PageHeaderData. Set
    // pd_upper to the same value so PostgreSQL's REGBUF_STANDARD full-page
    // image has a zero-length pd_lower..pd_upper hole and therefore preserves
    // every raw EFM1/EFS1 byte after the page header.
    unsafe {
        let header = &mut *page.cast::<pg_sys::PageHeaderData>();
        if header_bytes > header.pd_special {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: raw page header exceeds special offset".to_owned(),
            );
        }
        header.pd_lower = header_bytes;
        header.pd_upper = header_bytes;
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

    fn xor_page_byte(relation: RelationHandle, block: u32, offset: usize) {
        let buffer = LockedBufferGuard::read_main_handle(
            relation,
            block,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .expect("corruption-test block should open");
        let mut wal = WalTxnScope::start_handle(relation);
        {
            let page = wal.register_page(&buffer);
            assert!(offset < buffer.page_size());
            unsafe {
                *page.page_ptr().cast::<u8>().add(offset) ^= 0x01;
            }
        }
        wal.finish();
    }

    fn assert_heapam_sees_empty(name: &str, relation: RelationHandle) {
        assert_eq!(
            Spi::get_one::<String>("SHOW data_checksums")
                .unwrap()
                .as_deref(),
            Some("on"),
            "fast fixed-stride verification requires PostgreSQL page checksums"
        );
        let blocks = main_fork_block_count_handle(relation);
        for block in 0..blocks {
            let buffer = LockedBufferGuard::read_main_handle(
                relation,
                block,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
            .expect("raw block should open for header audit");
            let header = unsafe { &*buffer.page().cast::<pg_sys::PageHeaderData>() };
            assert_eq!(header.pd_lower, header.pd_upper);
            assert_eq!(usize::from(header.pd_lower), unsafe {
                pg_sys::SizeOfPageHeaderData()
            });
        }
        assert_eq!(
            Spi::get_one::<i64>(&format!("SELECT count(*) FROM {name}")).unwrap(),
            Some(0),
            "heapam must see no fake line pointers in the raw relation"
        );
        Spi::run(&format!("ANALYZE {name}")).expect("heapam ANALYZE must safely see an empty heap");
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
            write_node(handle, &metadata, &node, 0).expect("node write should succeed");
        }
        if metadata.layout.is_packed() && count >= 3 {
            let first = sample_node(&metadata.layout, 0);
            let first_encoded = first.encode(&metadata.layout).unwrap();
            let page_guard = write_packed_node(handle, &metadata, 0, 0, &first_encoded, 2)
                .expect_err("page-level retry guard must protect published later slots");
            assert!(page_guard.contains("truncate a published slot"));

            let committed_prefix = count - 1;
            let clean_digest = committed_page_digest(handle, &metadata, committed_prefix).unwrap();
            let mut unreachable_tail = sample_node(&metadata.layout, committed_prefix);
            unreachable_tail.exact_vector[0] = 0.75;
            write_node(handle, &metadata, &unreachable_tail, committed_prefix)
                .expect("unreachable tail retry should succeed");
            assert_eq!(
                committed_page_digest(handle, &metadata, committed_prefix).unwrap(),
                clean_digest,
                "Ready digest must bind only the committed packed prefix"
            );
            write_node(
                handle,
                &metadata,
                &sample_node(&metadata.layout, committed_prefix),
                committed_prefix,
            )
            .expect("tail fixture should restore its canonical node");
        }
        let retry = sample_node(&metadata.layout, count - 1);
        write_node(handle, &metadata, &retry, 0).expect("tail retry should be idempotent");

        for ordinal in 0..count {
            let expected = sample_node(&metadata.layout, ordinal);
            let mut decoded = FixedStrideNodeV1::empty();
            let telemetry = read_node(handle, &metadata, ordinal, expected.vec_id, &mut decoded)
                .expect("node read should succeed");
            assert_eq!(decoded, expected);
            assert_eq!(
                telemetry.logical_blocks_touched,
                if metadata.layout.is_packed() {
                    1
                } else {
                    metadata.layout.extent_blocks
                }
            );
        }
        let requests = (0..count)
            .map(|ordinal| FixedStrideReadRequest {
                node_ordinal: ordinal,
                vec_id: sample_node(&metadata.layout, ordinal).vec_id,
            })
            .collect::<Vec<_>>();
        let mut batch = Vec::new();
        let batch_telemetry = read_nodes(handle, &metadata, &requests, &mut batch)
            .expect("batched node read should succeed");
        assert_eq!(batch.len(), count as usize);
        assert_eq!(
            batch_telemetry.logical_blocks_touched,
            if metadata.layout.is_packed() {
                1
            } else {
                metadata.layout.extent_blocks * count as u32
            }
        );
        assert_eq!(
            batch_telemetry.shared_buffer_hits + batch_telemetry.shared_buffer_reads,
            u64::from(batch_telemetry.logical_blocks_touched),
            "PostgreSQL BufferUsage must account for every fixed-stride block read"
        );
        let mut decoded = FixedStrideNodeV1::empty();
        assert!(read_node(handle, &metadata, 0, u64::MAX, &mut decoded).is_err());
        assert_heapam_sees_empty(name, handle);
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

    #[pgrx::pg_test]
    fn fixed_stride_store_relation_corruption_fails_closed() {
        let name = "ec_distann_fixed_stride_corruption_test";
        Spi::run(&format!(
            "CREATE TABLE {name} (__ecdz_raw bytea) WITH (autovacuum_enabled=false)"
        ))
        .unwrap();
        let relation = RelationGuard::try_open(
            relation_oid(name),
            pg_sys::AccessExclusiveLock as pg_sys::LOCKMODE,
        )
        .unwrap();
        let handle = NonNull::new(relation.as_ptr()).unwrap();
        let metadata = FixedStrideMetadataV1 {
            generation_tag: [0x55; 16],
            layout: DistannFixedStrideLayoutDescriptorV1::new(9, 4, 8).unwrap(),
        };
        initialize(handle, &metadata).unwrap();
        read_metadata(handle, &metadata).unwrap();
        let node = sample_node(&metadata.layout, 0);
        write_node(handle, &metadata, &node, 0).unwrap();

        let mut wrong_metadata = metadata.clone();
        wrong_metadata.generation_tag[0] ^= 1;
        assert!(read_metadata(handle, &wrong_metadata).is_err());
        let mut out = FixedStrideNodeV1::empty();
        assert!(read_node_verified(handle, &metadata, 1, 10_001, &mut out).is_err());
        assert!(write_node(
            handle,
            &metadata,
            &sample_node(
                &metadata.layout,
                u64::from(metadata.layout.nodes_per_page) * 2
            ),
            0,
        )
        .is_err());

        let page_header = usize::from(metadata.layout.pg_page_header_bytes);
        xor_page_byte(handle, 1, page_header + PAGE_DIGEST_OFFSET);
        assert!(read_node_verified(handle, &metadata, 0, node.vec_id, &mut out).is_err());
        xor_page_byte(handle, 1, page_header + PAGE_DIGEST_OFFSET);

        xor_page_byte(handle, 1, page_header + 32);
        assert!(read_node(handle, &metadata, 0, node.vec_id, &mut out).is_err());
        xor_page_byte(handle, 1, page_header + 32);

        let node_offset = usize::from(metadata.layout.data_offset().unwrap());
        xor_page_byte(handle, 1, node_offset + 4);
        assert!(read_node_verified(handle, &metadata, 0, node.vec_id, &mut out).is_err());
        xor_page_byte(handle, 1, node_offset + 4);

        xor_page_byte(handle, 1, node_offset + 16);
        assert!(read_node(handle, &metadata, 0, node.vec_id, &mut out).is_err());
        xor_page_byte(handle, 1, node_offset + 16);

        xor_page_byte(handle, 1, node_offset + NODE_DIGEST_OFFSET);
        assert!(read_node_verified(handle, &metadata, 0, node.vec_id, &mut out).is_err());
        xor_page_byte(handle, 1, node_offset + NODE_DIGEST_OFFSET);

        let padding_offset = node_offset + metadata.layout.node_stride_bytes as usize - 1;
        xor_page_byte(handle, 1, padding_offset);
        assert!(read_node_verified(handle, &metadata, 0, node.vec_id, &mut out).is_err());
        xor_page_byte(handle, 1, padding_offset);
        assert_heapam_sees_empty(name, handle);

        let multi_name = "ec_distann_fixed_stride_truncated_test";
        Spi::run(&format!(
            "CREATE TABLE {multi_name} (__ecdz_raw bytea) WITH (autovacuum_enabled=false)"
        ))
        .unwrap();
        let multi_relation = RelationGuard::try_open(
            relation_oid(multi_name),
            pg_sys::AccessExclusiveLock as pg_sys::LOCKMODE,
        )
        .unwrap();
        let multi_handle = NonNull::new(multi_relation.as_ptr()).unwrap();
        let multi_metadata = FixedStrideMetadataV1 {
            generation_tag: [0x66; 16],
            layout: DistannFixedStrideLayoutDescriptorV1::new(2048, 2, 8).unwrap(),
        };
        initialize(multi_handle, &multi_metadata).unwrap();
        read_metadata(multi_handle, &multi_metadata).unwrap();
        let multi_node = sample_node(&multi_metadata.layout, 0);
        write_node(multi_handle, &multi_metadata, &multi_node, 0).unwrap();
        let blocks = main_fork_block_count_handle(multi_handle);
        assert!(blocks > 2);
        unsafe { pg_sys::RelationTruncate(multi_relation.as_ptr(), blocks - 1) };
        assert!(read_node_verified(
            multi_handle,
            &multi_metadata,
            0,
            multi_node.vec_id,
            &mut out,
        )
        .is_err());
    }
}
