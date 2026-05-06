#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpirePartitionObjectKind {
    Root = 1,
    Internal = 2,
    Leaf = 3,
    Delta = 4,
}

impl SpirePartitionObjectKind {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Root),
            2 => Ok(Self::Internal),
            3 => Ok(Self::Leaf),
            4 => Ok(Self::Delta),
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
