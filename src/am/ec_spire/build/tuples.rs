fn build_spire_ecvector_tuple(
    heap_tid: ItemPointer,
    bytes: &[u8],
    payload_format: SpireAssignmentPayloadFormat,
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
        source_vector,
    }
}

fn build_spire_tqvector_tuple(
    heap_tid: ItemPointer,
    bytes: &[u8],
    payload_format: SpireAssignmentPayloadFormat,
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

unsafe fn detoasted_varlena_bytes(datum: pg_sys::Datum, label: &str) -> Vec<u8> {
    let original = datum.cast_mut_ptr::<c_void>().cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    if varlena.is_null() {
        pgrx::error!("ec_spire could not detoast {label}");
    }
    let owned = !ptr::eq(varlena, original);
    let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if owned {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }
    bytes
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer, context: &str) -> ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_spire {context} received a null heap tid");
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    ItemPointer {
        block_number,
        offset_number,
    }
}

pub(super) unsafe fn resolve_indexed_vector_kind(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    context: &str,
) -> SpireIndexedVectorKind {
    if index_info.is_null() {
        pgrx::error!("ec_spire {context} received a null IndexInfo");
    }
    let index_info = unsafe { &*index_info };
    if index_info.ii_NumIndexAttrs != 1 || index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_spire currently supports single-column indexes only");
    }
    if !index_info.ii_Expressions.is_null() {
        pgrx::error!("ec_spire does not support expression indexes yet");
    }
    if !index_info.ii_Predicate.is_null() {
        pgrx::error!("ec_spire does not support partial indexes yet");
    }

    let attnum = i32::from(index_info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("ec_spire requires a base heap column index key");
    }

    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(attnum as usize - 1)
        .expect("resolved indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_spire indexed column references a dropped column");
    }
    unsafe { resolve_indexed_vector_kind_from_type(att.atttypid) }
        .unwrap_or_else(|| pgrx::error!("ec_spire indexed column must be ecvector or tqvector"))
}

unsafe fn resolve_indexed_vector_kind_from_type(
    type_oid: pg_sys::Oid,
) -> Option<SpireIndexedVectorKind> {
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
        "ecvector" => Some(SpireIndexedVectorKind::Ecvector),
        "tqvector" => Some(SpireIndexedVectorKind::Tqvector),
        _ => None,
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
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
            let index_relid: u32 = (*index_relation).rd_id.into();
            let local_store_config = local_store_config_from_relation_plan(
                SPIRE_INITIAL_EPOCH,
                &local_store_relation_plan,
                local_store_relation_plan
                    .iter()
                    .map(|entry| (entry.local_store_id, index_relid)),
            )
            .unwrap_or_else(|e| pgrx::error!("{e}"));
            let recursive_fanout = options.recursive_fanout();
            page::initialize_root_control_page(index_relation, SpireRootControlState::empty());
            let indexed_vector_kind =
                resolve_indexed_vector_kind(heap_relation, index_info, "ambuild");
            let mut state = SpireBuildState::new(options, indexed_vector_kind);
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
            let index_tuples = if state.scanned_tuples == 0 {
                0.0
            } else if let Some(recursive_fanout) = recursive_fanout {
                publish_relation_recursive_routing_build(
                    index_relation,
                    &state,
                    recursive_fanout,
                    local_store_config,
                )
                .unwrap_or_else(|e| {
                    pgrx::error!("ec_spire recursive populated ambuild failed: {e}")
                }) as f64
            } else {
                publish_relation_partitioned_single_level_build(
                    index_relation,
                    &state,
                    local_store_config,
                )
                .unwrap_or_else(|e| pgrx::error!("ec_spire populated ambuild failed: {e}"))
                    as f64
            };

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = index_tuples;
            result.into_pg()
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambuildempty(_index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            page::initialize_root_control_page(_index_relation, SpireRootControlState::empty());
        })
    }
}

unsafe extern "C-unwind" fn ec_spire_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = &mut *state.cast::<SpireBuildState>();
            let heap_tid = decode_heap_tid(tid, "ambuild");
            let tuple = build_spire_index_tuple(
                values,
                isnull,
                heap_tid,
                state.indexed_vector_kind,
                state.options.assignment_payload_format(),
                "ambuild",
            );
            state.push(tuple);
        })
    }
}
