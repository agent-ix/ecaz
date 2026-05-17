#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpirePartitionObjectKind {
    Root = 1,
    Internal = 2,
    Leaf = 3,
    Delta = 4,
    TopGraph = 5,
}

impl SpirePartitionObjectKind {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Root),
            2 => Ok(Self::Internal),
            3 => Ok(Self::Leaf),
            4 => Ok(Self::Delta),
            5 => Ok(Self::TopGraph),
            other => Err(format!("ec_spire invalid partition object kind: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireLocalStoreRelationPlanEntry {
    pub(super) local_store_id: u32,
    pub(super) relation_name: String,
    pub(super) tablespace_oid: u32,
}

pub(super) fn spire_local_store_relation_name(
    index_relid: u32,
    local_store_id: u32,
) -> Result<String, String> {
    if index_relid == 0 {
        return Err("ec_spire local store relation name needs a valid index relid".to_owned());
    }

    let relation_name =
        format!("{SPIRE_STORE_RELATION_NAME_PREFIX}_{index_relid}_{local_store_id}");
    let max_identifier_bytes = usize::try_from(pg_sys::NAMEDATALEN)
        .map_err(|_| "ec_spire NAMEDATALEN exceeds usize".to_owned())?
        .saturating_sub(1);
    if relation_name.len() > max_identifier_bytes {
        return Err(format!(
            "ec_spire local store relation name '{relation_name}' exceeds PostgreSQL identifier limit {max_identifier_bytes}"
        ));
    }

    Ok(relation_name)
}

pub(super) fn plan_local_store_relations(
    index_relid: u32,
    tablespace_plan: impl IntoIterator<Item = (u32, u32)>,
) -> Result<Vec<SpireLocalStoreRelationPlanEntry>, String> {
    let mut entries = Vec::new();
    for (local_store_id, tablespace_oid) in tablespace_plan {
        entries.push(SpireLocalStoreRelationPlanEntry {
            local_store_id,
            relation_name: spire_local_store_relation_name(index_relid, local_store_id)?,
            tablespace_oid,
        });
    }
    if entries.is_empty() {
        return Err(
            "ec_spire local store relation plan must include at least one store".to_owned(),
        );
    }

    entries.sort_by_key(|entry| entry.local_store_id);
    for window in entries.windows(2) {
        if window[0].local_store_id == window[1].local_store_id {
            return Err(format!(
                "ec_spire local store relation plan duplicate local_store_id {}",
                window[0].local_store_id
            ));
        }
    }

    Ok(entries)
}

pub(super) fn local_store_config_from_relation_plan(
    generation: u64,
    relation_plan: &[SpireLocalStoreRelationPlanEntry],
    store_relids: impl IntoIterator<Item = (u32, u32)>,
) -> Result<SpireLocalStoreConfig, String> {
    if relation_plan.is_empty() {
        return Err(
            "ec_spire local store relation plan must include at least one store".to_owned(),
        );
    }

    let mut relids_by_store_id = BTreeMap::new();
    for (local_store_id, store_relid) in store_relids {
        if relids_by_store_id
            .insert(local_store_id, store_relid)
            .is_some()
        {
            return Err(format!(
                "ec_spire local store relation plan duplicate created relid for local_store_id {local_store_id}"
            ));
        }
    }

    let mut descriptors = Vec::with_capacity(relation_plan.len());
    for entry in relation_plan {
        let store_relid = relids_by_store_id
            .remove(&entry.local_store_id)
            .ok_or_else(|| {
                format!(
                    "ec_spire local store relation plan missing created relid for local_store_id {}",
                    entry.local_store_id
                )
            })?;
        descriptors.push(SpireLocalStoreDescriptor::available(
            entry.local_store_id,
            store_relid,
            entry.tablespace_oid,
        )?);
    }
    if let Some((local_store_id, _)) = relids_by_store_id.iter().next() {
        return Err(format!(
            "ec_spire local store relation plan has unexpected created relid for local_store_id {local_store_id}"
        ));
    }

    SpireLocalStoreConfig::from_stores(generation, descriptors)
}

unsafe fn spire_aux_store_reloptions() -> Result<pg_sys::Datum, String> {
    let option = std::ffi::CString::new("autovacuum_enabled=false")
        .map_err(|_| "ec_spire auxiliary store reloption contains NUL".to_owned())?;
    let text = unsafe { pg_sys::cstring_to_text(option.as_ptr()) };
    if text.is_null() {
        return Err("ec_spire failed to allocate auxiliary store reloption text".to_owned());
    }

    let mut elems = [unsafe { pg_sys::PointerGetDatum(text.cast()) }];
    let array = unsafe {
        pg_sys::construct_array_builtin(elems.as_mut_ptr(), elems.len() as i32, pg_sys::TEXTOID)
    };
    if array.is_null() {
        return Err("ec_spire failed to allocate auxiliary store reloptions array".to_owned());
    }

    Ok(unsafe { pg_sys::PointerGetDatum(array.cast()) })
}

pub(super) unsafe fn create_local_store_relations_for_build(
    index_relation: pg_sys::Relation,
    relation_plan: &[SpireLocalStoreRelationPlanEntry],
) -> Result<Vec<(u32, u32)>, String> {
    if index_relation.is_null() {
        return Err("ec_spire local store relation creation needs a valid index relation".to_owned());
    }
    if relation_plan.is_empty() {
        return Err(
            "ec_spire local store relation creation needs at least one planned store".to_owned(),
        );
    }

    let index_relid = unsafe { (*index_relation).rd_id };
    if index_relid == pg_sys::InvalidOid {
        return Err("ec_spire local store relation creation needs a valid index relid".to_owned());
    }

    if relation_plan.len() == 1 {
        return Ok(vec![(relation_plan[0].local_store_id, index_relid.into())]);
    }

    let index_class = unsafe { &*(*index_relation).rd_rel };
    let namespace_oid = index_class.relnamespace;
    let owner_oid = index_class.relowner;
    let relpersistence = index_class.relpersistence;
    let mut created = Vec::with_capacity(relation_plan.len());

    for entry in relation_plan {
        let relname = std::ffi::CString::new(entry.relation_name.as_str())
            .map_err(|_| "ec_spire local store relation name contains NUL".to_owned())?;
        let existing = unsafe { pg_sys::get_relname_relid(relname.as_ptr(), namespace_oid) };
        if existing != pg_sys::InvalidOid {
            return Err(format!(
                "ec_spire multi-store REINDEX is not supported yet: auxiliary local store relation '{}' already exists; drop and recreate the index until auxiliary store rebuild lifecycle lands",
                entry.relation_name
            ));
        }

        let tuple_desc = unsafe { pg_sys::CreateTupleDescCopy((*index_relation).rd_att) };
        if tuple_desc.is_null() {
            return Err("ec_spire failed to allocate local store tuple descriptor".to_owned());
        }
        let reloptions = unsafe { spire_aux_store_reloptions()? };

        let store_relid = unsafe {
            pg_sys::heap_create_with_catalog(
                relname.as_ptr(),
                namespace_oid,
                pg_sys::Oid::from(entry.tablespace_oid),
                pg_sys::InvalidOid,
                pg_sys::InvalidOid,
                pg_sys::InvalidOid,
                owner_oid,
                pg_sys::HEAP_TABLE_AM_OID,
                tuple_desc,
                std::ptr::null_mut(),
                pg_sys::RELKIND_RELATION as std::ffi::c_char,
                relpersistence,
                false,
                false,
                pg_sys::OnCommitAction::ONCOMMIT_NOOP,
                reloptions,
                false,
                false,
                true,
                pg_sys::InvalidOid,
                std::ptr::null_mut(),
            )
        };
        unsafe { pg_sys::FreeTupleDesc(tuple_desc) };
        if store_relid == pg_sys::InvalidOid {
            return Err(format!(
                "ec_spire failed to create local_store_id {} relation '{}'",
                entry.local_store_id, entry.relation_name
            ));
        }

        let store_object = pg_sys::ObjectAddress {
            classId: pg_sys::RelationRelationId,
            objectId: store_relid,
            objectSubId: 0,
        };
        let index_object = pg_sys::ObjectAddress {
            classId: pg_sys::RelationRelationId,
            objectId: index_relid,
            objectSubId: 0,
        };
        unsafe {
            pg_sys::recordDependencyOn(
                &store_object,
                &index_object,
                pg_sys::DependencyType::DEPENDENCY_INTERNAL,
            )
        };
        unsafe { pg_sys::CommandCounterIncrement() };

        let Some(store_relation) = crate::storage::relation_guard::RelationGuard::try_open(
            store_relid,
            pg_sys::AccessExclusiveLock as pg_sys::LOCKMODE,
        ) else {
            return Err(format!(
                "ec_spire failed to open created local_store_id {} relation {}",
                entry.local_store_id, store_relid
            ));
        };
        unsafe { page::initialize_aux_store_metadata_page(store_relation.as_ptr()) };
        created.push((entry.local_store_id, store_relid.into()));
    }

    Ok(created)
}
