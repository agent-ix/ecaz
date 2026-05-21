pub(super) fn set_scan_heap_tid(
    scan_output: &mut crate::am::common::scan_output::IndexScanOutput<'_>,
    heap_tid: ItemPointer,
) {
    scan_output.set_heap_tid(heap_tid);
}

pub(super) fn set_scan_orderby_score(
    scan_output: &mut crate::am::common::scan_output::IndexScanOutput<'_>,
    score: f32,
) {
    scan_output.set_orderby_score(
        score,
        "ec_spire scan orderby values",
        "ec_spire scan orderby nulls",
    );
}

pub(super) fn clear_scan_orderby_output(
    scan_output: &mut crate::am::common::scan_output::IndexScanOutput<'_>,
) {
    scan_output.clear_orderby_output();
}

pub(super) struct ResolvedScanHeapRelation {
    relation: pg_sys::Relation,
    _owned: Option<crate::storage::relation_guard::HeapRelationGuard>,
}

impl ResolvedScanHeapRelation {
    fn borrowed(relation: pg_sys::Relation) -> Self {
        Self {
            relation,
            _owned: None,
        }
    }

    fn owned(guard: crate::storage::relation_guard::HeapRelationGuard) -> Self {
        let relation = guard.as_ptr();
        Self {
            relation,
            _owned: Some(guard),
        }
    }

    pub(super) fn as_ptr(&self) -> pg_sys::Relation {
        self.relation
    }
}

struct SpireIndexScanView<'a> {
    scan_ref: &'a mut pg_sys::IndexScanDescData,
}

impl<'a> SpireIndexScanView<'a> {
    unsafe fn from_raw(scan: pg_sys::IndexScanDesc, label: &str) -> Self {
        // SAFETY: AM callbacks supply a live IndexScanDesc for the duration of
        // the callback; null is rejected before safe accessors are exposed.
        let Some(scan_ref) = (unsafe { scan.as_mut() }) else {
            pgrx::error!("ec_spire {label} received a null scan descriptor");
        };
        Self { scan_ref }
    }

    fn index_relation(&self) -> pg_sys::Relation {
        self.scan_ref.indexRelation
    }

    fn clear_recheck_flags_and_orderby_outputs(&mut self) {
        self.scan_ref.xs_recheck = false;
        self.scan_ref.xs_recheckorderby = false;
        self.scan_ref.xs_orderbyvals = ptr::null_mut();
        self.scan_ref.xs_orderbynulls = ptr::null_mut();
    }

    fn mark_tuple_output_current(&mut self) {
        self.scan_ref.xs_recheck = false;
        self.scan_ref.xs_recheckorderby = false;
    }

    fn opaque_mut(&mut self, label: &str) -> &mut SpireScanOpaque {
        let opaque_ptr = self.scan_ref.opaque.cast::<SpireScanOpaque>();
        if opaque_ptr.is_null() {
            pgrx::error!("ec_spire {label} missing scan opaque state");
        }
        // SAFETY: ambeginscan allocates SpireScanOpaque into scan.opaque, and
        // AM callbacks serialize mutable access to the current scan descriptor.
        unsafe { &mut *opaque_ptr }
    }

    fn take_opaque_for_end_scan(&mut self) -> Option<*mut SpireScanOpaque> {
        let opaque_ptr = self.scan_ref.opaque.cast::<SpireScanOpaque>();
        if opaque_ptr.is_null() {
            return None;
        }
        self.scan_ref.opaque = ptr::null_mut();
        Some(opaque_ptr)
    }

    fn heap_relation(&self) -> ResolvedScanHeapRelation {
        if !self.scan_ref.heapRelation.is_null() {
            return ResolvedScanHeapRelation::borrowed(self.scan_ref.heapRelation);
        }

        // SAFETY: the scan descriptor view was constructed from a live AM
        // callback scan; indexRelation is live for relation resolution here.
        let heap_oid =
            unsafe { crate::storage::relation::index_heap_relation_oid(self.index_relation()) };
        if heap_oid == pg_sys::InvalidOid {
            pgrx::error!("ec_spire heap rerank could not resolve heap relation");
        }
        let Some(relation) =
            crate::storage::relation_guard::HeapRelationGuard::try_access_share(heap_oid)
        else {
            pgrx::error!("ec_spire heap rerank failed to open heap relation");
        };
        ResolvedScanHeapRelation::owned(relation)
    }

    fn snapshot(&self) -> pg_sys::Snapshot {
        if !self.scan_ref.xs_snapshot.is_null() {
            return self.scan_ref.xs_snapshot;
        }

        if let Some(active_snapshot) = crate::storage::snapshot_guard::active_snapshot() {
            return active_snapshot;
        }

        pgrx::error!("ec_spire heap rerank requires an executor or active snapshot");
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
    // SAFETY: root_control belongs to this open relation and stores the active
    // epoch manifest tuple id; page helper returns owned bytes.
    let epoch_bytes = page::read_object_tuple(index_relation, root_control.epoch_manifest_tid)?;
    // SAFETY: root_control belongs to this open relation and stores the active
    // object manifest tuple id; page helper returns owned bytes.
    let object_bytes = page::read_object_tuple(index_relation, root_control.object_manifest_tid)?;
    // SAFETY: root_control belongs to this open relation and stores the active
    // placement-directory tuple id; page helper returns owned bytes.
    let placement_bytes =
        page::read_object_tuple(index_relation, root_control.placement_directory_tid)?;
    // SAFETY: root_control belongs to this relation and names the local store
    // config for the same active epoch manifest set.
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
        .find(|placement| placement.node_id != super::meta::SPIRE_LOCAL_NODE_ID)
    else {
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
    // SAFETY: root_control belongs to this open relation and stores the active
    // local-store config tuple id; page helper returns owned bytes.
    let bytes = page::read_object_tuple(index_relation, root_control.local_store_config_tid)?;
    SpireLocalStoreConfig::decode(&bytes)
}

unsafe fn decode_scan_orderby_query(orderbys: pg_sys::ScanKey) -> Result<SpireScanQuery, String> {
    if orderbys.is_null() {
        return Err("ec_spire amrescan received null order-by scan keys".to_owned());
    }

    // SAFETY: orderbys was checked non-null and points at PostgreSQL's first
    // ORDER BY ScanKey for the active rescan callback.
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
    let scan_view =
        unsafe { SpireIndexScanView::from_raw(scan, "heap rerank candidate preparation") };
    let heap_relation = scan_view.heap_relation();
    let heap_relation_ptr = heap_relation.as_ptr();
    let snapshot_pg = scan_view.snapshot();
    // SAFETY: the scan view proves this is the live IndexScanDesc for this scan
    // path; indexRelation is read only to resolve the indexed vector attribute.
    let indexed_attribute = unsafe {
        source::resolve_indexed_vector_attribute(
            heap_relation_ptr,
            scan_view.index_relation(),
            "ec_spire heap rerank indexed column",
        )
    };
    let slot = unsafe { allocate_heap_slot(heap_relation_ptr) }?;
    // SAFETY: the resolved heap relation/snapshot and allocated tuple slot are
    // live for the duration of candidate preparation.
    let mut heap_reader = unsafe {
        crate::am::common::heap_slot::HeapSlotReader::from_raw(
            heap_relation_ptr,
            snapshot_pg,
            slot.as_ptr(),
            "ec_spire",
        )
    }?;

    let result = prepare_single_level_snapshot_scan_candidates_with_prefetch(
        snapshot,
        object_store,
        query,
        options,
        |candidates| {
            unsafe { prefetch_heap_rerank_candidate_blocks(heap_relation_ptr, candidates) };
            Ok(())
        },
        |candidate| {
            exact_heap_source_inner_product(
                &mut heap_reader,
                indexed_attribute,
                query.values(),
                candidate.heap_tid,
            )
        },
    );

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
    crate::am::stream::prefetch_relation_blocks(
        heap_relation,
        block_numbers,
        "ec_spire heap rerank",
    );
}

unsafe fn allocate_heap_slot(
    heap_relation: pg_sys::Relation,
) -> Result<crate::storage::slot_guard::TupleTableSlotGuard<'static>, String> {
    crate::storage::slot_guard::TupleTableSlotGuard::single_for_heap(heap_relation)
        .ok_or_else(|| "ec_spire heap rerank failed to allocate a heap tuple slot".to_owned())
}

fn exact_heap_source_inner_product(
    heap_reader: &mut crate::am::common::heap_slot::HeapSlotReader<'_>,
    indexed_attribute: source::IndexedVectorAttribute,
    query: &[f32],
    heap_tid: ItemPointer,
) -> Result<Option<f32>, String> {
    let Some(source_vector) = load_indexed_source_vector_from_heap_row(
        heap_reader,
        indexed_attribute,
        heap_tid,
        "ec_spire heap rerank source vector",
    )?
    else {
        return Ok(None);
    };
    exact_source_inner_product(query, &source_vector).map(Some)
}

pub(super) fn load_indexed_source_vector_from_heap_row(
    heap_reader: &mut crate::am::common::heap_slot::HeapSlotReader<'_>,
    indexed_attribute: source::IndexedVectorAttribute,
    heap_tid: ItemPointer,
    label: &str,
) -> Result<Option<Vec<f32>>, String> {
    if !heap_reader.fetch_row_version(heap_tid)? {
        return Ok(None);
    }
    let datum = heap_reader.required_datum(indexed_attribute.attnum, label)?;
    // SAFETY: datum is the non-null vector datum read from the fetched slot.
    let result =
        unsafe { indexed_vector_datum_to_source_vector(datum, indexed_attribute.kind, label) };
    heap_reader.clear();
    result.map(Some)
}

unsafe fn indexed_vector_datum_to_source_vector(
    datum: pg_sys::Datum,
    kind: source::IndexedVectorKind,
    label: &str,
) -> Result<Vec<f32>, String> {
    // SAFETY: datum is a non-null varlena vector datum read from a live slot.
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
    // SAFETY: datum is a non-null varlena value borrowed from PostgreSQL; pgrx
    // detoasts/copies it into owned bytes before the slot is cleared.
    unsafe { DetoastedVarlena::packed_from_datum(datum) }
        .ok_or_else(|| format!("ec_spire could not detoast {label}"))
        .map(|datum| datum.to_vec())
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
