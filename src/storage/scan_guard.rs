use pgrx::pg_sys;

use super::{
    relation_guard::{HeapRelationGuard, IndexRelationGuard},
    snapshot_guard::ActiveSnapshotGuard,
};

pub(crate) struct IndexScanGuard {
    scan: pg_sys::IndexScanDesc,
}

impl IndexScanGuard {
    pub(crate) fn begin(
        heap_relation: &HeapRelationGuard,
        index_relation: &IndexRelationGuard,
        snapshot: &ActiveSnapshotGuard,
        nkeys: i32,
        norderbys: i32,
    ) -> Option<Self> {
        #[cfg(feature = "pg18")]
        // SAFETY: `heap_relation`, `index_relation`, and `snapshot` are owned
        // by live guards in the caller; this guard owns the matching
        // `index_endscan`.
        let scan = unsafe {
            pg_sys::index_beginscan(
                heap_relation.as_ptr(),
                index_relation.as_ptr(),
                snapshot.as_ptr(),
                std::ptr::null_mut(),
                nkeys,
                norderbys,
            )
        };
        #[cfg(not(feature = "pg18"))]
        // SAFETY: `heap_relation`, `index_relation`, and `snapshot` are owned
        // by live guards in the caller; this guard owns the matching
        // `index_endscan`.
        let scan = unsafe {
            pg_sys::index_beginscan(
                heap_relation.as_ptr(),
                index_relation.as_ptr(),
                snapshot.as_ptr(),
                nkeys,
                norderbys,
            )
        };
        if scan.is_null() {
            return None;
        }
        Some(Self { scan })
    }

    pub(crate) fn as_ptr(&self) -> pg_sys::IndexScanDesc {
        self.scan
    }
}

impl Drop for IndexScanGuard {
    fn drop(&mut self) {
        // SAFETY: `scan` was returned by `IndexScanGuard::begin`; this guard
        // owns the matching end call.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::index_endscan(self.scan) };
    }
}
