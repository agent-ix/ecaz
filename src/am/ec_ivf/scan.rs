use std::ptr;

use pgrx::{pg_sys, PgBox};

#[derive(Debug, Default)]
struct EcIvfScanOpaque {
    rescan_called: bool,
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
            if scan.is_null() {
                pgrx::error!("ec_ivf failed to allocate scan descriptor");
            }

            (*scan).parallel_scan = ptr::null_mut();
            (*scan).opaque = PgBox::<EcIvfScanOpaque>::alloc0().into_pg().cast();
            scan
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amrescan(
    scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_ivf amrescan received a null scan descriptor");
            }
            if nkeys != 0 {
                pgrx::error!("ec_ivf scan does not support index quals yet");
            }
            if norderbys != 1 {
                pgrx::error!("ec_ivf scan currently requires exactly one ORDER BY query");
            }
            if orderbys.is_null() {
                pgrx::error!("ec_ivf amrescan received null order-by scan keys");
            }

            let opaque_ptr = (*scan).opaque.cast::<EcIvfScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_ivf amrescan missing scan opaque state");
            }
            (*opaque_ptr).rescan_called = true;
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_ivf amgettuple received a null scan descriptor");
            }
            if direction != pg_sys::ScanDirection::ForwardScanDirection {
                pgrx::error!("ec_ivf amgettuple only supports forward scan direction");
            }
            let opaque_ptr = (*scan).opaque.cast::<EcIvfScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_ivf amgettuple missing scan opaque state");
            }
            if !(*opaque_ptr).rescan_called {
                pgrx::error!("ec_ivf amgettuple requires amrescan before scan execution");
            }

            false
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amendscan(scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }

            let opaque_ptr = (*scan).opaque;
            if !opaque_ptr.is_null() {
                pg_sys::pfree(opaque_ptr);
                (*scan).opaque = ptr::null_mut();
            }
        })
    }
}

#[cfg(any(test, feature = "pg_test"))]
struct DebugHeapBackedScan {
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    scan: pg_sys::IndexScanDesc,
    registered_snapshot: pg_sys::Snapshot,
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_push_latest_snapshot() -> pg_sys::Snapshot {
    unsafe { pg_sys::CommandCounterIncrement() };
    let snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
    if snapshot.is_null() {
        pgrx::error!("ec_ivf debug scan could not acquire a latest snapshot");
    }
    unsafe { pg_sys::PushActiveSnapshot(snapshot) };
    snapshot
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_begin_heap_backed_scan(index_oid: pg_sys::Oid) -> DebugHeapBackedScan {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        pgrx::error!("ec_ivf debug scan could not resolve heap relation for index {index_oid}");
    }

    let heap_relation =
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let registered_snapshot = unsafe { debug_push_latest_snapshot() };
    #[cfg(feature = "pg18")]
    let scan = unsafe {
        pg_sys::index_beginscan(
            heap_relation,
            index_relation,
            registered_snapshot,
            ptr::null_mut(),
            0,
            1,
        )
    };
    #[cfg(not(feature = "pg18"))]
    let scan = unsafe {
        pg_sys::index_beginscan(heap_relation, index_relation, registered_snapshot, 0, 1)
    };
    if scan.is_null() {
        unsafe {
            pg_sys::PopActiveSnapshot();
            pg_sys::UnregisterSnapshot(registered_snapshot);
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
            pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }
        pgrx::error!("ec_ivf debug scan failed to begin heap-backed index scan");
    }

    DebugHeapBackedScan {
        index_relation,
        heap_relation,
        scan,
        registered_snapshot,
    }
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_end_heap_backed_scan(state: DebugHeapBackedScan) {
    unsafe {
        pg_sys::index_endscan(state.scan);
        pg_sys::PopActiveSnapshot();
        pg_sys::UnregisterSnapshot(state.registered_snapshot);
        pg_sys::index_close(
            state.index_relation,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        );
        pg_sys::table_close(
            state.heap_relation,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        );
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_gettuple_after_rescan_result(index_oid: pg_sys::Oid) -> bool {
    let state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let mut orderby = pg_sys::ScanKeyData::default();
    unsafe { pg_sys::index_rescan(state.scan, ptr::null_mut(), 0, &mut orderby, 1) };
    let tid = unsafe {
        pg_sys::index_getnext_tid(state.scan, pg_sys::ScanDirection::ForwardScanDirection)
    };
    let found = !tid.is_null();

    unsafe { debug_end_heap_backed_scan(state) };
    found
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_metadata(index_oid: pg_sys::Oid) -> (u16, u32, u32, u32, u64) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::page::read_metadata_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        metadata.format_version,
        metadata.nlists,
        metadata.nprobe,
        metadata.training_sample_rows,
        metadata.seed,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_build_metadata(
    index_oid: pg_sys::Oid,
) -> (u16, u32, u16, u64, bool, bool) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::page::read_metadata_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        metadata.dimensions,
        metadata.nlists,
        metadata.training_version,
        metadata.total_live_tuples,
        metadata.centroid_head != crate::storage::page::ItemPointer::INVALID,
        metadata.directory_head != crate::storage::page::ItemPointer::INVALID,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_directory_summary(
    index_oid: pg_sys::Oid,
) -> (u32, u32, u64, u64, u64) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::page::read_metadata_page(index_relation) };

    if metadata.directory_head == crate::storage::page::ItemPointer::INVALID {
        if metadata.total_live_tuples != 0 {
            unsafe {
                pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
            };
            pgrx::error!("ec_ivf metadata has live tuples but no directory head");
        }
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        return (metadata.nlists, metadata.nlists, 0, 0, 0);
    }

    let mut next_tid = metadata.directory_head;
    let mut empty_lists = 0_u32;
    let mut live_sum = 0_u64;
    let mut dead_sum = 0_u64;
    let mut inserted_sum = 0_u64;
    for expected_list_id in 0..metadata.nlists {
        let (directory, following_tid) = unsafe {
            super::page::read_ivf_list_directory_and_next(index_relation, next_tid)
                .unwrap_or_else(|e| pgrx::error!("{e}"))
        };
        if directory.list_id != expected_list_id {
            unsafe {
                pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
            };
            pgrx::error!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id,
                expected_list_id
            );
        }
        if directory.live_count == 0 {
            empty_lists += 1;
        }
        live_sum += directory.live_count;
        dead_sum += directory.dead_count;
        inserted_sum += directory.inserted_since_build;
        next_tid = following_tid;
    }

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        metadata.nlists,
        empty_lists,
        live_sum,
        dead_sum,
        inserted_sum,
    )
}
