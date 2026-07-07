//! pgrx-side ambuild wiring for `ec_distann` (Task 162 M0 scaffold slice).
//!
//! The current slice validates the indexed column, records dimensions, and
//! writes the format-v1 metadata page; the monolithic seed-deterministic
//! graph build (FR-076 records + FR-077 determinism contract) lands in the
//! following Task 162 slices. Until data pages exist the cost model keeps
//! the planner away from this index (see `cost.rs`), so scans cannot reach
//! the not-yet-implemented traversal path through plan selection.

use std::ffi::c_void;
use std::ptr::{self, NonNull};

use pgrx::{pg_sys, PgBox};

use crate::am::common::callback::pg_am_callback;
use crate::am::ec_diskann::{decode_heap_tid, ecvector_datum_to_vec};
use crate::storage::buffer_guard::LockedBufferGuard;
use crate::storage::page::{ItemPointer, METADATA_BLOCK_NUMBER};
use crate::storage::relation::RelationHandle;
use crate::storage::wal;
use crate::DEFAULT_QUANT_SEED;

use super::options::{self, EcDistannOptions};
use super::page::DistannMetadataPage;

const P_NEW: pg_sys::BlockNumber = u32::MAX;

struct BuildState {
    options: EcDistannOptions,
    dimensions: Option<u16>,
    scanned_tuples: usize,
}

impl BuildState {
    fn new(index_relation: pg_sys::Relation) -> Self {
        let options = options::relation_options(index_relation);
        Self {
            options,
            dimensions: None,
            scanned_tuples: 0,
        }
    }

    fn observe(&mut self, _heap_tid: ItemPointer, source_vector: &[f32]) {
        self.scanned_tuples += 1;
        if source_vector.is_empty() {
            pgrx::error!("ec_distann ambuild received an empty indexed vector");
        }
        let dim = u16::try_from(source_vector.len()).unwrap_or_else(|_| {
            pgrx::error!(
                "ec_distann indexed vector dimension {} exceeds 65535",
                source_vector.len()
            )
        });
        match self.dimensions {
            None => self.dimensions = Some(dim),
            Some(existing) if existing == dim => {}
            Some(existing) => pgrx::error!(
                "ec_distann ambuild requires a single dimension; saw {dim} after {existing}"
            ),
        }
    }
}

fn empty_metadata(state: &BuildState) -> DistannMetadataPage {
    DistannMetadataPage::empty(
        state.options.graph_degree as u16,
        state.options.build_list_size as u16,
        state.options.alpha,
        state.dimensions.unwrap_or(0),
        DEFAULT_QUANT_SEED,
        state.options.neighbor_code_format.metadata_kind(),
        state.options.head_index_cap as u32,
        state.options.closure_epsilon,
    )
}

pub(super) unsafe extern "C-unwind" fn ec_distann_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    pg_am_callback!({
        let mut state = BuildState::new(index_relation);
        validate_single_ecvector_attribute(heap_relation, index_info);

        let heap_tuples = pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            index_info,
            false,
            false,
            Some(ec_distann_build_callback),
            (&mut state as *mut BuildState).cast(),
            ptr::null_mut(),
        );

        if heap_tuples != state.scanned_tuples as f64 {
            pgrx::error!(
                "ec_distann ambuild scanned {heap_tuples} heap tuples but observed {}",
                state.scanned_tuples
            );
        }

        initialize_metadata_page(index_relation, empty_metadata(&state));

        let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
        result.heap_tuples = heap_tuples;
        // No graph-node records exist yet; the FR-076 record write path
        // lands in the next Task 162 slice.
        result.index_tuples = 0.0;
        result.into_pg()
    })
}

pub(super) unsafe extern "C-unwind" fn ec_distann_ambuildempty(index_relation: pg_sys::Relation) {
    pg_am_callback!({
        let state = BuildState::new(index_relation);
        initialize_metadata_page(index_relation, empty_metadata(&state));
    })
}

unsafe extern "C-unwind" fn ec_distann_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    pg_am_callback!({
        if values.is_null() || isnull.is_null() {
            pgrx::error!("ec_distann build callback received null datum arrays");
        }
        if *isnull {
            pgrx::error!("ec_distann does not support NULL indexed values");
        }
        let datum = *values;
        if datum.is_null() {
            pgrx::error!("ec_distann build callback received a null indexed datum");
        }
        let state = &mut *state.cast::<BuildState>();
        let source_vector = ecvector_datum_to_vec(datum);
        let heap_tid = decode_heap_tid(tid);
        state.observe(heap_tid, &source_vector);
    })
}

unsafe fn validate_single_ecvector_attribute(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) {
    if index_info.is_null() {
        pgrx::error!("ec_distann ambuild received a null IndexInfo");
    }
    let info = crate::am::common::pg_ptr::index_info(
        NonNull::new(index_info).expect("ec_distann IndexInfo should be non-null"),
    );
    if info.ii_NumIndexAttrs != 1 || info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_distann currently supports single-column indexes only");
    }
    if heap_relation.is_null() {
        pgrx::error!("ec_distann ambuild received a null heap relation");
    }
}

/// Decode the block-0 metadata record. Shared by later scan/build slices
/// and the TC-037 metadata assertions.
pub(crate) unsafe fn read_metadata_from_index(
    index_relation: pg_sys::Relation,
) -> Result<DistannMetadataPage, String> {
    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann metadata read needs a valid index relation".to_owned())?;
    let metadata_buffer = LockedBufferGuard::read_main_handle(
        handle,
        METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .ok_or_else(|| "ec_distann could not open metadata page".to_owned())?;
    let page = metadata_buffer.page();
    // SAFETY: `page` comes from the locked metadata buffer and remains pinned
    // while the special area is inspected; the size check below guarantees the
    // special pointer covers the serialized metadata payload.
    let metadata_bytes = unsafe {
        let special_size = pg_sys::PageGetSpecialSize(page) as usize;
        if special_size < super::page::DISTANN_METADATA_BYTES {
            return Err(format!(
                "ec_distann metadata page special area too small: got {special_size}, expected at least {}",
                super::page::DISTANN_METADATA_BYTES
            ));
        }
        let metadata_ptr = pg_sys::PageGetSpecialPointer(page).cast::<u8>();
        std::slice::from_raw_parts(
            metadata_ptr.cast_const(),
            super::page::DISTANN_METADATA_BYTES,
        )
    };
    DistannMetadataPage::decode(metadata_bytes)
}

unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: DistannMetadataPage,
) {
    let handle = NonNull::new(index_relation).unwrap_or_else(|| {
        pgrx::error!("ec_distann metadata initialization needs a valid index relation")
    });
    initialize_metadata_page_handle(handle, metadata);
}

fn initialize_metadata_page_handle(handle: RelationHandle, metadata: DistannMetadataPage) {
    let existing_blocks = crate::storage::relation::main_fork_block_count_handle(handle);
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        METADATA_BLOCK_NUMBER
    };
    let buffer = if target_block == P_NEW {
        LockedBufferGuard::read_main_locked_handle(
            handle,
            target_block,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
        )
    } else {
        LockedBufferGuard::read_main_handle(
            handle,
            target_block,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .unwrap_or_else(|| pgrx::error!("ec_distann failed to allocate metadata buffer"));
    write_metadata_to_buffer(handle, &buffer, &metadata);
}

fn write_metadata_to_buffer(
    handle: RelationHandle,
    buffer: &LockedBufferGuard,
    metadata: &DistannMetadataPage,
) {
    let mut wal_txn = wal::WalTxnScope::start_handle(handle);
    {
        let mut page = wal_txn.register_page(buffer);
        let metadata_bytes = metadata.encode();
        let special_size = (metadata_bytes.len() + 7) & !7;
        page.init(special_size);
        // SAFETY: `page` is the WAL-registered metadata page; the special area
        // was just sized from the encoded metadata before this owned-memory
        // copy.
        unsafe {
            let dst = pg_sys::PageGetSpecialPointer(page.page_ptr()).cast::<u8>();
            ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), dst, metadata_bytes.len());
        }
    }
    wal_txn.finish();
}
