#[allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
pub mod pg_sys {
    use std::ptr;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub type Buffer = i32;
    pub type BlockNumber = u32;
    pub type LOCKMODE = i32;
    pub type Oid = u32;
    pub type Page = *mut u8;
    pub type Snapshot = *mut SnapshotData;
    pub type IndexScanDesc = *mut ScanData;
    pub type TableScanDesc = *mut ScanData;

    pub const AccessShareLock: i32 = 1;
    pub const BUFFER_LOCK_UNLOCK: i32 = 0;

    pub mod ForkNumber {
        pub const MAIN_FORKNUM: i32 = 0;
    }

    pub mod ReadBufferMode {
        pub type Type = i32;
        pub const RBM_NORMAL: Type = 0;
        pub const RBM_ZERO_AND_LOCK: Type = 1;
    }

    #[derive(Debug)]
    pub enum LWLockMode {
        LW_SHARED,
        LW_EXCLUSIVE,
    }

    pub struct LWLock;
    pub struct SnapshotData;
    pub struct ScanData;
    pub struct TupleTableSlot;
    pub struct SPITupleTable;
    pub struct TupleDescData;
    pub struct TupleTableSlotOps;

    pub struct RelationData {
        pub rd_att: *mut TupleDescData,
    }

    pub type Relation = *mut RelationData;

    pub static LWLOCK_ACQUIRE_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static LWLOCK_RELEASE_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static RELEASE_BUFFER_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static LOCK_BUFFER_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static UNLOCK_RELEASE_BUFFER_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static INDEX_CLOSE_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static TABLE_CLOSE_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static RELATION_CLOSE_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static DROP_SLOT_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static UNREGISTER_SNAPSHOT_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static PUSH_ACTIVE_SNAPSHOT_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static POP_ACTIVE_SNAPSHOT_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static INDEX_ENDSCAN_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static HEAP_ENDSCAN_CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static SPI_FREETUPTABLE_CALLS: AtomicUsize = AtomicUsize::new(0);

    pub fn reset_counters() {
        for counter in [
            &LWLOCK_ACQUIRE_CALLS,
            &LWLOCK_RELEASE_CALLS,
            &RELEASE_BUFFER_CALLS,
            &LOCK_BUFFER_CALLS,
            &UNLOCK_RELEASE_BUFFER_CALLS,
            &INDEX_CLOSE_CALLS,
            &TABLE_CLOSE_CALLS,
            &RELATION_CLOSE_CALLS,
            &DROP_SLOT_CALLS,
            &UNREGISTER_SNAPSHOT_CALLS,
            &PUSH_ACTIVE_SNAPSHOT_CALLS,
            &POP_ACTIVE_SNAPSHOT_CALLS,
            &INDEX_ENDSCAN_CALLS,
            &HEAP_ENDSCAN_CALLS,
            &SPI_FREETUPTABLE_CALLS,
        ] {
            counter.store(0, Ordering::SeqCst);
        }
    }

    pub unsafe fn LWLockAcquire(_lock: *mut LWLock, _mode: LWLockMode) {
        LWLOCK_ACQUIRE_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn LWLockRelease(_lock: *mut LWLock) {
        LWLOCK_RELEASE_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn ReadBufferExtended(
        _relation: Relation,
        _fork: i32,
        block_number: BlockNumber,
        _mode: ReadBufferMode::Type,
        _strategy: *mut (),
    ) -> Buffer {
        if block_number == BlockNumber::MAX {
            return 0;
        }
        block_number as Buffer + 1
    }

    pub unsafe fn BufferIsValid(buffer: Buffer) -> bool {
        buffer > 0
    }

    pub unsafe fn ReleaseBuffer(_buffer: Buffer) {
        RELEASE_BUFFER_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn BufferGetBlockNumber(buffer: Buffer) -> BlockNumber {
        buffer as BlockNumber - 1
    }

    pub unsafe fn LockBuffer(_buffer: Buffer, _lockmode: i32) {
        LOCK_BUFFER_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn BufferGetPage(_buffer: Buffer) -> Page {
        ptr::dangling_mut::<u8>()
    }

    pub unsafe fn BufferGetPageSize(_buffer: Buffer) -> usize {
        8192
    }

    pub unsafe fn UnlockReleaseBuffer(_buffer: Buffer) {
        UNLOCK_RELEASE_BUFFER_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn open_relation(oid: Oid) -> Relation {
        if oid == 0 {
            return ptr::null_mut();
        }
        Box::into_raw(Box::new(RelationData {
            rd_att: ptr::dangling_mut::<TupleDescData>(),
        }))
    }

    unsafe fn close_relation(relation: Relation) {
        if !relation.is_null() {
            drop(unsafe { Box::from_raw(relation) });
        }
    }

    pub unsafe fn index_open(index_oid: Oid, _lockmode: LOCKMODE) -> Relation {
        open_relation(index_oid)
    }

    pub unsafe fn index_close(relation: Relation, _lockmode: LOCKMODE) {
        INDEX_CLOSE_CALLS.fetch_add(1, Ordering::SeqCst);
        unsafe { close_relation(relation) };
    }

    pub unsafe fn table_open(relation_oid: Oid, _lockmode: LOCKMODE) -> Relation {
        open_relation(relation_oid)
    }

    pub unsafe fn table_close(relation: Relation, _lockmode: LOCKMODE) {
        TABLE_CLOSE_CALLS.fetch_add(1, Ordering::SeqCst);
        unsafe { close_relation(relation) };
    }

    pub unsafe fn relation_open(relation_oid: Oid, _lockmode: LOCKMODE) -> Relation {
        open_relation(relation_oid)
    }

    pub unsafe fn relation_close(relation: Relation, _lockmode: LOCKMODE) {
        RELATION_CLOSE_CALLS.fetch_add(1, Ordering::SeqCst);
        unsafe { close_relation(relation) };
    }

    pub unsafe fn table_slot_create(
        _relation: Relation,
        _estate: *mut (),
    ) -> *mut TupleTableSlot {
        Box::into_raw(Box::new(TupleTableSlot))
    }

    pub unsafe fn table_slot_callbacks(_relation: Relation) -> *mut TupleTableSlotOps {
        ptr::dangling_mut::<TupleTableSlotOps>()
    }

    pub unsafe fn MakeSingleTupleTableSlot(
        _tuple_desc: *mut TupleDescData,
        _ops: *mut TupleTableSlotOps,
    ) -> *mut TupleTableSlot {
        Box::into_raw(Box::new(TupleTableSlot))
    }

    pub unsafe fn ExecDropSingleTupleTableSlot(slot: *mut TupleTableSlot) {
        DROP_SLOT_CALLS.fetch_add(1, Ordering::SeqCst);
        if !slot.is_null() {
            drop(unsafe { Box::from_raw(slot) });
        }
    }

    pub unsafe fn GetLatestSnapshot() -> Snapshot {
        Box::into_raw(Box::new(SnapshotData))
    }

    pub unsafe fn GetTransactionSnapshot() -> Snapshot {
        Box::into_raw(Box::new(SnapshotData))
    }

    pub unsafe fn RegisterSnapshot(snapshot: Snapshot) -> Snapshot {
        snapshot
    }

    pub unsafe fn UnregisterSnapshot(snapshot: Snapshot) {
        UNREGISTER_SNAPSHOT_CALLS.fetch_add(1, Ordering::SeqCst);
        if !snapshot.is_null() {
            drop(unsafe { Box::from_raw(snapshot) });
        }
    }

    pub unsafe fn PushActiveSnapshot(_snapshot: Snapshot) {
        PUSH_ACTIVE_SNAPSHOT_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn PopActiveSnapshot() {
        POP_ACTIVE_SNAPSHOT_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn CommandCounterIncrement() {}

    pub unsafe fn index_beginscan(
        _heap_relation: Relation,
        _index_relation: Relation,
        _snapshot: Snapshot,
        _nkeys: i32,
        _norderbys: i32,
    ) -> IndexScanDesc {
        Box::into_raw(Box::new(ScanData))
    }

    pub unsafe fn index_endscan(scan: IndexScanDesc) {
        INDEX_ENDSCAN_CALLS.fetch_add(1, Ordering::SeqCst);
        if !scan.is_null() {
            drop(unsafe { Box::from_raw(scan) });
        }
    }

    pub unsafe fn heap_beginscan(
        _heap_relation: Relation,
        _snapshot: Snapshot,
        _nkeys: i32,
        _key: *mut (),
        _parallel_scan: *mut (),
        _flags: u32,
    ) -> TableScanDesc {
        Box::into_raw(Box::new(ScanData))
    }

    pub unsafe fn heap_endscan(scan: TableScanDesc) {
        HEAP_ENDSCAN_CALLS.fetch_add(1, Ordering::SeqCst);
        if !scan.is_null() {
            drop(unsafe { Box::from_raw(scan) });
        }
    }

    pub unsafe fn SPI_freetuptable(tuptable: *mut SPITupleTable) {
        SPI_FREETUPTABLE_CALLS.fetch_add(1, Ordering::SeqCst);
        if !tuptable.is_null() {
            drop(unsafe { Box::from_raw(tuptable) });
        }
    }
}

#[path = "../../../src/storage/buffer_guard.rs"]
pub mod buffer_guard;

#[path = "../../../src/storage/lock_guard.rs"]
pub mod lock_guard;

#[path = "../../../src/storage/relation_guard.rs"]
pub mod relation_guard;

#[path = "../../../src/storage/snapshot_guard.rs"]
pub mod snapshot_guard;

#[path = "../../../src/storage/scan_guard.rs"]
pub mod scan_guard;

#[path = "../../../src/storage/slot_guard.rs"]
pub mod slot_guard;

#[path = "../../../src/storage/spi_guard.rs"]
pub mod spi_guard;

#[cfg(test)]
mod tests {
    use super::buffer_guard::{LockedBufferGuard, PinnedBufferGuard};
    use super::lock_guard::LwLockGuard;
    use super::pg_sys;
    use super::relation_guard::{HeapRelationGuard, IndexRelationGuard, RelationGuard};
    use super::scan_guard::{HeapScanGuard, IndexScanGuard};
    use super::slot_guard::TupleTableSlotGuard;
    use super::snapshot_guard::{ActiveSnapshotGuard, RegisteredSnapshotGuard};
    use super::spi_guard::SpiTupleTableGuard;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());
    static CUSTOM_LWLOCK_RELEASE_CALLS: AtomicUsize = AtomicUsize::new(0);

    unsafe fn custom_lwlock_release(_lock: *mut pg_sys::LWLock) {
        CUSTOM_LWLOCK_RELEASE_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn lock_guard_releases_adopted_lwlock_once() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        CUSTOM_LWLOCK_RELEASE_CALLS.store(0, Ordering::SeqCst);
        let mut lock = pg_sys::LWLock;

        {
            let _guard = unsafe { LwLockGuard::acquire_shared(&mut lock) };
        }
        {
            let _guard = unsafe { LwLockGuard::acquire_exclusive(&mut lock) };
        }
        {
            let _guard =
                unsafe { LwLockGuard::from_acquired_with_release(&mut lock, custom_lwlock_release) };
        }

        assert_eq!(pg_sys::LWLOCK_ACQUIRE_CALLS.load(Ordering::SeqCst), 2);
        assert_eq!(pg_sys::LWLOCK_RELEASE_CALLS.load(Ordering::SeqCst), 2);
        assert_eq!(CUSTOM_LWLOCK_RELEASE_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn pinned_buffer_guard_rejects_invalid_and_releases_pin() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();

        assert!(unsafe { PinnedBufferGuard::from_pinned(0) }.is_none());
        {
            let guard = unsafe { PinnedBufferGuard::from_pinned(5) }.unwrap();
            assert_eq!(guard.buffer(), 5);
            assert_eq!(guard.block_number(), 4);
            {
                let lock = guard.lock(2);
                assert_eq!(lock.page_size(), 8192);
                assert!(!lock.page().is_null());
            }
        }

        assert_eq!(pg_sys::LOCK_BUFFER_CALLS.load(Ordering::SeqCst), 2);
        assert_eq!(pg_sys::RELEASE_BUFFER_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn pinned_buffer_read_main_wraps_read_buffer_and_rejects_invalid_reads() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let relation = HeapRelationGuard::try_access_share(21).unwrap();

        assert!(unsafe {
            PinnedBufferGuard::read_main(
                relation.as_ptr(),
                pg_sys::BlockNumber::MAX,
                pg_sys::ReadBufferMode::RBM_NORMAL,
            )
        }
        .is_none());
        {
            let guard = unsafe {
                PinnedBufferGuard::read_main(
                    relation.as_ptr(),
                    6,
                    pg_sys::ReadBufferMode::RBM_NORMAL,
                )
            }
            .unwrap();
            assert_eq!(guard.buffer(), 7);
            assert_eq!(guard.block_number(), 6);
        }

        assert_eq!(pg_sys::RELEASE_BUFFER_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn locked_buffer_guard_rejects_invalid_and_unlocks_release() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();

        assert!(unsafe { LockedBufferGuard::lock_pinned(0, 2) }.is_none());
        {
            let guard = unsafe { LockedBufferGuard::lock_pinned(8, 2) }.unwrap();
            assert_eq!(guard.buffer(), 8);
            assert_eq!(guard.block_number(), 7);
            assert_eq!(guard.page_size(), 8192);
            assert!(!guard.page().is_null());
        }

        assert_eq!(pg_sys::LOCK_BUFFER_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(
            pg_sys::UNLOCK_RELEASE_BUFFER_CALLS.load(Ordering::SeqCst),
            1
        );
    }

    #[test]
    fn locked_buffer_read_main_variants_wrap_read_buffer() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let relation = HeapRelationGuard::try_access_share(23).unwrap();

        assert!(unsafe {
            LockedBufferGuard::read_main(
                relation.as_ptr(),
                pg_sys::BlockNumber::MAX,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                2,
            )
        }
        .is_none());
        assert!(unsafe {
            LockedBufferGuard::read_main_locked(
                relation.as_ptr(),
                pg_sys::BlockNumber::MAX,
                pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
            )
        }
        .is_none());

        {
            let guard = unsafe {
                LockedBufferGuard::read_main(
                    relation.as_ptr(),
                    9,
                    pg_sys::ReadBufferMode::RBM_NORMAL,
                    2,
                )
            }
            .unwrap();
            assert_eq!(guard.buffer(), 10);
            assert_eq!(guard.block_number(), 9);
            assert_eq!(guard.page_size(), 8192);
            assert!(!guard.page().is_null());
        }
        {
            let guard = unsafe {
                LockedBufferGuard::read_main_locked(
                    relation.as_ptr(),
                    10,
                    pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
                )
            }
            .unwrap();
            assert_eq!(guard.buffer(), 11);
            assert_eq!(guard.block_number(), 10);
        }

        assert_eq!(pg_sys::LOCK_BUFFER_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(
            pg_sys::UNLOCK_RELEASE_BUFFER_CALLS.load(Ordering::SeqCst),
            2
        );
    }

    #[test]
    fn relation_guards_reject_null_and_close_matching_relation_kind() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();

        assert!(IndexRelationGuard::try_open(0, 1).is_none());
        assert!(HeapRelationGuard::try_open(0, 1).is_none());
        assert!(RelationGuard::try_open(0, 1).is_none());
        {
            let access_share = IndexRelationGuard::access_share(10, "guard coverage");
            let index = IndexRelationGuard::try_access_share(11).unwrap();
            let heap = HeapRelationGuard::try_access_share(12).unwrap();
            let relation = RelationGuard::try_open(13, 1).unwrap();
            assert!(!access_share.as_ptr().is_null());
            assert!(!index.as_ptr().is_null());
            assert!(!heap.as_ptr().is_null());
            assert!(!relation.as_ptr().is_null());
        }
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let failed_open =
            std::panic::catch_unwind(|| IndexRelationGuard::open(0, 1, "guard coverage"));
        std::panic::set_hook(previous_hook);
        assert!(failed_open.is_err());

        assert_eq!(pg_sys::INDEX_CLOSE_CALLS.load(Ordering::SeqCst), 2);
        assert_eq!(pg_sys::TABLE_CLOSE_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(pg_sys::RELATION_CLOSE_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn slot_guard_drops_created_slots_once() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let relation = HeapRelationGuard::try_access_share(22).unwrap();

        {
            let slot = TupleTableSlotGuard::create(relation.as_ptr()).unwrap();
            assert!(!slot.as_ptr().is_null());
        }
        {
            let slot = TupleTableSlotGuard::single_for_heap(relation.as_ptr()).unwrap();
            assert!(!slot.as_ptr().is_null());
        }

        assert_eq!(pg_sys::DROP_SLOT_CALLS.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn snapshot_guards_unregister_and_active_guard_pops() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();

        {
            let snapshot = RegisteredSnapshotGuard::latest().unwrap();
            assert!(!snapshot.as_ptr().is_null());
        }
        {
            let snapshot = RegisteredSnapshotGuard::transaction().unwrap();
            assert!(!snapshot.as_ptr().is_null());
        }
        {
            let snapshot = ActiveSnapshotGuard::latest().unwrap();
            assert!(!snapshot.as_ptr().is_null());
        }

        assert_eq!(
            pg_sys::UNREGISTER_SNAPSHOT_CALLS.load(Ordering::SeqCst),
            3
        );
        assert_eq!(
            pg_sys::PUSH_ACTIVE_SNAPSHOT_CALLS.load(Ordering::SeqCst),
            1
        );
        assert_eq!(pg_sys::POP_ACTIVE_SNAPSHOT_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn scan_guards_end_index_and_heap_scans_once() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();
        let index = IndexRelationGuard::try_access_share(31).unwrap();
        let heap = HeapRelationGuard::try_access_share(32).unwrap();
        let snapshot = ActiveSnapshotGuard::latest().unwrap();

        {
            let scan = IndexScanGuard::begin(&heap, &index, &snapshot, 0, 1).unwrap();
            assert!(!scan.as_ptr().is_null());
        }
        {
            let scan = HeapScanGuard::begin(heap.as_ptr(), &snapshot, 0).unwrap();
            assert!(!scan.as_ptr().is_null());
        }

        assert_eq!(pg_sys::INDEX_ENDSCAN_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(pg_sys::HEAP_ENDSCAN_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn spi_tuple_table_guard_rejects_null_and_frees_owned_table() {
        let _serial = TEST_LOCK.lock().unwrap();
        pg_sys::reset_counters();

        assert!(unsafe { SpiTupleTableGuard::from_owned(std::ptr::null_mut()) }.is_none());
        {
            let table = Box::into_raw(Box::new(pg_sys::SPITupleTable));
            let guard = unsafe { SpiTupleTableGuard::from_owned(table) }.unwrap();
            assert_eq!(guard.as_ptr(), table);
        }

        assert_eq!(pg_sys::SPI_FREETUPTABLE_CALLS.load(Ordering::SeqCst), 1);
    }
}
