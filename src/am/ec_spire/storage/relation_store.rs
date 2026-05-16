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
        let object_tid = if relation_object_tuple_fits(encoded.len()) {
            unsafe { page::append_object_tuple(self.store_relation, &encoded)? }
        } else {
            unsafe {
                self.insert_large_partition_object_chain(
                    durable_object.header,
                    durable_object.dimensions,
                    &encoded,
                )?
            }
        };
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
        let layout = leaf_v2_column_layout(assignments)?;
        let max_segment_rows = leaf_v2_max_segment_rows(
            pg_sys::BLCKSZ as usize,
            layout.payload_stride,
            layout.vec_id_stride,
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
            layout.payload_format,
            u32::try_from(layout.payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds u32".to_owned())?,
            layout.vec_id_kind,
            u16::try_from(layout.vec_id_stride)
                .map_err(|_| "ec_spire leaf V2 vec_id stride exceeds u16".to_owned())?,
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
            layout.payload_format,
            u32::try_from(layout.payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds u32".to_owned())?,
            layout.vec_id_kind,
            u16::try_from(layout.vec_id_stride)
                .map_err(|_| "ec_spire leaf V2 vec_id stride exceeds u16".to_owned())?,
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

    pub(super) unsafe fn insert_top_graph_object(
        &self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire relation object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = if relation_object_tuple_fits(encoded.len()) {
            unsafe { page::append_object_tuple(self.store_relation, &encoded)? }
        } else {
            unsafe {
                self.insert_large_partition_object_chain(
                    durable_object.header,
                    durable_object.dimensions,
                    &encoded,
                )?
            }
        };
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
        self.validate_local_available_placement(placement)?;
        let meta = unsafe {
            page::with_pinned_object_tuple(self.store_relation, placement.object_tid, |raw| {
                decode_relation_object_chain_meta(raw)
            })?
        };
        let object = if let Some(meta) = meta {
            let raw = unsafe { self.read_large_partition_object_bytes(placement, &meta)? };
            SpireRoutingPartitionObject::decode(&raw)?
        } else {
            unsafe {
                self.with_single_tuple_object_bytes(placement, |raw| {
                    SpireRoutingPartitionObject::decode(raw)
                })?
            }
        };
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
                        if header.kind == SpirePartitionObjectKind::Leaf
                            && header.flags & LEAF_V2_META_FLAG != 0
                        {
                            let meta = SpireLeafPartitionObjectV2Meta::decode(raw)?;
                            if u64::from(placement.object_bytes) != meta.object_bytes_total {
                                return Err(format!(
                                    "ec_spire placement object_bytes {} does not match leaf V2 total {}",
                                    placement.object_bytes, meta.object_bytes_total
                                ));
                            }
                            header = meta.header;
                        } else if relation_object_chain_kind_supported(header.kind)
                            && header.flags & PARTITION_OBJECT_V2_CHAIN_META_FLAG != 0
                        {
                            let meta = decode_relation_object_chain_meta(raw)?.ok_or_else(|| {
                                "ec_spire partition object V2 chain meta decode returned no meta"
                                    .to_owned()
                            })?;
                            if u64::from(placement.object_bytes) != meta.object_bytes_total {
                                return Err(format!(
                                    "ec_spire placement object_bytes {} does not match partition object V2 chain total {}",
                                    placement.object_bytes, meta.object_bytes_total
                                ));
                            }
                            header = meta.header;
                        } else {
                            return Err(format!(
                                "ec_spire unsupported partition object V2 header kind {:?} flags {}",
                                header.kind, header.flags
                            ));
                        }
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
        if relation_object_chain_kind_supported(header.kind)
            && header.flags & PARTITION_OBJECT_V2_CHAIN_META_FLAG != 0
        {
            let meta = unsafe {
                page::with_pinned_object_tuple(self.store_relation, placement.object_tid, |raw| {
                    decode_relation_object_chain_meta(raw)
                })?
            }
            .ok_or_else(|| "ec_spire partition object V2 chain meta missing".to_owned())?;
            let mut next_locator = meta.first_segment_locator;
            for _ in 0..meta.segment_count {
                if next_locator == ItemPointer::INVALID {
                    return Err(
                        "ec_spire partition object V2 segment chain ended early".to_owned()
                    );
                }
                locators.push(next_locator);
                let segment = unsafe {
                    page::with_pinned_object_tuple(self.store_relation, next_locator, |raw| {
                        decode_relation_object_chain_segment(raw, &meta)
                    })?
                };
                next_locator = segment.next_segment_locator;
            }
            if next_locator != ItemPointer::INVALID {
                return Err(
                    "ec_spire partition object V2 segment chain has trailing locator".to_owned()
                );
            }
            return Ok(locators);
        }

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

    pub(super) unsafe fn prefetch_object_tuples(
        &self,
        placements: &[SpirePlacementEntry],
    ) -> Result<(), String> {
        let store_keys = [(self.local_store_id, self.store_relid)];
        let groups = relation_object_prefetch_groups(&store_keys, placements)?;
        for group in groups {
            unsafe { self.prefetch_object_blocks(&group.block_numbers)? };
        }
        Ok(())
    }

    unsafe fn prefetch_object_blocks(
        &self,
        block_numbers: &[pg_sys::BlockNumber],
    ) -> Result<(), String> {
        #[cfg(feature = "pg18")]
        unsafe {
            prefetch_relation_blocks_with_read_stream(self.store_relation, block_numbers);
        }

        #[cfg(not(feature = "pg18"))]
        {
            for block_number in block_numbers {
                unsafe {
                    pg_sys::PrefetchBuffer(
                        self.store_relation,
                        pg_sys::ForkNumber::MAIN_FORKNUM,
                        *block_number,
                    );
                }
            }
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

    pub(super) unsafe fn read_top_graph_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireTopGraphPartitionObject, String> {
        self.validate_local_available_placement(placement)?;
        let meta = unsafe {
            page::with_pinned_object_tuple(self.store_relation, placement.object_tid, |raw| {
                decode_relation_object_chain_meta(raw)
            })?
        };
        let object = if let Some(meta) = meta {
            let raw = unsafe { self.read_large_partition_object_bytes(placement, &meta)? };
            SpireTopGraphPartitionObject::decode(&raw)?
        } else {
            unsafe {
                self.with_single_tuple_object_bytes(placement, |raw| {
                    SpireTopGraphPartitionObject::decode(raw)
                })?
            }
        };
        if object.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match top graph pid {}",
                placement.pid, object.header.pid
            ));
        }
        if object.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match top graph version {}",
                placement.object_version, object.header.object_version
            ));
        }
        if object.header.published_epoch_backref == 0
            || object.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire top graph published epoch backref {} is not valid for placement epoch {}",
                object.header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(object)
    }

    pub(super) unsafe fn read_object_bytes(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<Vec<u8>, String> {
        self.validate_local_available_placement(placement)?;
        let meta = unsafe {
            page::with_pinned_object_tuple(self.store_relation, placement.object_tid, |raw| {
                decode_relation_object_chain_meta(raw)
            })?
        };
        if let Some(meta) = meta {
            return unsafe { self.read_large_partition_object_bytes(placement, &meta) };
        }
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

    unsafe fn insert_large_partition_object_chain(
        &self,
        header: SpirePartitionObjectHeader,
        dimensions: u16,
        encoded: &[u8],
    ) -> Result<ItemPointer, String> {
        let chunk_bytes = max_partition_object_chain_segment_payload_bytes()?;
        let segment_count = encoded
            .len()
            .checked_add(chunk_bytes - 1)
            .and_then(|value| value.checked_div(chunk_bytes))
            .ok_or_else(|| {
                "ec_spire partition object V2 segment count overflow".to_owned()
            })?;
        let segment_count_u32 = u32::try_from(segment_count)
            .map_err(|_| "ec_spire partition object V2 segment count exceeds u32".to_owned())?;

        let mut next_segment_locator = ItemPointer::INVALID;
        for segment_index in (0..segment_count).rev() {
            let byte_base = segment_index
                .checked_mul(chunk_bytes)
                .ok_or_else(|| "ec_spire partition object V2 byte_base overflow".to_owned())?;
            let byte_end = encoded.len().min(byte_base + chunk_bytes);
            let segment_no = u32::try_from(segment_index)
                .map_err(|_| "ec_spire partition object V2 segment index exceeds u32".to_owned())?;
            let encoded_segment = encode_relation_object_chain_segment(
                header,
                segment_no,
                u32::try_from(byte_base)
                    .map_err(|_| "ec_spire partition object V2 byte_base exceeds u32".to_owned())?,
                next_segment_locator,
                &encoded[byte_base..byte_end],
            )?;
            next_segment_locator =
                unsafe { page::append_object_tuple(self.store_relation, &encoded_segment)? };
        }

        let encoded_meta = encode_relation_object_chain_meta(
            header,
            dimensions,
            segment_count_u32,
            next_segment_locator,
            u64::try_from(encoded.len()).map_err(|_| {
                "ec_spire partition object V2 object length exceeds u64".to_owned()
            })?,
        )?;
        unsafe { page::append_object_tuple(self.store_relation, &encoded_meta) }
    }

    unsafe fn read_large_partition_object_bytes(
        &self,
        placement: &SpirePlacementEntry,
        meta: &RelationObjectChainMeta,
    ) -> Result<Vec<u8>, String> {
        if u64::from(placement.object_bytes) != meta.object_bytes_total {
            return Err(format!(
                "ec_spire placement object_bytes {} does not match partition object V2 chain total {}",
                placement.object_bytes, meta.object_bytes_total
            ));
        }
        let expected_len = usize::try_from(meta.object_bytes_total)
            .map_err(|_| "ec_spire partition object V2 object length exceeds usize".to_owned())?;
        let mut out = Vec::with_capacity(expected_len);
        let mut next_locator = meta.first_segment_locator;
        for expected_segment_no in 0..meta.segment_count {
            if next_locator == ItemPointer::INVALID {
                return Err("ec_spire partition object V2 segment chain ended early".to_owned());
            }
            let segment = unsafe {
                page::with_pinned_object_tuple(self.store_relation, next_locator, |raw| {
                    decode_relation_object_chain_segment(raw, meta)
                })?
            };
            if segment.segment_no != expected_segment_no {
                return Err(format!(
                    "ec_spire partition object V2 segment number mismatch: got {}, expected {expected_segment_no}",
                    segment.segment_no
                ));
            }
            if usize::try_from(segment.byte_base)
                .map_err(|_| "ec_spire partition object V2 byte_base exceeds usize".to_owned())?
                != out.len()
            {
                return Err(format!(
                    "ec_spire partition object V2 byte_base mismatch: got {}, expected {}",
                    segment.byte_base,
                    out.len()
                ));
            }
            out.extend_from_slice(&segment.payload);
            next_locator = segment.next_segment_locator;
        }
        if next_locator != ItemPointer::INVALID {
            return Err("ec_spire partition object V2 segment chain has trailing locator".to_owned());
        }
        if out.len() != expected_len {
            return Err(format!(
                "ec_spire partition object V2 byte length mismatch: got {}, expected {expected_len}",
                out.len()
            ));
        }
        Ok(out)
    }
}

#[derive(Debug, Clone)]
struct RelationObjectChainMeta {
    header: SpirePartitionObjectHeader,
    dimensions: u16,
    segment_count: u32,
    first_segment_locator: ItemPointer,
    object_bytes_total: u64,
}

#[derive(Debug, Clone)]
struct RelationObjectChainSegment {
    segment_no: u32,
    byte_base: u32,
    next_segment_locator: ItemPointer,
    payload: Vec<u8>,
}

fn relation_object_tuple_fits(payload_len: usize) -> bool {
    raw_tuple_storage_bytes(payload_len) <= usable_page_bytes(pg_sys::BLCKSZ as usize)
}

fn relation_object_chain_kind_supported(kind: SpirePartitionObjectKind) -> bool {
    matches!(
        kind,
        SpirePartitionObjectKind::Root
            | SpirePartitionObjectKind::Internal
            | SpirePartitionObjectKind::TopGraph
    )
}

fn max_partition_object_chain_segment_payload_bytes() -> Result<usize, String> {
    let max_tuple_payload = max_relation_object_tuple_payload_bytes(pg_sys::BLCKSZ as usize)?;
    max_tuple_payload
        .checked_sub(PARTITION_OBJECT_HEADER_BYTES)
        .and_then(|value| value.checked_sub(PARTITION_OBJECT_V2_CHAIN_SEGMENT_PREFIX_BYTES))
        .filter(|value| *value > 0)
        .ok_or_else(|| "ec_spire partition object V2 segment payload capacity is zero".to_owned())
}

fn max_relation_object_tuple_payload_bytes(page_size: usize) -> Result<usize, String> {
    let usable = usable_page_bytes(page_size).min(7_000);
    for payload_len in (1..=usable).rev() {
        if raw_tuple_storage_bytes(payload_len) <= usable {
            return Ok(payload_len);
        }
    }
    Err("ec_spire relation object tuple capacity is zero".to_owned())
}

fn encode_relation_object_chain_meta(
    header: SpirePartitionObjectHeader,
    dimensions: u16,
    segment_count: u32,
    first_segment_locator: ItemPointer,
    object_bytes_total: u64,
) -> Result<Vec<u8>, String> {
    if !relation_object_chain_kind_supported(header.kind) {
        return Err(format!(
            "ec_spire partition object V2 chain meta kind must be Root, Internal, or TopGraph, got {:?}",
            header.kind
        ));
    }
    if dimensions == 0 {
        return Err("ec_spire partition object V2 chain meta dimensions 0 is invalid".to_owned());
    }
    if segment_count == 0 {
        return Err(
            "ec_spire partition object V2 chain meta requires at least one segment".to_owned()
        );
    }
    if first_segment_locator == ItemPointer::INVALID {
        return Err("ec_spire partition object V2 chain meta requires first segment".to_owned());
    }
    let mut header = header;
    header.flags |= PARTITION_OBJECT_V2_CHAIN_META_FLAG;
    let mut out = header.encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V2);
    out.extend_from_slice(&dimensions.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    out.extend_from_slice(&segment_count.to_le_bytes());
    first_segment_locator.encode_into(&mut out);
    out.extend_from_slice(&object_bytes_total.to_le_bytes());
    debug_assert_eq!(
        out.len(),
        PARTITION_OBJECT_HEADER_BYTES + PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES
    );
    Ok(out)
}

fn decode_relation_object_chain_meta(
    input: &[u8],
) -> Result<Option<RelationObjectChainMeta>, String> {
    let (header, format_version, tail) =
        SpirePartitionObjectHeader::decode_prefix_with_format_version(input)?;
    if format_version != PARTITION_OBJECT_FORMAT_VERSION_V2
        || header.flags & PARTITION_OBJECT_V2_CHAIN_META_FLAG == 0
    {
        return Ok(None);
    }
    if !relation_object_chain_kind_supported(header.kind) {
        return Err(format!(
            "ec_spire partition object V2 chain meta kind must be Root, Internal, or TopGraph, got {:?}",
            header.kind
        ));
    }
    if tail.len() != PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES {
        return Err(format!(
            "ec_spire partition object V2 chain meta length mismatch: got {}, expected {PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES}",
            tail.len()
        ));
    }
    let dimensions = u16::from_le_bytes(tail[0..2].try_into().expect("object dimensions"));
    let reserved = u16::from_le_bytes(tail[2..4].try_into().expect("object reserved"));
    if reserved != 0 {
        return Err(format!(
            "ec_spire partition object V2 chain meta reserved bytes must be zero, got {reserved}"
        ));
    }
    let segment_count = u32::from_le_bytes(tail[4..8].try_into().expect("segment count"));
    let first_segment_locator = ItemPointer::decode(&tail[8..14])?;
    let object_bytes_total =
        u64::from_le_bytes(tail[14..22].try_into().expect("object bytes total"));
    if dimensions == 0 {
        return Err("ec_spire partition object V2 chain meta dimensions 0 is invalid".to_owned());
    }
    if segment_count == 0 {
        return Err(
            "ec_spire partition object V2 chain meta segment_count 0 is invalid".to_owned()
        );
    }
    if first_segment_locator == ItemPointer::INVALID {
        return Err(
            "ec_spire partition object V2 chain meta first segment is invalid".to_owned()
        );
    }
    if object_bytes_total == 0 {
        return Err(
            "ec_spire partition object V2 chain meta object_bytes_total 0 is invalid".to_owned()
        );
    }
    Ok(Some(RelationObjectChainMeta {
        header,
        dimensions,
        segment_count,
        first_segment_locator,
        object_bytes_total,
    }))
}

fn encode_relation_object_chain_segment(
    header: SpirePartitionObjectHeader,
    segment_no: u32,
    byte_base: u32,
    next_segment_locator: ItemPointer,
    payload: &[u8],
) -> Result<Vec<u8>, String> {
    if !relation_object_chain_kind_supported(header.kind) {
        return Err(format!(
            "ec_spire partition object V2 chain segment kind must be Root, Internal, or TopGraph, got {:?}",
            header.kind
        ));
    }
    if payload.is_empty() {
        return Err(
            "ec_spire partition object V2 chain segment payload must not be empty".to_owned()
        );
    }
    let mut header = header;
    header.child_count = 0;
    header.flags = PARTITION_OBJECT_V2_CHAIN_SEGMENT_FLAG;
    let mut out = header.encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V2);
    out.extend_from_slice(&segment_no.to_le_bytes());
    out.extend_from_slice(&byte_base.to_le_bytes());
    next_segment_locator.encode_into(&mut out);
    out.extend_from_slice(payload);
    Ok(out)
}

fn decode_relation_object_chain_segment(
    input: &[u8],
    meta: &RelationObjectChainMeta,
) -> Result<RelationObjectChainSegment, String> {
    let (header, format_version, tail) =
        SpirePartitionObjectHeader::decode_prefix_with_format_version(input)?;
    if format_version != PARTITION_OBJECT_FORMAT_VERSION_V2 {
        return Err(format!(
            "ec_spire partition object V2 chain segment format version must be 2, got {format_version}"
        ));
    }
    if header.flags & PARTITION_OBJECT_V2_CHAIN_SEGMENT_FLAG == 0 {
        return Err(format!(
            "ec_spire partition object V2 chain segment missing segment flag: {}",
            header.flags
        ));
    }
    if header.kind != meta.header.kind
        || header.pid != meta.header.pid
        || header.object_version != meta.header.object_version
        || header.published_epoch_backref != meta.header.published_epoch_backref
        || header.level != meta.header.level
        || header.parent_pid != meta.header.parent_pid
    {
        return Err(
            "ec_spire partition object V2 chain segment header does not match meta".to_owned()
        );
    }
    if tail.len() <= PARTITION_OBJECT_V2_CHAIN_SEGMENT_PREFIX_BYTES {
        return Err(format!(
            "ec_spire partition object V2 chain segment too short: got {}, expected more than {PARTITION_OBJECT_V2_CHAIN_SEGMENT_PREFIX_BYTES}",
            tail.len()
        ));
    }
    let segment_no = u32::from_le_bytes(tail[0..4].try_into().expect("segment no"));
    let byte_base = u32::from_le_bytes(tail[4..8].try_into().expect("byte base"));
    let next_segment_locator = ItemPointer::decode(&tail[8..14])?;
    Ok(RelationObjectChainSegment {
        segment_no,
        byte_base,
        next_segment_locator,
        payload: tail[PARTITION_OBJECT_V2_CHAIN_SEGMENT_PREFIX_BYTES..].to_vec(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRelationObjectPrefetchGroup {
    local_store_id: u32,
    store_relid: u32,
    block_numbers: Vec<pg_sys::BlockNumber>,
}

fn relation_object_prefetch_groups(
    store_keys: &[(u32, u32)],
    placements: &[SpirePlacementEntry],
) -> Result<Vec<SpireRelationObjectPrefetchGroup>, String> {
    let available_store_keys = store_keys.iter().copied().collect::<BTreeSet<_>>();
    let mut blocks_by_store = BTreeMap::<(u32, u32), BTreeSet<pg_sys::BlockNumber>>::new();

    for placement in placements {
        if placement.node_id != SPIRE_LOCAL_NODE_ID {
            return Err(format!(
                "ec_spire relation object prefetch cannot read node_id {}",
                placement.node_id
            ));
        }
        if placement.state != SpirePlacementState::Available {
            return Err(format!(
                "ec_spire relation object prefetch cannot read {:?} placement",
                placement.state
            ));
        }

        let store_key = (placement.local_store_id, placement.store_relid);
        if !available_store_keys.contains(&store_key) {
            return Err(format!(
                "ec_spire relation object prefetch is missing local_store_id {} relid {}",
                placement.local_store_id, placement.store_relid
            ));
        }

        blocks_by_store
            .entry(store_key)
            .or_default()
            .insert(placement.object_tid.block_number);
    }

    Ok(blocks_by_store
        .into_iter()
        .map(
            |((local_store_id, store_relid), block_numbers)| SpireRelationObjectPrefetchGroup {
                local_store_id,
                store_relid,
                block_numbers: block_numbers.into_iter().collect(),
            },
        )
        .collect())
}

#[cfg(feature = "pg18")]
unsafe fn prefetch_relation_blocks_with_read_stream(
    relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
) {
    if block_numbers.is_empty() {
        return;
    }

    let mut state = crate::am::stream::BlockSequencePrefetchState::new(block_numbers.to_vec());
    let stream = unsafe {
        pg_sys::read_stream_begin_relation(
            pg_sys::READ_STREAM_DEFAULT as i32,
            ptr::null_mut(),
            relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            Some(crate::am::stream::block_sequence_prefetch_cb),
            (&mut state as *mut crate::am::stream::BlockSequencePrefetchState).cast(),
            size_of::<pg_sys::BlockNumber>(),
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

impl SpireObjectReader for SpireRelationObjectStore {
    fn prefetch_object(&self, placement: &SpirePlacementEntry) -> Result<(), String> {
        unsafe { SpireRelationObjectStore::prefetch_object_tuple(self, placement) }
    }

    fn prefetch_objects(&self, placements: &[SpirePlacementEntry]) -> Result<(), String> {
        unsafe { SpireRelationObjectStore::prefetch_object_tuples(self, placements) }
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

    fn read_top_graph_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireTopGraphPartitionObject, String> {
        unsafe { SpireRelationObjectStore::read_top_graph_object(self, placement) }
    }
}

pub(super) struct SpireRelationObjectStoreSet {
    config: Option<SpireLocalStoreConfig>,
    stores: Vec<SpireRelationObjectStore>,
    store_indexes_by_key: HashMap<(u32, u32), usize>,
    opened_relations: Vec<(pg_sys::Relation, pg_sys::LOCKMODE)>,
}

struct OpenedRelationsGuard {
    relations: Vec<(pg_sys::Relation, pg_sys::LOCKMODE)>,
}

impl OpenedRelationsGuard {
    fn new() -> Self {
        Self {
            relations: Vec::new(),
        }
    }

    fn push(&mut self, relation: pg_sys::Relation, lockmode: pg_sys::LOCKMODE) {
        self.relations.push((relation, lockmode));
    }

    fn into_inner(mut self) -> Vec<(pg_sys::Relation, pg_sys::LOCKMODE)> {
        std::mem::take(&mut self.relations)
    }
}

impl Drop for OpenedRelationsGuard {
    fn drop(&mut self) {
        for (relation, lockmode) in self.relations.drain(..).rev() {
            unsafe { pg_sys::relation_close(relation, lockmode) };
        }
    }
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
        let mut store_indexes_by_key = HashMap::with_capacity(config.stores.len());
        let mut opened_relations = OpenedRelationsGuard::new();

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
                opened_relations.push(relation, lockmode);
                relation
            };
            stores.push(SpireRelationObjectStore::for_store_relation_id(
                store_relation,
                descriptor.local_store_id,
                descriptor.store_relid,
            ));
            let store_index = stores.len() - 1;
            if store_indexes_by_key
                .insert(
                    (descriptor.local_store_id, descriptor.store_relid),
                    store_index,
                )
                .is_some()
            {
                return Err(format!(
                    "ec_spire relation object store set duplicate local_store_id {} relid {}",
                    descriptor.local_store_id, descriptor.store_relid
                ));
            }
        }

        Ok(Self {
            config: Some(config),
            stores,
            store_indexes_by_key,
            opened_relations: opened_relations.into_inner(),
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
        let mut store_indexes_by_key = HashMap::with_capacity(relid_by_store_id.len());
        let mut opened_relations = OpenedRelationsGuard::new();
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
                opened_relations.push(relation, lockmode);
                relation
            };
            stores.push(SpireRelationObjectStore::for_store_relation_id(
                store_relation,
                local_store_id,
                store_relid,
            ));
            let store_index = stores.len() - 1;
            if store_indexes_by_key
                .insert((local_store_id, store_relid), store_index)
                .is_some()
            {
                return Err(format!(
                    "ec_spire relation object store set duplicate local_store_id {local_store_id} relid {store_relid}"
                ));
            }
        }

        Ok(Self {
            config: None,
            stores,
            store_indexes_by_key,
            opened_relations: opened_relations.into_inner(),
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

    pub(super) unsafe fn insert_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe {
            self.store_mut_for_pid(object.header.pid)?
                .insert_top_graph_object(epoch, object)
        }
    }

    pub(super) unsafe fn insert_delta_object_for_base_placement(
        &mut self,
        epoch: u64,
        base_placement: &SpirePlacementEntry,
        object: &SpireDeltaPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe {
            self.store_mut_for_placement(base_placement)?
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

    pub(super) unsafe fn prefetch_objects(
        &self,
        placements: &[SpirePlacementEntry],
    ) -> Result<(), String> {
        let store_keys = self.store_indexes_by_key.keys().copied().collect::<Vec<_>>();
        let groups = relation_object_prefetch_groups(&store_keys, placements)?;
        for group in groups {
            let store_index = *self
                .store_indexes_by_key
                .get(&(group.local_store_id, group.store_relid))
                .ok_or_else(|| {
                    format!(
                        "ec_spire relation object store set is missing local_store_id {} relid {}",
                        group.local_store_id, group.store_relid
                    )
                })?;
            let store = self.stores.get(store_index).ok_or_else(|| {
                format!(
                    "ec_spire relation object store set has stale index for local_store_id {} relid {}",
                    group.local_store_id, group.store_relid
                )
            })?;
            unsafe { store.prefetch_object_blocks(&group.block_numbers)? };
        }
        Ok(())
    }

    fn store_mut_for_pid(&mut self, pid: u64) -> Result<&mut SpireRelationObjectStore, String> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| "ec_spire relation object store set was opened read-only".to_owned())?;
        let descriptor = *config.store_for_pid(pid)?;
        let store_index = *self
            .store_indexes_by_key
            .get(&(descriptor.local_store_id, descriptor.store_relid))
            .ok_or_else(|| {
                format!(
                    "ec_spire relation object store set is missing writable local_store_id {} relid {}",
                    descriptor.local_store_id, descriptor.store_relid
                )
            })?;
        self.stores.get_mut(store_index).ok_or_else(|| {
            format!(
                "ec_spire relation object store set has stale writable index for local_store_id {} relid {}",
                descriptor.local_store_id, descriptor.store_relid
            )
        })
    }

    fn store_for_placement(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<&SpireRelationObjectStore, String> {
        let store_index = *self
            .store_indexes_by_key
            .get(&(placement.local_store_id, placement.store_relid))
            .ok_or_else(|| {
                format!(
                    "ec_spire relation object store set is missing local_store_id {} relid {}",
                    placement.local_store_id, placement.store_relid
                )
            })?;
        self.stores.get(store_index).ok_or_else(|| {
            format!(
                "ec_spire relation object store set has stale index for local_store_id {} relid {}",
                placement.local_store_id, placement.store_relid
            )
        })
    }

    fn store_mut_for_placement(
        &mut self,
        placement: &SpirePlacementEntry,
    ) -> Result<&mut SpireRelationObjectStore, String> {
        if let Some(config) = &self.config {
            config.validate_placement(placement)?;
        }
        let store_index = *self
            .store_indexes_by_key
            .get(&(placement.local_store_id, placement.store_relid))
            .ok_or_else(|| {
                format!(
                    "ec_spire relation object store set is missing writable local_store_id {} relid {}",
                    placement.local_store_id, placement.store_relid
                )
            })?;
        self.stores.get_mut(store_index).ok_or_else(|| {
            format!(
                "ec_spire relation object store set has stale writable index for local_store_id {} relid {}",
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

    fn prefetch_objects(&self, placements: &[SpirePlacementEntry]) -> Result<(), String> {
        unsafe { SpireRelationObjectStoreSet::prefetch_objects(self, placements) }
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

    fn read_top_graph_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireTopGraphPartitionObject, String> {
        unsafe {
            self.store_for_placement(placement)?
                .read_top_graph_object(placement)
        }
    }
}
