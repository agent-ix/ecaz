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
    pub const RowExclusiveLock: i32 = 3;
    pub const BUFFER_LOCK_UNLOCK: i32 = 0;
    pub const BUFFER_LOCK_SHARE: i32 = 1;
    pub const BUFFER_LOCK_EXCLUSIVE: i32 = 2;
    pub const GENERIC_XLOG_FULL_IMAGE: i32 = 1;
    pub const InvalidOffsetNumber: u16 = 0;

    pub type XLogRecPtr = u64;
    pub struct GenericXLogState;
    pub struct ItemIdData {
        lp_off_value: u32,
        lp_flags_value: u32,
        lp_len_value: u32,
    }

    impl ItemIdData {
        pub fn lp_off(&self) -> u32 {
            self.lp_off_value
        }
        pub fn lp_flags(&self) -> u32 {
            self.lp_flags_value
        }
        pub fn lp_len(&self) -> u32 {
            self.lp_len_value
        }
    }
    pub type ItemId = *mut ItemIdData;

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
        pub rd_id: Oid,
    }

    pub type Relation = *mut RelationData;

    pub const BLCKSZ: u32 = 8192;
    pub const InvalidOid: Oid = 0;
    pub const InvalidBuffer: Buffer = 0;
    pub const READ_STREAM_DEFAULT: i32 = 0;

    pub struct ReadStream;

    pub unsafe fn PrefetchBuffer(
        _relation: Relation,
        _fork: i32,
        _block: BlockNumber,
    ) {
    }

    pub unsafe fn read_stream_begin_relation(
        _flags: i32,
        _strategy: *mut (),
        _relation: Relation,
        _fork: i32,
        _callback: *mut (),
        _callback_state: *mut std::ffi::c_void,
        _per_buffer_data_size: usize,
    ) -> *mut ReadStream {
        Box::into_raw(Box::new(ReadStream))
    }

    pub unsafe fn read_stream_next_buffer(
        _stream: *mut ReadStream,
        _per_buffer_data: *mut *mut std::ffi::c_void,
    ) -> Buffer {
        BUFFER_REGISTRY.with(|r| {
            let mut reg = r.borrow_mut();
            match reg.read_stream_queue.pop_front() {
                Some((rd_id, block)) => reg.pin_buffer(rd_id, block),
                None => InvalidBuffer,
            }
        })
    }

    pub unsafe fn read_stream_end(stream: *mut ReadStream) {
        if !stream.is_null() {
            drop(unsafe { Box::from_raw(stream) });
        }
    }

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
        reset_buffer_registry();
    }

    pub unsafe fn LWLockAcquire(_lock: *mut LWLock, _mode: LWLockMode) {
        LWLOCK_ACQUIRE_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn LWLockRelease(_lock: *mut LWLock) {
        LWLOCK_RELEASE_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn ReadBufferExtended(
        relation: Relation,
        _fork: i32,
        block_number: BlockNumber,
        mode: ReadBufferMode::Type,
        _strategy: *mut (),
    ) -> Buffer {
        // SAFETY: relation may be null; only dereferenced after the check.
        let rd_id = if relation.is_null() {
            InvalidOid
        } else {
            unsafe { (*relation).rd_id }
        };

        if block_number == BlockNumber::MAX {
            // P_NEW: only valid with RBM_ZERO_AND_LOCK in this emulator. Other
            // modes preserve the prior `return 0 (== InvalidBuffer)` behavior
            // that the existing pg_guards tests still rely on.
            if mode != ReadBufferMode::RBM_ZERO_AND_LOCK || rd_id == InvalidOid {
                return InvalidBuffer;
            }
            return BUFFER_REGISTRY.with(|r| {
                let mut reg = r.borrow_mut();
                let new_block = reg.allocate_block(rd_id);
                reg.pin_buffer(rd_id, new_block)
            });
        }

        if rd_id == InvalidOid {
            // Legacy pg_guards tests pass synthetic block numbers through a
            // non-emulator-backed relation; keep the old `block + 1` mapping.
            return block_number as Buffer + 1;
        }

        BUFFER_REGISTRY.with(|r| {
            let mut reg = r.borrow_mut();
            // Allocate up to block_number so legacy tests that pre-pin an
            // arbitrary block keep working; the emulator round-trip tests
            // exclusively use P_NEW for first-touch and never widen the range
            // out from under themselves.
            while reg.relation_block_count(rd_id).unwrap_or(0) <= block_number {
                reg.allocate_block(rd_id);
            }
            reg.pin_buffer(rd_id, block_number)
        })
    }

    pub unsafe fn BufferIsValid(buffer: Buffer) -> bool {
        buffer > 0
    }

    pub unsafe fn ReleaseBuffer(_buffer: Buffer) {
        RELEASE_BUFFER_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn BufferGetBlockNumber(buffer: Buffer) -> BlockNumber {
        BUFFER_REGISTRY.with(|r| r.borrow().buffer_block_number(buffer))
    }

    pub unsafe fn LockBuffer(_buffer: Buffer, _lockmode: i32) {
        LOCK_BUFFER_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    pub unsafe fn BufferGetPage(buffer: Buffer) -> Page {
        BUFFER_REGISTRY.with(|r| r.borrow().buffer_page_ptr(buffer))
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
            rd_id: oid,
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

    /// Sources the block count from the emulator's buffer registry. Returns 0
    /// when the relation is null (legacy pg_guards tests rely on this) or when
    /// no block has been allocated for the relation's OID yet.
    pub unsafe fn RelationGetNumberOfBlocksInFork(
        relation: Relation,
        _fork: i32,
    ) -> BlockNumber {
        if relation.is_null() {
            return 0;
        }
        // SAFETY: relation was checked non-null; rd_id is a u32 read by value.
        let rd_id = unsafe { (*relation).rd_id };
        BUFFER_REGISTRY.with(|r| r.borrow().relation_block_count(rd_id).unwrap_or(0))
    }

    pub unsafe fn GetPageWithFreeSpace(
        _relation: Relation,
        _required_space: usize,
    ) -> BlockNumber {
        BlockNumber::MAX
    }

    pub unsafe fn RecordPageWithFreeSpace(
        _relation: Relation,
        _block: BlockNumber,
        _free_space: usize,
    ) {
    }

    // GenericXLog stubs. These do not actually emit WAL; the registered page
    // is just whatever the buffer points to.
    pub unsafe fn GenericXLogStart(_relation: Relation) -> *mut GenericXLogState {
        Box::into_raw(Box::new(GenericXLogState))
    }

    pub unsafe fn GenericXLogRegisterBuffer(
        _state: *mut GenericXLogState,
        buffer: Buffer,
        _flags: i32,
    ) -> Page {
        unsafe { BufferGetPage(buffer) }
    }

    pub unsafe fn GenericXLogFinish(state: *mut GenericXLogState) -> XLogRecPtr {
        if !state.is_null() {
            drop(unsafe { Box::from_raw(state) });
        }
        0
    }

    pub unsafe fn GenericXLogAbort(state: *mut GenericXLogState) {
        if !state.is_null() {
            drop(unsafe { Box::from_raw(state) });
        }
    }

    // Phase-1 backing-page emulator. Each (relation_oid, block_number) holds a
    // real `[u8; 8192]` page plus a stable line-pointer table; the page-level
    // helpers below operate on that storage so the success paths in
    // `src/am/ec_spire/page.rs` and `src/am/ec_spire/storage/relation_store.rs`
    // are exercisable from the careful shadow crate.

    const EMULATOR_PAGE_BYTES: usize = 8192;
    const EMULATOR_TUPLE_HEADER_BYTES: usize = 4;
    const EMULATOR_LINE_POINTER_BYTES: usize = 4;
    const EMULATOR_ALIGNMENT_BYTES: usize = 8;
    const EMULATOR_LP_NORMAL: u32 = 1;

    fn emulator_align_up(value: usize, alignment: usize) -> usize {
        let remainder = value % alignment;
        if remainder == 0 {
            value
        } else {
            value + (alignment - remainder)
        }
    }

    fn emulator_raw_tuple_storage_bytes(payload_len: usize) -> usize {
        emulator_align_up(
            EMULATOR_TUPLE_HEADER_BYTES + payload_len + EMULATOR_LINE_POINTER_BYTES,
            EMULATOR_ALIGNMENT_BYTES,
        )
    }

    pub(super) struct BackingPage {
        bytes: Box<[u8; EMULATOR_PAGE_BYTES]>,
        line_pointers: Vec<Box<ItemIdData>>,
        next_tuple_offset: usize,
        bytes_consumed: usize,
        special_size: usize,
        initialized: bool,
    }

    impl BackingPage {
        fn new() -> Self {
            Self {
                bytes: Box::new([0u8; EMULATOR_PAGE_BYTES]),
                line_pointers: Vec::new(),
                next_tuple_offset: EMULATOR_PAGE_BYTES,
                bytes_consumed: 0,
                special_size: 0,
                initialized: false,
            }
        }

        fn init(&mut self, special_size: usize) {
            self.bytes.fill(0);
            self.line_pointers.clear();
            self.special_size = special_size;
            self.next_tuple_offset = EMULATOR_PAGE_BYTES - special_size;
            self.bytes_consumed = 0;
            self.initialized = true;
        }

        fn page_ptr(&self) -> *mut u8 {
            // SAFETY of cast: callers only read/write through the returned
            // pointer while their buffer pin keeps this BackingPage alive.
            self.bytes.as_ptr() as *mut u8
        }

        fn special_ptr(&self) -> *mut u8 {
            // SAFETY: special_size <= EMULATOR_PAGE_BYTES is enforced by init().
            unsafe {
                self.bytes
                    .as_ptr()
                    .add(EMULATOR_PAGE_BYTES - self.special_size) as *mut u8
            }
        }

        fn free_space(&self) -> usize {
            EMULATOR_PAGE_BYTES.saturating_sub(self.special_size + self.bytes_consumed)
        }

        fn add_item(&mut self, payload: &[u8]) -> Option<u16> {
            let cost = emulator_raw_tuple_storage_bytes(payload.len());
            if self.free_space() < cost {
                return None;
            }
            let aligned_payload = emulator_align_up(payload.len(), EMULATOR_ALIGNMENT_BYTES);
            let new_offset = self.next_tuple_offset - aligned_payload;
            self.bytes[new_offset..new_offset + payload.len()].copy_from_slice(payload);
            self.next_tuple_offset = new_offset;
            self.bytes_consumed += cost;
            self.line_pointers.push(Box::new(ItemIdData {
                lp_off_value: new_offset as u32,
                lp_flags_value: EMULATOR_LP_NORMAL,
                lp_len_value: payload.len() as u32,
            }));
            Some(self.line_pointers.len() as u16)
        }

        fn delete_item(&mut self, offset: u16) {
            if offset == InvalidOffsetNumber {
                return;
            }
            if let Some(item) = self.line_pointers.get_mut((offset - 1) as usize) {
                let aligned = emulator_align_up(
                    item.lp_len_value as usize,
                    EMULATOR_ALIGNMENT_BYTES,
                );
                self.bytes_consumed = self.bytes_consumed.saturating_sub(aligned);
                item.lp_flags_value = 0;
                item.lp_len_value = 0;
            }
        }

        fn item_id_ptr(&mut self, offset: u16) -> *mut ItemIdData {
            if offset == InvalidOffsetNumber {
                return ptr::null_mut();
            }
            match self.line_pointers.get_mut((offset - 1) as usize) {
                Some(boxed) => &mut **boxed as *mut ItemIdData,
                None => ptr::null_mut(),
            }
        }
    }

    #[derive(Default)]
    pub(super) struct BufferRegistry {
        pages: std::collections::HashMap<(Oid, BlockNumber), Box<BackingPage>>,
        relation_block_counts: std::collections::HashMap<Oid, BlockNumber>,
        // bytes-pointer (as usize) -> stable raw pointer into Box<BackingPage>
        page_lookup: std::collections::HashMap<usize, *mut BackingPage>,
        // buffer_id -> (relation_oid, block_number); legacy convention is
        // buffer == block_number + 1 so existing pg_guards tests stay green.
        buffers: std::collections::HashMap<Buffer, (Oid, BlockNumber)>,
        // Drains in `read_stream_next_buffer`; populated by
        // `enqueue_read_stream_blocks_for_test` so prefetch loops have
        // something to iterate over.
        read_stream_queue: std::collections::VecDeque<(Oid, BlockNumber)>,
    }

    impl BufferRegistry {
        fn allocate_block(&mut self, rd_id: Oid) -> BlockNumber {
            let count = self.relation_block_counts.entry(rd_id).or_insert(0);
            let block_number = *count;
            *count = count
                .checked_add(1)
                .expect("emulator block count overflow");
            let mut backing = Box::new(BackingPage::new());
            let bytes_ptr = backing.page_ptr();
            let raw = &mut *backing as *mut BackingPage;
            self.page_lookup.insert(bytes_ptr as usize, raw);
            self.pages.insert((rd_id, block_number), backing);
            block_number
        }

        fn pin_buffer(&mut self, rd_id: Oid, block_number: BlockNumber) -> Buffer {
            let buffer = (block_number as Buffer)
                .checked_add(1)
                .expect("emulator buffer id overflow");
            self.buffers.insert(buffer, (rd_id, block_number));
            buffer
        }

        fn buffer_page_ptr(&self, buffer: Buffer) -> *mut u8 {
            if let Some((rd_id, block)) = self.buffers.get(&buffer) {
                if let Some(page) = self.pages.get(&(*rd_id, *block)) {
                    return page.page_ptr();
                }
            }
            // Legacy fallback for tests that pass synthetic Buffer ids without
            // going through ReadBufferExtended; preserves the
            // `!page().is_null()` assertions in the existing pg_guards tests.
            ptr::dangling_mut::<u8>()
        }

        fn buffer_block_number(&self, buffer: Buffer) -> BlockNumber {
            if let Some((_oid, block)) = self.buffers.get(&buffer) {
                return *block;
            }
            // Legacy: buffer = block + 1.
            (buffer as BlockNumber).saturating_sub(1)
        }

        fn relation_block_count(&self, rd_id: Oid) -> Option<BlockNumber> {
            self.relation_block_counts.get(&rd_id).copied()
        }

        fn page_mut(&self, page_ptr: Page) -> Option<&mut BackingPage> {
            let raw = *self.page_lookup.get(&(page_ptr as usize))?;
            // SAFETY: raw points into a Box<BackingPage> owned by self.pages.
            // The Box keeps the BackingPage at a stable heap address for as
            // long as the entry is in the map; reset_buffer_registry drops
            // everything together. We never hand out two &mut to the same
            // BackingPage simultaneously because each pg_sys page helper
            // borrows the registry, looks up exactly one page, and returns.
            Some(unsafe { &mut *raw })
        }
    }

    std::thread_local! {
        static BUFFER_REGISTRY: std::cell::RefCell<BufferRegistry> =
            std::cell::RefCell::new(BufferRegistry::default());
    }

    pub fn reset_buffer_registry() {
        BUFFER_REGISTRY.with(|r| *r.borrow_mut() = BufferRegistry::default());
    }

    /// Test-only: overwrite the raw bytes of an existing item on a
    /// known (rd_id, block, offset). Used to craft chain-corruption
    /// scenarios that the public insert API cannot produce. Returns
    /// false when the (rd_id, block, offset) is unknown.
    pub fn set_raw_tuple_bytes_for_test(
        rd_id: Oid,
        block_number: BlockNumber,
        offset_number: u16,
        new_bytes: &[u8],
    ) -> bool {
        if offset_number == InvalidOffsetNumber {
            return false;
        }
        BUFFER_REGISTRY.with(|r| {
            let mut reg = r.borrow_mut();
            let Some(page) = reg.pages.get_mut(&(rd_id, block_number)) else {
                return false;
            };
            let Some(item) = page.line_pointers.get_mut((offset_number - 1) as usize) else {
                return false;
            };
            let lp_off = item.lp_off_value as usize;
            let max_end = EMULATOR_PAGE_BYTES - page.special_size;
            if lp_off + new_bytes.len() > max_end {
                return false;
            }
            page.bytes[lp_off..lp_off + new_bytes.len()].copy_from_slice(new_bytes);
            item.lp_len_value = new_bytes.len() as u32;
            true
        })
    }

    /// Test-only: queue a list of (rd_id, block_number) entries that
    /// subsequent `read_stream_next_buffer` calls will surface. Drains
    /// each buffer once and then returns `InvalidBuffer`.
    pub fn enqueue_read_stream_blocks_for_test(blocks: Vec<(Oid, BlockNumber)>) {
        BUFFER_REGISTRY.with(|r| {
            let mut reg = r.borrow_mut();
            reg.read_stream_queue.extend(blocks);
        });
    }

    // Retained as a backward-compatible no-op for packets 033-034 tests that
    // still call it; the emulator's `RelationGetNumberOfBlocksInFork` now
    // sources its answer from `BUFFER_REGISTRY`, so a non-null relation that
    // has never been pinned returns 0 naturally.
    pub fn set_relation_block_count(_count: BlockNumber) {}

    pub unsafe fn PageInit(page: Page, _page_size: usize, special_size: usize) {
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            if let Some(backing) = reg.page_mut(page) {
                backing.init(special_size);
            }
        });
    }

    pub unsafe fn PageAddItemExtended(
        page: Page,
        item: *mut std::ffi::c_void,
        size: usize,
        _offset_number: u16,
        _flags: i32,
    ) -> u16 {
        if item.is_null() {
            return InvalidOffsetNumber;
        }
        // SAFETY: caller pins the page; `item` points at `size` bytes of
        // PostgreSQL-owned tuple memory and stays live for the call.
        let payload = unsafe { std::slice::from_raw_parts(item as *const u8, size) };
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            match reg.page_mut(page) {
                Some(backing) => backing.add_item(payload).unwrap_or(InvalidOffsetNumber),
                None => InvalidOffsetNumber,
            }
        })
    }

    pub unsafe fn PageGetItem(page: Page, item_id: ItemId) -> *mut std::ffi::c_void {
        if item_id.is_null() {
            return ptr::null_mut();
        }
        // SAFETY: caller pins the page; `item_id` came from `PageGetItemId`
        // for this page and is alive while the buffer pin is held.
        let id = unsafe { &*item_id };
        let offset = id.lp_off_value as usize;
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            match reg.page_mut(page) {
                // SAFETY: lp_off was checked < EMULATOR_PAGE_BYTES when added.
                Some(backing) => unsafe { backing.page_ptr().add(offset) as *mut std::ffi::c_void },
                None => ptr::null_mut(),
            }
        })
    }

    pub unsafe fn PageGetItemId(page: Page, offset: u16) -> ItemId {
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            match reg.page_mut(page) {
                Some(backing) => backing.item_id_ptr(offset),
                None => ptr::null_mut(),
            }
        })
    }

    pub unsafe fn PageGetMaxOffsetNumber(page: Page) -> u16 {
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            match reg.page_mut(page) {
                Some(backing) => backing.line_pointers.len() as u16,
                None => 0,
            }
        })
    }

    pub unsafe fn PageGetFreeSpace(page: Page) -> usize {
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            match reg.page_mut(page) {
                Some(backing) => backing.free_space(),
                None => 0,
            }
        })
    }

    pub unsafe fn PageGetSpecialPointer(page: Page) -> *mut std::ffi::c_void {
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            match reg.page_mut(page) {
                Some(backing) => backing.special_ptr() as *mut std::ffi::c_void,
                None => ptr::null_mut(),
            }
        })
    }

    pub unsafe fn PageGetSpecialSize(page: Page) -> u16 {
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            match reg.page_mut(page) {
                Some(backing) => backing.special_size as u16,
                None => 0,
            }
        })
    }

    pub unsafe fn PageIndexTupleDeleteNoCompact(page: Page, offset: u16) {
        BUFFER_REGISTRY.with(|r| {
            let reg = r.borrow();
            if let Some(backing) = reg.page_mut(page) {
                backing.delete_item(offset);
            }
        });
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

#[path = "../../../src/storage/wal.rs"]
pub mod wal;

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
        // MAX + RBM_ZERO_AND_LOCK on a valid relation is the legitimate P_NEW
        // allocation path under the backing-page emulator; pass a null
        // relation to keep this assertion covering the InvalidBuffer
        // early-out instead.
        assert!(unsafe {
            LockedBufferGuard::read_main_locked(
                std::ptr::null_mut(),
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
