/// Relation-backed SPIRE object store.
///
/// Insert methods take `&self` because PostgreSQL buffer locks and WAL, not
/// Rust ownership of this wrapper, serialize relation mutation.
pub(super) struct SpireRelationObjectStore {
    store_relation: pg_sys::Relation,
    local_store_id: u32,
    store_relid: u32,
}

impl SpireRelationObjectStore {
    pub(super) unsafe fn for_index_relation(
        index_relation: pg_sys::Relation,
    ) -> Result<Self, String> {
        if index_relation.is_null() {
            return Err("ec_spire relation object store needs a valid relation".to_owned());
        }
        let relation_oid = unsafe { (*index_relation).rd_id };
        if relation_oid == pg_sys::InvalidOid {
            return Err("ec_spire relation object store relid is invalid".to_owned());
        }
        let store_relid = relation_oid.into();
        Ok(Self::for_store_relation_id(
            index_relation,
            SPIRE_SINGLE_LOCAL_STORE_ID,
            store_relid,
        ))
    }

    pub(super) fn for_store_relation_id(
        store_relation: pg_sys::Relation,
        local_store_id: u32,
        store_relid: u32,
    ) -> Self {
        Self {
            store_relation,
            local_store_id,
            store_relid,
        }
    }

    pub(super) unsafe fn insert_routing_object(
        &self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire relation object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = unsafe { page::append_object_tuple(self.store_relation, &encoded)? };
        let placement = SpirePlacementEntry::local_store_available_by_id(
            epoch,
            durable_object.header.pid,
            self.local_store_id,
            self.store_relid,
            durable_object.header.object_version,
            object_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) unsafe fn insert_leaf_object_v2_from_rows(
        &self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignments: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire relation object store epoch 0 is invalid".to_owned());
        }
        validate_leaf_assignments(assignments)?;
        let assignment_count = u32::try_from(assignments.len())
            .map_err(|_| "ec_spire leaf V2 assignment count exceeds u32".to_owned())?;
        let (payload_format, payload_stride) = leaf_v2_payload_layout(assignments)?;
        let max_segment_rows = leaf_v2_max_segment_rows(
            pg_sys::BLCKSZ as usize,
            payload_stride,
            LEAF_V2_LOCAL_VEC_ID_STRIDE,
        )?;
        let segment_count = if assignments.is_empty() {
            0
        } else {
            let count = assignments
                .len()
                .checked_add(max_segment_rows - 1)
                .and_then(|value| value.checked_div(max_segment_rows))
                .ok_or_else(|| "ec_spire leaf V2 segment count overflow".to_owned())?;
            u32::try_from(count)
                .map_err(|_| "ec_spire leaf V2 segment count exceeds u32".to_owned())?
        };

        let provisional_first_segment = if assignments.is_empty() {
            ItemPointer::INVALID
        } else {
            ItemPointer {
                block_number: 1,
                offset_number: 1,
            }
        };
        let provisional_meta = SpireLeafPartitionObjectV2Meta::new(
            pid,
            object_version,
            parent_pid,
            assignment_count,
            payload_format,
            u32::try_from(payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds u32".to_owned())?,
            segment_count,
            provisional_first_segment,
            1,
            epoch,
        )?;

        let mut next_segment_locator = ItemPointer::INVALID;
        let mut segment_bytes_total = 0_u64;
        for segment_index in (0..usize::try_from(segment_count).unwrap_or(0)).rev() {
            let row_base = segment_index
                .checked_mul(max_segment_rows)
                .ok_or_else(|| "ec_spire leaf V2 row_base overflow".to_owned())?;
            let rows_end = assignments.len().min(row_base + max_segment_rows);
            let segment = SpireLeafPartitionObjectV2Segment::new(
                &provisional_meta,
                u32::try_from(segment_index)
                    .map_err(|_| "ec_spire leaf V2 segment index exceeds u32".to_owned())?,
                u32::try_from(row_base)
                    .map_err(|_| "ec_spire leaf V2 row_base exceeds u32".to_owned())?,
                next_segment_locator,
                &assignments[row_base..rows_end],
            )?;
            let encoded_segment = segment.encode(&provisional_meta)?;
            segment_bytes_total =
                segment_bytes_total
                    .checked_add(u64::try_from(encoded_segment.len()).map_err(|_| {
                        "ec_spire leaf V2 segment byte length exceeds u64".to_owned()
                    })?)
                    .ok_or_else(|| "ec_spire leaf V2 object byte length overflow".to_owned())?;
            next_segment_locator =
                unsafe { page::append_object_tuple(self.store_relation, &encoded_segment)? };
        }

        let first_segment_locator = if assignments.is_empty() {
            ItemPointer::INVALID
        } else {
            next_segment_locator
        };
        let meta_bytes_len = PARTITION_OBJECT_HEADER_BYTES
            .checked_add(LEAF_V2_META_BODY_BYTES)
            .ok_or_else(|| "ec_spire leaf V2 meta byte length overflow".to_owned())?;
        let object_bytes_total = segment_bytes_total
            .checked_add(
                u64::try_from(meta_bytes_len)
                    .map_err(|_| "ec_spire leaf V2 meta byte length exceeds u64".to_owned())?,
            )
            .ok_or_else(|| "ec_spire leaf V2 object byte length overflow".to_owned())?;
        let object_bytes = u32::try_from(object_bytes_total)
            .map_err(|_| "ec_spire leaf V2 object length exceeds u32".to_owned())?;
        let meta = SpireLeafPartitionObjectV2Meta::new(
            pid,
            object_version,
            parent_pid,
            assignment_count,
            payload_format,
            u32::try_from(payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds u32".to_owned())?,
            segment_count,
            first_segment_locator,
            object_bytes_total,
            epoch,
        )?;
        let encoded_meta = meta.encode()?;
        let meta_tid = unsafe { page::append_object_tuple(self.store_relation, &encoded_meta)? };
        let placement = SpirePlacementEntry::local_store_available_by_id(
            epoch,
            pid,
            self.local_store_id,
            self.store_relid,
            object_version,
            meta_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) unsafe fn insert_delta_object(
        &self,
        epoch: u64,
        object: &SpireDeltaPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire relation object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = unsafe { page::append_object_tuple(self.store_relation, &encoded)? };
        let placement = SpirePlacementEntry::local_store_available_by_id(
            epoch,
            durable_object.header.pid,
            self.local_store_id,
            self.store_relid,
            durable_object.header.object_version,
            object_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) unsafe fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        unsafe {
            self.with_single_tuple_object_bytes(placement, |raw| {
                let object = SpireRoutingPartitionObject::decode(raw)?;
                if object.header.pid != placement.pid {
                    return Err(format!(
                        "ec_spire placement pid {} does not match object pid {}",
                        placement.pid, object.header.pid
                    ));
                }
                if object.header.object_version != placement.object_version {
                    return Err(format!(
                        "ec_spire placement object_version {} does not match object version {}",
                        placement.object_version, object.header.object_version
                    ));
                }
                if object.header.published_epoch_backref == 0
                    || object.header.published_epoch_backref > placement.epoch
                {
                    return Err(format!(
                        "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                        object.header.published_epoch_backref, placement.epoch
                    ));
                }
                Ok(object)
            })
        }
    }

    pub(super) unsafe fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        self.validate_local_available_placement(placement)?;
        let meta = unsafe {
            page::with_pinned_object_tuple(self.store_relation, placement.object_tid, |raw| {
                SpireLeafPartitionObjectV2Meta::decode(raw)
            })?
        };
        if meta.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match leaf V2 pid {}",
                placement.pid, meta.header.pid
            ));
        }
        if meta.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match leaf V2 version {}",
                placement.object_version, meta.header.object_version
            ));
        }
        if meta.header.published_epoch_backref == 0
            || meta.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire leaf V2 published epoch backref {} is not valid for placement epoch {}",
                meta.header.published_epoch_backref, placement.epoch
            ));
        }
        if u64::from(placement.object_bytes) != meta.object_bytes_total {
            return Err(format!(
                "ec_spire placement object_bytes {} does not match leaf V2 total {}",
                placement.object_bytes, meta.object_bytes_total
            ));
        }

        let segment_count = usize::try_from(meta.segment_count)
            .map_err(|_| "ec_spire leaf V2 segment count exceeds usize".to_owned())?;
        let mut segments = Vec::with_capacity(segment_count);
        let mut next_locator = meta.first_segment_locator;
        for _ in 0..segment_count {
            if next_locator == ItemPointer::INVALID {
                return Err("ec_spire leaf V2 segment chain ended early".to_owned());
            }
            let segment = unsafe {
                page::with_pinned_object_tuple(self.store_relation, next_locator, |raw| {
                    SpireLeafPartitionObjectV2Segment::decode(raw, &meta)
                })?
            };
            next_locator = segment.next_segment_locator;
            segments.push(segment);
        }
        if next_locator != ItemPointer::INVALID {
            return Err("ec_spire leaf V2 segment chain has trailing locator".to_owned());
        }
        SpireLeafPartitionObjectV2::new(meta, segments)
    }

    pub(super) unsafe fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        self.validate_local_available_placement(placement)?;
        let header = unsafe {
            page::with_pinned_object_tuple(self.store_relation, placement.object_tid, |raw| {
                let (mut header, format_version, _) =
                    SpirePartitionObjectHeader::decode_prefix_with_format_version(raw)?;
                match format_version {
                    PARTITION_OBJECT_FORMAT_VERSION_V1 => {
                        let expected_len =
                            usize::try_from(placement.object_bytes).map_err(|_| {
                                "ec_spire placement object_bytes exceeds usize".to_owned()
                            })?;
                        if raw.len() != expected_len {
                            return Err(format!(
                                "ec_spire object byte length mismatch: placement {}, tuple {}",
                                placement.object_bytes,
                                raw.len()
                            ));
                        }
                    }
                    PARTITION_OBJECT_FORMAT_VERSION_V2 => {
                        let meta = SpireLeafPartitionObjectV2Meta::decode(raw)?;
                        if u64::from(placement.object_bytes) != meta.object_bytes_total {
                            return Err(format!(
                                "ec_spire placement object_bytes {} does not match leaf V2 total {}",
                                placement.object_bytes, meta.object_bytes_total
                            ));
                        }
                        header = meta.header;
                    }
                    other => {
                        return Err(format!(
                            "ec_spire unsupported partition object format version: {other}"
                        ));
                    }
                }
                Ok(header)
            })?
        };
        if header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, header.pid
            ));
        }
        if header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, header.object_version
            ));
        }
        if header.published_epoch_backref == 0 || header.published_epoch_backref > placement.epoch {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(header)
    }

    pub(super) unsafe fn active_object_tuple_locators(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<Vec<ItemPointer>, String> {
        self.validate_local_available_placement(placement)?;
        let header = unsafe { self.read_object_header(placement)? };
        let mut locators = vec![placement.object_tid];
        if header.kind != SpirePartitionObjectKind::Leaf || header.flags & LEAF_V2_META_FLAG == 0 {
            return Ok(locators);
        }

        let meta = unsafe {
            page::with_pinned_object_tuple(self.store_relation, placement.object_tid, |raw| {
                SpireLeafPartitionObjectV2Meta::decode(raw)
            })?
        };
        if meta.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match leaf V2 pid {}",
                placement.pid, meta.header.pid
            ));
        }
        if meta.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match leaf V2 version {}",
                placement.object_version, meta.header.object_version
            ));
        }

        let mut next_locator = meta.first_segment_locator;
        for _ in 0..meta.segment_count {
            if next_locator == ItemPointer::INVALID {
                return Err("ec_spire leaf V2 segment chain ended early".to_owned());
            }
            locators.push(next_locator);
            let segment = unsafe {
                page::with_pinned_object_tuple(self.store_relation, next_locator, |raw| {
                    SpireLeafPartitionObjectV2Segment::decode(raw, &meta)
                })?
            };
            next_locator = segment.next_segment_locator;
        }
        if next_locator != ItemPointer::INVALID {
            return Err("ec_spire leaf V2 segment chain has trailing locator".to_owned());
        }

        Ok(locators)
    }

    pub(super) unsafe fn prefetch_object_tuple(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<(), String> {
        self.validate_local_available_placement(placement)?;
        unsafe {
            pg_sys::PrefetchBuffer(
                self.store_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                placement.object_tid.block_number,
            );
        }
        Ok(())
    }

    pub(super) unsafe fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        unsafe {
            self.with_single_tuple_object_bytes(placement, |raw| {
                let object = SpireLeafPartitionObject::decode(raw)?;
                if object.header.pid != placement.pid {
                    return Err(format!(
                        "ec_spire placement pid {} does not match object pid {}",
                        placement.pid, object.header.pid
                    ));
                }
                if object.header.object_version != placement.object_version {
                    return Err(format!(
                        "ec_spire placement object_version {} does not match object version {}",
                        placement.object_version, object.header.object_version
                    ));
                }
                if object.header.published_epoch_backref == 0
                    || object.header.published_epoch_backref > placement.epoch
                {
                    return Err(format!(
                        "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                        object.header.published_epoch_backref, placement.epoch
                    ));
                }
                Ok(object)
            })
        }
    }

    pub(super) unsafe fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        unsafe {
            self.with_single_tuple_object_bytes(placement, |raw| {
                let object = SpireDeltaPartitionObject::decode(raw)?;
                if object.header.pid != placement.pid {
                    return Err(format!(
                        "ec_spire placement pid {} does not match object pid {}",
                        placement.pid, object.header.pid
                    ));
                }
                if object.header.object_version != placement.object_version {
                    return Err(format!(
                        "ec_spire placement object_version {} does not match object version {}",
                        placement.object_version, object.header.object_version
                    ));
                }
                if object.header.published_epoch_backref == 0
                    || object.header.published_epoch_backref > placement.epoch
                {
                    return Err(format!(
                        "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                        object.header.published_epoch_backref, placement.epoch
                    ));
                }
                Ok(object)
            })
        }
    }

    pub(super) unsafe fn read_object_bytes(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<Vec<u8>, String> {
        unsafe { self.with_single_tuple_object_bytes(placement, |raw| Ok(raw.to_vec())) }
    }

    unsafe fn with_single_tuple_object_bytes<F, R>(
        &self,
        placement: &SpirePlacementEntry,
        f: F,
    ) -> Result<R, String>
    where
        F: FnOnce(&[u8]) -> Result<R, String>,
    {
        self.validate_local_available_placement(placement)?;
        unsafe {
            page::with_pinned_object_tuple(self.store_relation, placement.object_tid, |raw| {
                let expected_len = usize::try_from(placement.object_bytes)
                    .map_err(|_| "ec_spire placement object_bytes exceeds usize".to_owned())?;
                if raw.len() != expected_len {
                    return Err(format!(
                        "ec_spire object byte length mismatch: placement {}, tuple {}",
                        placement.object_bytes,
                        raw.len()
                    ));
                }
                f(raw)
            })
        }
    }

    fn validate_local_available_placement(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<(), String> {
        if placement.node_id != SPIRE_LOCAL_NODE_ID {
            return Err(format!(
                "ec_spire relation object store cannot read node_id {}",
                placement.node_id
            ));
        }
        if placement.local_store_id != self.local_store_id {
            return Err(format!(
                "ec_spire placement local_store_id {} does not match relation object store id {}",
                placement.local_store_id, self.local_store_id
            ));
        }
        if placement.store_relid != self.store_relid {
            return Err(format!(
                "ec_spire placement store_relid {} does not match relation store relid {}",
                placement.store_relid, self.store_relid
            ));
        }
        if placement.state != SpirePlacementState::Available {
            return Err(format!(
                "ec_spire relation object store cannot read {:?} placement",
                placement.state
            ));
        }
        Ok(())
    }
}

impl SpireObjectReader for SpireRelationObjectStore {
    fn prefetch_object(&self, placement: &SpirePlacementEntry) -> Result<(), String> {
        unsafe { SpireRelationObjectStore::prefetch_object_tuple(self, placement) }
    }

    fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        unsafe { SpireRelationObjectStore::read_object_header(self, placement) }
    }

    fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        unsafe { SpireRelationObjectStore::read_routing_object(self, placement) }
    }

    fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        unsafe { SpireRelationObjectStore::read_leaf_object(self, placement) }
    }

    fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        unsafe { SpireRelationObjectStore::read_leaf_object_v2(self, placement) }
    }

    fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        unsafe { SpireRelationObjectStore::read_delta_object(self, placement) }
    }
}

pub(super) struct SpireRelationObjectStoreSet {
    config: Option<SpireLocalStoreConfig>,
    stores: Vec<SpireRelationObjectStore>,
    opened_relations: Vec<(pg_sys::Relation, pg_sys::LOCKMODE)>,
}

impl SpireRelationObjectStoreSet {
    pub(super) unsafe fn for_index_relation_and_config(
        index_relation: pg_sys::Relation,
        config: SpireLocalStoreConfig,
        lockmode: pg_sys::LOCKMODE,
    ) -> Result<Self, String> {
        if index_relation.is_null() {
            return Err("ec_spire relation object store set needs a valid index relation".to_owned());
        }
        let index_relid: u32 = unsafe { (*index_relation).rd_id }.into();
        let mut stores = Vec::with_capacity(config.stores.len());
        let mut opened_relations = Vec::new();

        for descriptor in &config.stores {
            if descriptor.state != SpireLocalStoreState::Available {
                return Err(format!(
                    "ec_spire cannot open unavailable local_store_id {} for writes",
                    descriptor.local_store_id
                ));
            }
            let store_relation = if descriptor.store_relid == index_relid {
                index_relation
            } else {
                let relid = pg_sys::Oid::from(descriptor.store_relid);
                let relation = unsafe { pg_sys::relation_open(relid, lockmode) };
                if relation.is_null() {
                    return Err(format!(
                        "ec_spire failed to open local_store_id {} relation {}",
                        descriptor.local_store_id, descriptor.store_relid
                    ));
                }
                opened_relations.push((relation, lockmode));
                relation
            };
            stores.push(SpireRelationObjectStore::for_store_relation_id(
                store_relation,
                descriptor.local_store_id,
                descriptor.store_relid,
            ));
        }

        Ok(Self {
            config: Some(config),
            stores,
            opened_relations,
        })
    }

    pub(super) unsafe fn for_index_relation_and_placements(
        index_relation: pg_sys::Relation,
        placement_directory: &SpirePlacementDirectory,
        lockmode: pg_sys::LOCKMODE,
    ) -> Result<Self, String> {
        if index_relation.is_null() {
            return Err("ec_spire relation object store set needs a valid index relation".to_owned());
        }
        let index_relid: u32 = unsafe { (*index_relation).rd_id }.into();
        let mut relid_by_store_id = BTreeMap::<u32, u32>::new();
        for placement in &placement_directory.entries {
            if let Some(existing_relid) =
                relid_by_store_id.insert(placement.local_store_id, placement.store_relid)
            {
                if existing_relid != placement.store_relid {
                    return Err(format!(
                        "ec_spire placement directory maps local_store_id {} to relids {} and {}",
                        placement.local_store_id, existing_relid, placement.store_relid
                    ));
                }
            }
        }

        let mut stores = Vec::with_capacity(relid_by_store_id.len());
        let mut opened_relations = Vec::new();
        for (local_store_id, store_relid) in relid_by_store_id {
            let store_relation = if store_relid == index_relid {
                index_relation
            } else {
                let relid = pg_sys::Oid::from(store_relid);
                let relation = unsafe { pg_sys::relation_open(relid, lockmode) };
                if relation.is_null() {
                    return Err(format!(
                        "ec_spire failed to open local_store_id {local_store_id} relation {store_relid}"
                    ));
                }
                opened_relations.push((relation, lockmode));
                relation
            };
            stores.push(SpireRelationObjectStore::for_store_relation_id(
                store_relation,
                local_store_id,
                store_relid,
            ));
        }

        Ok(Self {
            config: None,
            stores,
            opened_relations,
        })
    }

    pub(super) unsafe fn insert_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe {
            self.store_mut_for_pid(object.header.pid)?
                .insert_routing_object(epoch, object)
        }
    }

    pub(super) unsafe fn insert_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignments: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        unsafe {
            self.store_mut_for_pid(pid)?.insert_leaf_object_v2_from_rows(
                epoch,
                pid,
                object_version,
                parent_pid,
                assignments,
            )
        }
    }

    pub(super) unsafe fn insert_delta_object(
        &mut self,
        epoch: u64,
        object: &SpireDeltaPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe {
            self.store_mut_for_pid(object.header.pid)?
                .insert_delta_object(epoch, object)
        }
    }

    pub(super) unsafe fn active_object_tuple_locators(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<Vec<ItemPointer>, String> {
        unsafe {
            self.store_for_placement(placement)?
                .active_object_tuple_locators(placement)
        }
    }

    pub(super) unsafe fn prefetch_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<(), String> {
        unsafe {
            self.store_for_placement(placement)?
                .prefetch_object_tuple(placement)
        }
    }

    fn store_mut_for_pid(&mut self, pid: u64) -> Result<&mut SpireRelationObjectStore, String> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| "ec_spire relation object store set was opened read-only".to_owned())?;
        let descriptor = *config.store_for_pid(pid)?;
        self.stores
            .iter_mut()
            .find(|store| {
                store.local_store_id == descriptor.local_store_id
                    && store.store_relid == descriptor.store_relid
            })
            .ok_or_else(|| {
                format!(
                    "ec_spire relation object store set is missing writable local_store_id {} relid {}",
                    descriptor.local_store_id, descriptor.store_relid
                )
            })
    }

    fn store_for_placement(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<&SpireRelationObjectStore, String> {
        self.stores
            .iter()
            .find(|store| {
                store.local_store_id == placement.local_store_id
                    && store.store_relid == placement.store_relid
            })
            .ok_or_else(|| {
                format!(
                    "ec_spire relation object store set is missing local_store_id {} relid {}",
                    placement.local_store_id, placement.store_relid
                )
            })
    }
}

impl Drop for SpireRelationObjectStoreSet {
    fn drop(&mut self) {
        for (relation, lockmode) in self.opened_relations.drain(..).rev() {
            unsafe { pg_sys::relation_close(relation, lockmode) };
        }
    }
}

impl SpireObjectReader for SpireRelationObjectStoreSet {
    fn prefetch_object(&self, placement: &SpirePlacementEntry) -> Result<(), String> {
        unsafe { SpireRelationObjectStoreSet::prefetch_object(self, placement) }
    }

    fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        unsafe { self.store_for_placement(placement)?.read_object_header(placement) }
    }

    fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        unsafe { self.store_for_placement(placement)?.read_routing_object(placement) }
    }

    fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        unsafe { self.store_for_placement(placement)?.read_leaf_object(placement) }
    }

    fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        unsafe { self.store_for_placement(placement)?.read_leaf_object_v2(placement) }
    }

    fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        unsafe { self.store_for_placement(placement)?.read_delta_object(placement) }
    }
}
