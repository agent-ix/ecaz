pub(super) fn set_scan_heap_tid(scan: pg_sys::IndexScanDesc, heap_tid: ItemPointer) {
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut (*scan).xs_heaptid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
    }
}

pub(super) fn set_scan_orderby_score(scan: pg_sys::IndexScanDesc, score: f32) {
    unsafe {
        if (*scan).xs_orderbyvals.is_null() {
            (*scan).xs_orderbyvals =
                pg_sys::palloc0(std::mem::size_of::<pg_sys::Datum>()).cast::<pg_sys::Datum>();
        }
        if (*scan).xs_orderbynulls.is_null() {
            (*scan).xs_orderbynulls = pg_sys::palloc0(std::mem::size_of::<bool>()).cast::<bool>();
        }

        *(*scan).xs_orderbyvals = score.into_datum().expect("score should convert to datum");
        *(*scan).xs_orderbynulls = false;
    }
}

pub(super) fn clear_scan_orderby_output(scan: pg_sys::IndexScanDesc) {
    unsafe {
        if !(*scan).xs_orderbynulls.is_null() {
            *(*scan).xs_orderbynulls = true;
        }
    }
}

pub(super) struct ResolvedScanHeapRelation {
    relation: pg_sys::Relation,
    owned: bool,
}

impl ResolvedScanHeapRelation {
    fn borrowed(relation: pg_sys::Relation) -> Self {
        Self {
            relation,
            owned: false,
        }
    }

    fn owned(relation: pg_sys::Relation) -> Self {
        Self {
            relation,
            owned: true,
        }
    }

    pub(super) fn as_ptr(&self) -> pg_sys::Relation {
        self.relation
    }
}

impl Drop for ResolvedScanHeapRelation {
    fn drop(&mut self) {
        if self.owned && !self.relation.is_null() {
            // SAFETY: Owned heap relations are opened by
            // `resolve_scan_heap_relation` with `AccessShareLock`; this guard
            // owns the matching close.
            unsafe {
                pg_sys::table_close(
                    self.relation,
                    pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                );
            }
        }
    }
}

pub(super) unsafe fn load_relation_epoch_manifests(
    index_relation: pg_sys::Relation,
    root_control: SpireRootControlState,
) -> Result<
    (
        SpireEpochManifest,
        SpireObjectManifest,
        SpirePlacementDirectory,
    ),
    String,
> {
    if root_control.active_epoch == 0 {
        return Err("ec_spire cannot load manifests for empty active epoch".to_owned());
    }
    let epoch_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.epoch_manifest_tid)? };
    let object_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.object_manifest_tid)? };
    let placement_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.placement_directory_tid)? };
    let local_store_config =
        unsafe { load_relation_local_store_config(index_relation, root_control)? };
    let epoch_manifest = SpireEpochManifest::decode(&epoch_bytes)?;
    let object_manifest = SpireObjectManifest::decode(&object_bytes)?;
    let placement_directory = SpirePlacementDirectory::decode(&placement_bytes)?;
    if epoch_manifest.epoch != root_control.active_epoch {
        return Err(format!(
            "ec_spire root/control active epoch {} does not match epoch manifest {}",
            root_control.active_epoch, epoch_manifest.epoch
        ));
    }
    SpireValidatedEpochSnapshot::new(&epoch_manifest, &object_manifest, &placement_directory)?;
    ensure_local_heap_placement_directory_is_deliverable(&placement_directory)?;
    local_store_config.validate_placement_directory(&placement_directory)?;
    Ok((epoch_manifest, object_manifest, placement_directory))
}

fn ensure_local_heap_placement_directory_is_deliverable(
    placement_directory: &SpirePlacementDirectory,
) -> Result<(), String> {
    let remote_placement_count = placement_directory
        .entries
        .iter()
        .filter(|placement| placement.node_id != super::meta::SPIRE_LOCAL_NODE_ID)
        .count();
    if remote_placement_count == 0 {
        return Ok(());
    }

    let Some(first_remote) = placement_directory
        .entries
        .iter()
        .find(|placement| placement.node_id != super::meta::SPIRE_LOCAL_NODE_ID) else {
        return Err(
            "ec_spire local heap tuple delivery remote placement count disagrees with placement directory"
                .to_owned(),
        );
    };
    Err(format!(
        "ec_spire local heap tuple delivery requires {} before consuming {remote_placement_count} remote placement(s); first remote pid {} is on node_id {}",
        super::SPIRE_REMOTE_EXECUTOR_STEP_CUSTOM_SCAN_TUPLE_DELIVERY,
        first_remote.pid,
        first_remote.node_id
    ))
}

pub(super) unsafe fn load_relation_local_store_config(
    index_relation: pg_sys::Relation,
    root_control: SpireRootControlState,
) -> Result<SpireLocalStoreConfig, String> {
    if root_control.active_epoch == 0 {
        return Err("ec_spire cannot load local store config for empty active epoch".to_owned());
    }
    let bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.local_store_config_tid)? };
    SpireLocalStoreConfig::decode(&bytes)
}

unsafe fn decode_scan_orderby_query(orderbys: pg_sys::ScanKey) -> Result<SpireScanQuery, String> {
    if orderbys.is_null() {
        return Err("ec_spire amrescan received null order-by scan keys".to_owned());
    }

    let orderby = unsafe { &*orderbys };
    if (orderby.sk_flags as u32) & pg_sys::SK_ISNULL != 0 {
        return Err("ec_spire scan query must not be NULL".to_owned());
    }

    let values =
        Vec::<f32>::from_polymorphic_datum(orderby.sk_argument, false, pg_sys::FLOAT4ARRAYOID)
            .ok_or_else(|| "ec_spire scan requires a real[] ORDER BY query".to_owned())?;
    SpireScanQuery::new(values)
}

unsafe fn prepare_single_level_relation_snapshot_scan_candidates(
    scan: pg_sys::IndexScanDesc,
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    options: EcSpireOptions,
) -> Result<SpirePreparedScanCandidates, String> {
    let heap_relation = resolve_scan_heap_relation(scan);
    let heap_relation_ptr = heap_relation.as_ptr();
    let snapshot_pg = resolve_scan_snapshot(scan);
    let indexed_attribute = unsafe {
        source::resolve_indexed_vector_attribute(
            heap_relation_ptr,
            (*scan).indexRelation,
            "ec_spire heap rerank indexed column",
        )
    };
    let slot = unsafe { allocate_heap_slot(heap_relation_ptr)? };

    let result = prepare_single_level_snapshot_scan_candidates_with_prefetch(
        snapshot,
        object_store,
        query,
        options,
        |candidates| {
            unsafe { prefetch_heap_rerank_candidate_blocks(heap_relation_ptr, candidates) };
            Ok(())
        },
        |candidate| unsafe {
            exact_heap_source_inner_product(
                heap_relation_ptr,
                snapshot_pg,
                slot,
                indexed_attribute,
                query.values(),
                candidate.heap_tid,
            )
        },
    );

    unsafe { pg_sys::ExecDropSingleTupleTableSlot(slot) };
    result
}

fn heap_rerank_prefetch_block_numbers(
    candidates: &[SpireScoredScanCandidate],
) -> Vec<pg_sys::BlockNumber> {
    let mut block_numbers = candidates
        .iter()
        .map(|candidate| candidate.heap_tid.block_number)
        .collect::<Vec<_>>();
    block_numbers.sort_unstable();
    block_numbers.dedup();
    block_numbers
}

unsafe fn prefetch_heap_rerank_candidate_blocks(
    heap_relation: pg_sys::Relation,
    candidates: &[SpireScoredScanCandidate],
) {
    let block_numbers = heap_rerank_prefetch_block_numbers(candidates);
    if block_numbers.is_empty() {
        return;
    }
    unsafe { prefetch_heap_rerank_blocks(heap_relation, &block_numbers) };
}

#[cfg(feature = "pg18")]
unsafe fn prefetch_heap_rerank_blocks(
    heap_relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
) {
    let mut state = crate::am::stream::BlockSequencePrefetchState::new(block_numbers.to_vec());
    let stream = unsafe {
        pg_sys::read_stream_begin_relation(
            pg_sys::READ_STREAM_DEFAULT as i32,
            ptr::null_mut(),
            heap_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            Some(crate::am::stream::block_sequence_prefetch_cb),
            (&mut state as *mut crate::am::stream::BlockSequencePrefetchState).cast(),
            std::mem::size_of::<pg_sys::BlockNumber>(),
        )
    };

    loop {
        let mut per_buffer_data = ptr::null_mut();
        let buffer = unsafe { pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data) };
        if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
            break;
        }
        unsafe { pg_sys::ReleaseBuffer(buffer) };
    }

    unsafe { pg_sys::read_stream_end(stream) };
}

#[cfg(not(feature = "pg18"))]
unsafe fn prefetch_heap_rerank_blocks(
    heap_relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
) {
    for block_number in block_numbers {
        unsafe {
            pg_sys::PrefetchBuffer(
                heap_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                *block_number,
            );
        }
    }
}

fn resolve_scan_heap_relation(scan: pg_sys::IndexScanDesc) -> ResolvedScanHeapRelation {
    if scan.is_null() {
        pgrx::error!("ec_spire heap rerank received a null scan descriptor");
    }

    // SAFETY: `scan` is non-null and owned by PostgreSQL for this AM callback.
    let scan_ref = unsafe { &*scan };
    if !scan_ref.heapRelation.is_null() {
        return ResolvedScanHeapRelation::borrowed(scan_ref.heapRelation);
    }

    // SAFETY: `indexRelation` comes from the live scan descriptor.
    let heap_oid = unsafe { pg_sys::IndexGetRelation((*scan_ref.indexRelation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        pgrx::error!("ec_spire heap rerank could not resolve heap relation");
    }
    // SAFETY: `heap_oid` was resolved from the scan's index relation and is
    // closed by `ResolvedScanHeapRelation` when ownership is needed.
    let relation = unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    if relation.is_null() {
        pgrx::error!("ec_spire heap rerank failed to open heap relation");
    }
    ResolvedScanHeapRelation::owned(relation)
}

fn resolve_scan_snapshot(scan: pg_sys::IndexScanDesc) -> pg_sys::Snapshot {
    if scan.is_null() {
        pgrx::error!("ec_spire heap rerank received a null scan descriptor");
    }

    // SAFETY: `scan` is non-null and owned by PostgreSQL for this AM callback.
    let scan_ref = unsafe { &*scan };
    if !scan_ref.xs_snapshot.is_null() {
        return scan_ref.xs_snapshot;
    }

    // SAFETY: Reads PostgreSQL backend-local active snapshot state.
    let active_snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if !active_snapshot.is_null() {
        return active_snapshot;
    }

    pgrx::error!("ec_spire heap rerank requires an executor or active snapshot");
}

unsafe fn allocate_heap_slot(
    heap_relation: pg_sys::Relation,
) -> Result<*mut pg_sys::TupleTableSlot, String> {
    let slot = unsafe {
        pg_sys::MakeSingleTupleTableSlot(
            (*heap_relation).rd_att,
            pg_sys::table_slot_callbacks(heap_relation),
        )
    };
    if slot.is_null() {
        return Err("ec_spire heap rerank failed to allocate a heap tuple slot".to_owned());
    }
    Ok(slot)
}

unsafe fn exact_heap_source_inner_product(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    indexed_attribute: source::IndexedVectorAttribute,
    query: &[f32],
    heap_tid: ItemPointer,
) -> Result<Option<f32>, String> {
    let Some(source_vector) = unsafe {
        load_indexed_source_vector_from_heap_row(
            heap_relation,
            snapshot,
            slot,
            indexed_attribute,
            heap_tid,
            "ec_spire heap rerank source vector",
        )
    }?
    else {
        return Ok(None);
    };
    exact_source_inner_product(query, &source_vector).map(Some)
}

pub(super) unsafe fn load_indexed_source_vector_from_heap_row(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    indexed_attribute: source::IndexedVectorAttribute,
    heap_tid: ItemPointer,
    label: &str,
) -> Result<Option<Vec<f32>>, String> {
    if !unsafe { fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot)? } {
        return Ok(None);
    }
    let datum = unsafe { required_slot_datum(slot, indexed_attribute.attnum, label)? };
    let result =
        unsafe { indexed_vector_datum_to_source_vector(datum, indexed_attribute.kind, label) };
    unsafe { pg_sys::ExecClearTuple(slot) };
    result.map(Some)
}

unsafe fn fetch_heap_row_version(
    heap_relation: pg_sys::Relation,
    heap_tid: ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
) -> Result<bool, String> {
    let mut tid = pg_sys::ItemPointerData::default();
    pgrx::itemptr::item_pointer_set_all(&mut tid, heap_tid.block_number, heap_tid.offset_number);
    unsafe { pg_sys::ExecClearTuple(slot) };
    let fetched =
        unsafe { pg_sys::table_tuple_fetch_row_version(heap_relation, &mut tid, snapshot, slot) };
    if !fetched {
        return Ok(false);
    }
    Ok(true)
}

unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
) -> Result<pg_sys::Datum, String> {
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    let attr_index = usize::try_from(attnum - 1)
        .map_err(|_| "ec_spire heap rerank attribute number must be positive".to_owned())?;
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        return Err(format!("ec_spire does not support NULL {label}"));
    }
    Ok(unsafe { *(*slot).tts_values.add(attr_index) })
}

unsafe fn indexed_vector_datum_to_source_vector(
    datum: pg_sys::Datum,
    kind: source::IndexedVectorKind,
    label: &str,
) -> Result<Vec<f32>, String> {
    let bytes = unsafe { detoasted_varlena_bytes(datum, label)? };
    match kind {
        source::IndexedVectorKind::Ecvector => crate::unpack_raw_f32(&bytes, label),
        source::IndexedVectorKind::Tqvector => tqvector_bytes_to_source_vector(&bytes, label),
    }
}

fn tqvector_bytes_to_source_vector(bytes: &[u8], label: &str) -> Result<Vec<f32>, String> {
    let (dimensions, bits, seed, gamma, code) =
        crate::unpack(bytes).map_err(|e| format!("{label} is invalid tqvector: {e}"))?;
    let mut full_payload = Vec::with_capacity(size_of::<f32>() + code.len());
    full_payload.extend_from_slice(&gamma.to_le_bytes());
    full_payload.extend_from_slice(code);
    let quantizer = ProdQuantizer::cached(usize::from(dimensions), bits, seed);
    Ok(quantizer.decode_approximate(&full_payload))
}

unsafe fn detoasted_varlena_bytes(datum: pg_sys::Datum, label: &str) -> Result<Vec<u8>, String> {
    if datum.is_null() {
        return Err(format!("ec_spire does not support NULL {label}"));
    }
    let original = datum.cast_mut_ptr::<c_void>().cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    if varlena.is_null() {
        return Err(format!("ec_spire could not detoast {label}"));
    }
    let owned = !ptr::eq(varlena, original);
    let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if owned {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }
    Ok(bytes)
}

fn exact_source_inner_product(query: &[f32], source_vector: &[f32]) -> Result<f32, String> {
    if query.len() != source_vector.len() {
        return Err(format!(
            "ec_spire heap rerank dimension mismatch: query dim {}, heap dim {}",
            query.len(),
            source_vector.len()
        ));
    }
    if source_vector.iter().any(|value| !value.is_finite()) {
        return Err("ec_spire heap rerank source vector contains a non-finite value".to_owned());
    }
    let score = source::inner_product(query, source_vector);
    if !score.is_finite() {
        return Err("ec_spire heap rerank produced a non-finite score".to_owned());
    }
    Ok(score)
}
