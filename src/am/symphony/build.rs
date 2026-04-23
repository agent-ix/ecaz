use std::ptr;

use pgrx::{pg_sys, PgBox};
use rand::random;

use super::{options, page};
use crate::am::common::metadata;

const POPULATED_BUILD_ERROR: &str =
    "symphony ambuild for populated relations is not implemented yet";

pub(super) unsafe extern "C-unwind" fn symphony_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    _index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if heap_has_visible_tuples(heap_relation) {
                pgrx::error!("{POPULATED_BUILD_ERROR}");
            }

            write_initial_metadata(index_relation);
            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = 0.0;
            result.index_tuples = 0.0;
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

unsafe fn heap_has_visible_tuples(heap_relation: pg_sys::Relation) -> bool {
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

    let found_visible_tuple = unsafe {
        pg_sys::heap_getnextslot(scan, pg_sys::ScanDirection::ForwardScanDirection, slot)
    };

    unsafe {
        pg_sys::heap_endscan(scan);
        pg_sys::PopActiveSnapshot();
        pg_sys::UnregisterSnapshot(snapshot);
        pg_sys::ExecDropSingleTupleTableSlot(slot);
    }
    found_visible_tuple
}

#[cfg(any(test, feature = "pg_test"))]
mod tests {
    use super::{initial_metadata, options, page};

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
}
