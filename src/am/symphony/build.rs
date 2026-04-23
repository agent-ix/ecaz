use std::{ffi::CStr, ptr};

use pgrx::{itemptr::item_pointer_get_both, pg_sys, FromDatum, PgBox, PgTupleDesc};
use rand::random;

use super::{options, page};
use crate::am::common::metadata;

const POPULATED_BUILD_ERROR: &str =
    "symphony ambuild for populated relations is not implemented yet";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndexedVectorKind {
    Ecvector,
    Tqvector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IndexedVectorAttribute {
    attnum: i32,
    typoid: pg_sys::Oid,
    kind: IndexedVectorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SingletonBuildTuple {
    heap_tid: page::ItemPointer,
    dimensions: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildInput {
    Empty,
    Singleton(SingletonBuildTuple),
    MultiRow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BuildScanResult {
    heap_tuples: f64,
    input: BuildInput,
}

pub(super) unsafe extern "C-unwind" fn symphony_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let build_input = scan_heap_for_build_input(heap_relation, index_info);
            let options = options::relation_options(index_relation);
            let mut metadata = initial_metadata(options, random::<u64>());
            let index_tuples = match build_input.input {
                BuildInput::Empty => {
                    metadata::initialize_metadata_page(
                        index_relation,
                        &metadata.encode(),
                        "symphony",
                    );
                    0.0
                }
                BuildInput::Singleton(tuple) => {
                    metadata::initialize_metadata_page(
                        index_relation,
                        &metadata.encode(),
                        "symphony",
                    );
                    let entry_point =
                        write_singleton_build(index_relation, pg_sys::BLCKSZ as usize, tuple)
                            .unwrap_or_else(|err| pgrx::error!("{err}"));
                    metadata.entry_point = entry_point;
                    metadata.dimensions = tuple.dimensions;
                    metadata::initialize_metadata_page(
                        index_relation,
                        &metadata.encode(),
                        "symphony",
                    );
                    1.0
                }
                BuildInput::MultiRow => pgrx::error!("{POPULATED_BUILD_ERROR}"),
            };

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = build_input.heap_tuples;
            result.index_tuples = index_tuples;
            result.into_pg()
        })
    }
}

pub(super) unsafe extern "C-unwind" fn symphony_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            write_initial_metadata(index_relation);
        })
    }
}

fn initial_metadata(options: options::SymphonyOptions, seed: u64) -> page::MetadataPage {
    page::MetadataPage::current_v5_symphony(page::CurrentFormatMetadata {
        m: u16::try_from(options.m).expect("validated Symphony m should fit into u16"),
        ef_construction: u16::try_from(options.ef_construction)
            .expect("validated Symphony ef_construction should fit into u16"),
        entry_point: page::ItemPointer::INVALID,
        dimensions: 0,
        rabitq_bits: super::SYMPHONY_RABITQ_BITS,
        max_level: 0,
        seed,
        inserted_since_rebuild: 0,
        // The Phase-0 oracle keeps padding disabled until the padded
        // graph builder lands.
        padding_factor: u16::try_from(options.padding_factor)
            .expect("validated Symphony padding_factor should fit into u16"),
    })
}

unsafe fn write_initial_metadata(index_relation: pg_sys::Relation) {
    let options = unsafe { options::relation_options(index_relation) };
    let metadata = initial_metadata(options, random::<u64>());
    let encoded = metadata.encode();
    unsafe { metadata::initialize_metadata_page(index_relation, &encoded, "symphony") };
}

unsafe fn write_singleton_build(
    index_relation: pg_sys::Relation,
    page_size: usize,
    tuple: SingletonBuildTuple,
) -> Result<page::ItemPointer, String> {
    let (data_pages, entry_point) = singleton_data_pages(page_size, tuple.heap_tid)?;
    unsafe { write_data_pages(index_relation, &data_pages) };
    Ok(entry_point)
}

fn singleton_data_pages(
    page_size: usize,
    heap_tid: page::ItemPointer,
) -> Result<(page::DataPageChain, page::ItemPointer), String> {
    let mut data_pages = page::DataPageChain::new(page_size);
    let neighbor_tid = data_pages.insert_symphony_neighbor(&page::SymphonyNeighborTuple {
        count: 0,
        tids: Vec::new(),
        centered_codes: Vec::new(),
    })?;
    let entry_point = data_pages.insert_symphony_element(&page::SymphonyElementTuple {
        level: 0,
        deleted: false,
        heaptids: vec![heap_tid],
        neighbortid: neighbor_tid,
    })?;
    Ok((data_pages, entry_point))
}

unsafe fn write_data_pages(index_relation: pg_sys::Relation, data_pages: &page::DataPageChain) {
    for staged_page in data_pages.pages() {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                u32::MAX,
                pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            pgrx::error!(
                "symphony failed to allocate data buffer for block {}",
                staged_page.block_number()
            );
        }

        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let mut wal_txn = unsafe { crate::storage::wal::GenericXLogTxn::start(index_relation) };
        let page_ptr =
            unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };

        for tuple in staged_page.tuples() {
            let offset = unsafe {
                pg_sys::PageAddItemExtended(
                    page_ptr,
                    tuple.as_ptr().cast_mut().cast(),
                    tuple.len(),
                    pg_sys::InvalidOffsetNumber,
                    0,
                )
            };
            if offset == pg_sys::InvalidOffsetNumber {
                pgrx::error!(
                    "symphony failed to write tuple to block {}",
                    staged_page.block_number()
                );
            }
        }

        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }
}

unsafe fn scan_heap_for_build_input(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> BuildScanResult {
    let indexed_attribute =
        unsafe { resolve_indexed_vector_attribute_from_index_info(heap_relation, index_info) };
    let slot = unsafe {
        pg_sys::MakeSingleTupleTableSlot(
            (*heap_relation).rd_att,
            pg_sys::table_slot_callbacks(heap_relation),
        )
    };
    if slot.is_null() {
        pgrx::error!("symphony ambuild failed to allocate heap scan slot");
    }

    let snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
    unsafe { pg_sys::PushActiveSnapshot(snapshot) };
    let scan = unsafe {
        pg_sys::heap_beginscan(
            heap_relation,
            snapshot,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            pg_sys::ScanOptions::SO_TYPE_SEQSCAN
                | pg_sys::ScanOptions::SO_ALLOW_PAGEMODE
                | pg_sys::ScanOptions::SO_ALLOW_STRAT
                | pg_sys::ScanOptions::SO_ALLOW_SYNC,
        )
    };
    if scan.is_null() {
        unsafe {
            pg_sys::PopActiveSnapshot();
            pg_sys::UnregisterSnapshot(snapshot);
            pg_sys::ExecDropSingleTupleTableSlot(slot);
        }
        pgrx::error!("symphony ambuild failed to begin heap scan");
    }

    let mut heap_tuples = 0.0_f64;
    let mut singleton = None;
    let result = loop {
        if !unsafe {
            pg_sys::heap_getnextslot(scan, pg_sys::ScanDirection::ForwardScanDirection, slot)
        } {
            break singleton
                .map(BuildInput::Singleton)
                .unwrap_or(BuildInput::Empty);
        }

        heap_tuples += 1.0;
        if singleton.is_some() {
            break BuildInput::MultiRow;
        }

        let heap_tid = unsafe { decode_slot_tid(slot) };
        let datum =
            unsafe { required_slot_datum(slot, indexed_attribute.attnum, "indexed column") };
        let dimensions = vector_dimensions_from_datum(datum, indexed_attribute);
        singleton = Some(SingletonBuildTuple {
            heap_tid,
            dimensions,
        });
    };

    unsafe {
        pg_sys::heap_endscan(scan);
        pg_sys::PopActiveSnapshot();
        pg_sys::UnregisterSnapshot(snapshot);
        pg_sys::ExecDropSingleTupleTableSlot(slot);
    }

    BuildScanResult {
        heap_tuples,
        input: result,
    }
}

unsafe fn resolve_indexed_vector_attribute_from_index_info(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> IndexedVectorAttribute {
    if index_info.is_null() {
        pgrx::error!("symphony ambuild received a null IndexInfo");
    }
    let index_info = unsafe { &*index_info };
    if index_info.ii_NumIndexAttrs != 1 || index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("symphony ambuild currently supports single-column indexes only");
    }
    if !index_info.ii_Expressions.is_null() {
        pgrx::error!("symphony ambuild does not support expression indexes yet");
    }
    if !index_info.ii_Predicate.is_null() {
        pgrx::error!("symphony ambuild does not support partial indexes yet");
    }

    let attnum = i32::from(index_info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("symphony ambuild requires a base heap column index key");
    }

    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let attribute = tuple_desc
        .get(attnum as usize - 1)
        .expect("resolved Symphony indexed attribute should exist");
    if attribute.attisdropped {
        pgrx::error!("symphony ambuild references a dropped indexed column");
    }

    let kind = unsafe { resolve_indexed_vector_kind(attribute.atttypid) }.unwrap_or_else(|| {
        pgrx::error!("symphony ambuild requires an ecvector or tqvector column")
    });
    IndexedVectorAttribute {
        attnum,
        typoid: attribute.atttypid,
        kind,
    }
}

unsafe fn resolve_indexed_vector_kind(type_oid: pg_sys::Oid) -> Option<IndexedVectorKind> {
    let base_type_oid = unsafe { pg_sys::getBaseType(type_oid) };
    let formatted = unsafe { pg_sys::format_type_be(base_type_oid) };
    if formatted.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(formatted) }
        .to_string_lossy()
        .into_owned();
    unsafe { pg_sys::pfree(formatted.cast()) };
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    match type_name {
        "ecvector" => Some(IndexedVectorKind::Ecvector),
        "tqvector" => Some(IndexedVectorKind::Tqvector),
        _ => None,
    }
}

unsafe fn decode_slot_tid(slot: *mut pg_sys::TupleTableSlot) -> page::ItemPointer {
    let heap_tid = unsafe { (*slot).tts_tid };
    let tid = pg_sys::ItemPointerData {
        ip_blkid: heap_tid.ip_blkid,
        ip_posid: heap_tid.ip_posid,
    };
    let (block_number, offset_number) = item_pointer_get_both(tid);
    page::ItemPointer {
        block_number,
        offset_number,
    }
}

unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
) -> pg_sys::Datum {
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    let attr_index = usize::try_from(attnum - 1).expect("attribute number should be positive");
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        pgrx::error!("symphony does not support NULL {label}");
    }
    unsafe { *(*slot).tts_values.add(attr_index) }
}

fn vector_dimensions_from_datum(datum: pg_sys::Datum, attribute: IndexedVectorAttribute) -> u16 {
    let bytes = unsafe {
        Vec::<u8>::from_polymorphic_datum(datum, false, attribute.typoid).unwrap_or_else(|| {
            pgrx::error!("symphony failed to decode indexed column into binary form")
        })
    };
    vector_dimensions_from_bytes(&bytes, attribute.kind)
        .unwrap_or_else(|err| pgrx::error!("symphony indexed column decode failed: {err}"))
}

fn vector_dimensions_from_bytes(bytes: &[u8], kind: IndexedVectorKind) -> Result<u16, String> {
    match kind {
        IndexedVectorKind::Ecvector => {
            if bytes.len() % std::mem::size_of::<f32>() != 0 {
                return Err("ecvector payload length must be a multiple of 4 bytes".into());
            }
            u16::try_from(bytes.len() / std::mem::size_of::<f32>())
                .map_err(|_| format!("ecvector dimension {} exceeds u16", bytes.len() / 4))
        }
        IndexedVectorKind::Tqvector => {
            if bytes.len() < crate::MIN_BINARY_BYTES {
                return Err(format!(
                    "tqvector payload too short: got {}, need at least {}",
                    bytes.len(),
                    crate::MIN_BINARY_BYTES
                ));
            }
            Ok(u16::from_le_bytes(
                bytes[0..crate::HEADER_BYTES]
                    .try_into()
                    .expect("validated tqvector dimension prefix"),
            ))
        }
    }
}

#[cfg(any(test, feature = "pg_test"))]
mod tests {
    use super::{
        initial_metadata, options, page, singleton_data_pages, vector_dimensions_from_bytes,
        IndexedVectorKind,
    };
    use crate::storage::page::{DEFAULT_PAGE_SIZE, FIRST_DATA_BLOCK_NUMBER};

    fn tid(block_number: u32, offset_number: u16) -> page::ItemPointer {
        page::ItemPointer {
            block_number,
            offset_number,
        }
    }

    #[test]
    fn initial_metadata_tracks_v5_bootstrap_defaults() {
        let metadata = initial_metadata(
            options::SymphonyOptions {
                m: super::super::SYMPHONY_DEFAULT_M as i32,
                ef_construction: super::super::SYMPHONY_DEFAULT_EF_CONSTRUCTION as i32,
                padding_factor: super::super::SYMPHONY_BOOTSTRAP_PADDING_FACTOR as i32,
            },
            42,
        );

        assert_eq!(metadata.m, super::super::SYMPHONY_DEFAULT_M);
        assert_eq!(
            metadata.ef_construction,
            super::super::SYMPHONY_DEFAULT_EF_CONSTRUCTION
        );
        assert_eq!(
            metadata.padding_factor,
            super::super::SYMPHONY_BOOTSTRAP_PADDING_FACTOR
        );
        assert_eq!(metadata.rabitq_bits, super::super::SYMPHONY_RABITQ_BITS);
        assert_eq!(metadata.seed, 42);
        assert_eq!(metadata.entry_point, page::ItemPointer::INVALID);
        assert_eq!(metadata.dimensions, 0);
    }

    #[test]
    fn singleton_data_pages_write_empty_neighbor_and_element() {
        let (data_pages, entry_point) =
            singleton_data_pages(DEFAULT_PAGE_SIZE, tid(7, 9)).expect("singleton data pages");

        assert_eq!(data_pages.pages().len(), 1);
        assert_eq!(entry_point.block_number, FIRST_DATA_BLOCK_NUMBER);
        let element = data_pages.read_symphony_element(entry_point).unwrap();
        assert_eq!(element.level, 0);
        assert!(!element.deleted);
        assert_eq!(element.heaptids, vec![tid(7, 9)]);

        let neighbor = data_pages
            .read_symphony_neighbor(element.neighbortid, page::centered_code_len(1))
            .unwrap();
        assert_eq!(neighbor.count, 0);
        assert!(neighbor.tids.is_empty());
        assert!(neighbor.centered_codes.is_empty());
    }

    #[test]
    fn vector_dimensions_from_bytes_supports_ecvector_and_tqvector() {
        let ecvector = vec![0_u8; 4 * std::mem::size_of::<f32>()];
        assert_eq!(
            vector_dimensions_from_bytes(&ecvector, IndexedVectorKind::Ecvector).unwrap(),
            4
        );

        let tqvector = vec![4, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(
            vector_dimensions_from_bytes(&tqvector, IndexedVectorKind::Tqvector).unwrap(),
            4
        );
    }
}
