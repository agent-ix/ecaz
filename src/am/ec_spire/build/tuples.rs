fn build_spire_ecvector_tuple(
    heap_tid: ItemPointer,
    bytes: &[u8],
    payload_format: SpireAssignmentPayloadFormat,
    vec_id_source_identity: SpireVecIdSourceIdentity,
    context: &str,
) -> SpireBuildTuple {
    let source_vector = crate::unpack_raw_f32(bytes, "ec_spire indexed ecvector column")
        .unwrap_or_else(|e| pgrx::error!("ec_spire {context} found invalid indexed ecvector: {e}"));
    let dimensions = build_dimensions(source_vector.len(), context, "ecvector");
    let assignment = quantizer::encode_assignment_input(payload_format, heap_tid, &source_vector)
        .unwrap_or_else(|e| pgrx::error!("ec_spire {context} found invalid indexed ecvector: {e}"));
    SpireBuildTuple {
        heap_tid,
        dimensions,
        assignment,
        vec_id_source_identity,
        source_vector,
    }
}

fn build_spire_tqvector_tuple(
    heap_tid: ItemPointer,
    bytes: &[u8],
    payload_format: SpireAssignmentPayloadFormat,
    vec_id_source_identity: SpireVecIdSourceIdentity,
    context: &str,
) -> SpireBuildTuple {
    let (dimensions, bits, seed, gamma, code) = crate::unpack(bytes)
        .unwrap_or_else(|e| pgrx::error!("ec_spire {context} found invalid indexed tqvector: {e}"));
    let mut full_payload = Vec::with_capacity(size_of::<f32>() + code.len());
    full_payload.extend_from_slice(&gamma.to_le_bytes());
    full_payload.extend_from_slice(code);
    let quantizer = ProdQuantizer::cached(usize::from(dimensions), bits, seed);
    let source_vector = quantizer.decode_approximate(&full_payload);
    let assignment = quantizer::encode_assignment_input(payload_format, heap_tid, &source_vector)
        .unwrap_or_else(|e| pgrx::error!("ec_spire {context} found invalid indexed tqvector: {e}"));
    SpireBuildTuple {
        heap_tid,
        dimensions,
        assignment,
        vec_id_source_identity,
        source_vector,
    }
}

fn build_dimensions(dimensions: usize, context: &str, label: &str) -> u16 {
    u16::try_from(dimensions).unwrap_or_else(|_| {
        pgrx::error!(
            "ec_spire {context} found invalid indexed {label}: embedding dimension {dimensions} exceeds maximum 65535"
        )
    })
}

/// Detoast a varlena Datum produced by PostgreSQL tuple storage and
/// return its packed bytes as an owned vector. `label` is used only for
/// error messages.
///
/// # Safety
///
/// `datum` must be a valid varlena Datum sourced from a live PostgreSQL
/// tuple slot or HeapTuple — that is, a pointer Datum whose pointed-at
/// region is a well-formed PostgreSQL `varlena` (packed, compressed, or
/// fully-detoasted). The pointed-at region must remain live for the
/// duration of this call; the detoast wrapper copies the bytes before
/// returning, so the resulting `Vec<u8>` outlives the input Datum
/// safely.
unsafe fn detoasted_varlena_bytes(datum: pg_sys::Datum, label: &str) -> Vec<u8> {
    // SAFETY: callers pass a varlena Datum from PostgreSQL tuple storage; the
    // detoast wrapper copies packed bytes before returning.
    unsafe { DetoastedVarlena::packed_from_datum(datum) }
        .unwrap_or_else(|| pgrx::error!("ec_spire could not detoast {label}"))
        .to_vec()
}

pub(super) fn decode_heap_tid(tid: pg_sys::ItemPointer, context: &str) -> ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_spire {context} received a null heap tid");
    }
    crate::am::common::pg_ptr::item_pointer(
        std::ptr::NonNull::new(tid).expect("ec_spire heap tid should be non-null"),
    )
}

pub(super) fn resolve_indexed_tuple_layout(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    options: &options::EcSpireOptions,
    context: &str,
) -> SpireIndexedTupleLayout {
    if heap_relation.is_null() {
        pgrx::error!("ec_spire {context} received a null heap relation");
    }
    if index_info.is_null() {
        pgrx::error!("ec_spire {context} received a null IndexInfo");
    }
    let index_info = crate::am::common::pg_ptr::index_info(
        std::ptr::NonNull::new(index_info).expect("ec_spire IndexInfo should be non-null"),
    );
    if index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_spire currently supports exactly one vector key column");
    }
    if !index_info.ii_Expressions.is_null() {
        pgrx::error!("ec_spire does not support expression indexes yet");
    }
    if !index_info.ii_Predicate.is_null() {
        pgrx::error!("ec_spire does not support partial indexes yet");
    }
    let expected_index_attrs = match options.source_identity {
        options::SpireSourceIdentityProvider::None => 1,
        options::SpireSourceIdentityProvider::Include => 2,
    };
    if index_info.ii_NumIndexAttrs != expected_index_attrs {
        match options.source_identity {
            options::SpireSourceIdentityProvider::None => {
                pgrx::error!("ec_spire INCLUDE columns require WITH (source_identity = 'include')");
            }
            options::SpireSourceIdentityProvider::Include => {
                pgrx::error!(
                    "ec_spire source_identity = 'include' requires exactly one INCLUDE column"
                );
            }
        }
    }

    let attnum = i32::from(index_info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("ec_spire requires a base heap column index key");
    }

    let heap_relation = std::ptr::NonNull::new(heap_relation)
        .unwrap_or_else(|| pgrx::error!("ec_spire build needs a valid heap relation"));
    let tuple_desc = crate::storage::relation::relation_tuple_desc_copy_handle(heap_relation);
    let att = tuple_desc
        .get(attnum as usize - 1)
        .expect("resolved indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_spire indexed column references a dropped column");
    }
    let vector_kind = resolve_indexed_vector_kind_from_type(att.atttypid)
        .unwrap_or_else(|| pgrx::error!("ec_spire indexed column must be ecvector or tqvector"));
    let source_identity = match options.source_identity {
        options::SpireSourceIdentityProvider::None => None,
        options::SpireSourceIdentityProvider::Include => {
            let identity_attnum = i32::from(index_info.ii_IndexAttrNumbers[1]);
            if identity_attnum <= 0 {
                pgrx::error!("ec_spire source_identity INCLUDE column must be a base heap column");
            }
            let identity_att = tuple_desc
                .get(identity_attnum as usize - 1)
                .expect("resolved source identity attribute should exist");
            if identity_att.attisdropped {
                pgrx::error!("ec_spire source_identity INCLUDE column references a dropped column");
            }
            let datum_kind = resolve_source_identity_datum_kind(identity_att.atttypid)
                .unwrap_or_else(|| {
                    pgrx::error!("ec_spire source_identity INCLUDE column must be uuid or bytea")
                });
            Some(SpireSourceIdentityAttribute {
                index_attr_offset: 1,
                heap_attnum: identity_attnum,
                datum_kind,
            })
        }
    };
    SpireIndexedTupleLayout {
        vector_kind,
        source_identity,
    }
}

fn resolve_indexed_vector_kind_from_type(type_oid: pg_sys::Oid) -> Option<SpireIndexedVectorKind> {
    let name = crate::storage::type_info::formatted_base_type_name(type_oid)?;
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    match type_name {
        "ecvector" => Some(SpireIndexedVectorKind::Ecvector),
        "tqvector" => Some(SpireIndexedVectorKind::Tqvector),
        _ => None,
    }
}

fn resolve_source_identity_datum_kind(
    type_oid: pg_sys::Oid,
) -> Option<SpireSourceIdentityDatumKind> {
    match crate::storage::type_info::base_type_oid(type_oid) {
        pg_sys::UUIDOID => Some(SpireSourceIdentityDatumKind::Uuid),
        pg_sys::BYTEAOID => Some(SpireSourceIdentityDatumKind::Bytea16),
        _ => None,
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    pg_am_callback!({
        let total_started = Instant::now();
        let mut timing = SpireBuildTiming::default();
        let setup_started = Instant::now();
        let options = options::relation_options(index_relation);
        let local_store_tablespace_plan =
            options::resolve_local_store_tablespace_plan(index_relation, &options)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
        let local_store_relation_plan = plan_local_store_relations(
            (*index_relation).rd_id.into(),
            local_store_tablespace_plan
                .iter()
                .map(|entry| (entry.local_store_id, entry.tablespace_oid)),
        )
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        let store_relids =
            create_local_store_relations_for_build(index_relation, &local_store_relation_plan)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
        let local_store_config = local_store_config_from_relation_plan(
            SPIRE_INITIAL_EPOCH,
            &local_store_relation_plan,
            store_relids,
        )
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        let recursive_fanout = options.recursive_fanout();
        let top_graph_plan = options
            .top_graph_plan()
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        if top_graph_plan.enabled && recursive_fanout.is_none() {
            pgrx::error!("ec_spire top_graph_enabled requires recursive_fanout >= 2 during build");
        }
        page::initialize_root_control_page(index_relation, SpireRootControlState::empty());
        let tuple_layout =
            resolve_indexed_tuple_layout(heap_relation, index_info, &options, "ambuild");
        let mut state = SpireBuildState::new(options, tuple_layout);
        timing.add_setup(setup_started.elapsed());
        let heap_scan_started = Instant::now();
        let heap_tuples = pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            index_info,
            false,
            false,
            Some(ec_spire_build_callback),
            (&mut state as *mut SpireBuildState).cast(),
            ptr::null_mut(),
        );
        timing.add_heap_scan(heap_scan_started.elapsed());
        let index_tuples = if state.scanned_tuples == 0 {
            0.0
        } else if let Some(recursive_fanout) = recursive_fanout {
            publish_relation_recursive_routing_build(
                index_relation,
                &state,
                recursive_fanout,
                local_store_config,
                &mut timing,
            )
            .unwrap_or_else(|e| pgrx::error!("ec_spire recursive populated ambuild failed: {e}"))
                as f64
        } else {
            publish_relation_partitioned_single_level_build(
                index_relation,
                &state,
                local_store_config,
                &mut timing,
            )
            .unwrap_or_else(|e| pgrx::error!("ec_spire populated ambuild failed: {e}"))
                as f64
        };
        log_spire_ambuild_timing(
            index_relation,
            heap_tuples,
            state.scanned_tuples,
            index_tuples,
            recursive_fanout,
            &timing,
            total_started.elapsed(),
        );

        crate::fault::maybe_fail_palloc("ec_spire ambuild result");
        let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
        result.heap_tuples = heap_tuples;
        result.index_tuples = index_tuples;
        result.into_pg()
    })
}

fn log_spire_ambuild_timing(
    index_relation: pg_sys::Relation,
    heap_tuples: f64,
    scanned_tuples: usize,
    index_tuples: f64,
    recursive_fanout: Option<u32>,
    timing: &SpireBuildTiming,
    total_elapsed: Duration,
) {
    let index_name = crate::storage::relation::relation_name_handle(
        std::ptr::NonNull::new(index_relation)
            .unwrap_or_else(|| pgrx::error!("ec_spire build needs a valid index relation")),
    );
    pgrx::notice!(
        "ec_spire_ambuild_timing index={} phase=complete heap_tuples={} scanned_tuples={} index_tuples={} recursive_fanout={} setup_ms={} heap_scan_ms={} sample_collect_ms={} kmeans_ms={} kmeans_calls={} assignment_ms={} recursive_kmeans_ms={} recursive_kmeans_calls={} recursive_kmeans_max_level={} recursive_assignment_ms={} draft_ms={} top_graph_ms={} pq4_training_ms={} object_store_ms={} publish_ms={} total_ms={}",
        index_name,
        heap_tuples,
        scanned_tuples,
        index_tuples,
        recursive_fanout.unwrap_or(0),
        timing.setup_ms,
        timing.heap_scan_ms,
        timing.sample_collect_ms,
        timing.kmeans_ms,
        timing.kmeans_calls,
        timing.assignment_ms,
        timing.recursive_kmeans_ms,
        timing.recursive_kmeans_calls,
        timing.recursive_kmeans_max_level,
        timing.recursive_assignment_ms,
        timing.draft_ms,
        timing.top_graph_ms,
        0,
        timing.object_store_ms,
        timing.publish_ms,
        elapsed_ms(total_elapsed)
    );
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuildempty(_index_relation: pg_sys::Relation) {
    pg_am_callback!({
        page::initialize_root_control_page(_index_relation, SpireRootControlState::empty());
    })
}

unsafe extern "C-unwind" fn ec_spire_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    pg_am_callback!({
        let state = &mut *state.cast::<SpireBuildState>();
        let heap_tid = decode_heap_tid(tid, "ambuild");
        let tuple = build_spire_index_tuple(
            values,
            isnull,
            heap_tid,
            state.tuple_layout,
            state.options.assignment_payload_format(),
            "ambuild",
        );
        state.push(tuple);
    })
}
