#[derive(Debug, Clone)]
pub(super) struct SpireLocalObjectStore {
    local_store_id: u32,
    store_relid: u32,
    pages: DataPageChain,
}

impl SpireLocalObjectStore {
    pub(super) fn with_default_page_size(store_relid: u32) -> Result<Self, String> {
        Self::new(store_relid, DEFAULT_PAGE_SIZE)
    }

    pub(super) fn new(store_relid: u32, page_size: usize) -> Result<Self, String> {
        Self::new_for_store(SPIRE_SINGLE_LOCAL_STORE_ID, store_relid, page_size)
    }

    pub(super) fn for_store_descriptor(
        store: &SpireLocalStoreDescriptor,
        page_size: usize,
    ) -> Result<Self, String> {
        if store.state != SpireLocalStoreState::Available {
            return Err(format!(
                "ec_spire local object store cannot write {:?} store",
                store.state
            ));
        }
        Self::new_for_store(store.local_store_id, store.store_relid, page_size)
    }

    fn new_for_store(
        local_store_id: u32,
        store_relid: u32,
        page_size: usize,
    ) -> Result<Self, String> {
        if store_relid == 0 {
            return Err("ec_spire local object store relid 0 is invalid".to_owned());
        }
        if page_size == 0 {
            return Err("ec_spire local object store page size 0 is invalid".to_owned());
        }
        Ok(Self {
            local_store_id,
            store_relid,
            pages: DataPageChain::new(page_size),
        })
    }

    pub(super) fn page_count(&self) -> usize {
        self.pages.pages().len()
    }

    pub(super) fn insert_leaf_object(
        &mut self,
        epoch: u64,
        object: &SpireLeafPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = self.pages.insert_raw_tuple(encoded)?;
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

    pub(super) fn insert_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignments: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        validate_leaf_assignments(assignments)?;
        let assignment_count = u32::try_from(assignments.len())
            .map_err(|_| "ec_spire leaf V2 assignment count exceeds u32".to_owned())?;
        let layout = leaf_v2_column_layout(assignments)?;
        let max_segment_rows = leaf_v2_max_segment_rows(
            self.pages.page_size(),
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
            next_segment_locator = self.pages.insert_raw_tuple(encoded_segment)?;
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
        let meta_tid = self.pages.insert_raw_tuple(encoded_meta)?;
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

    pub(super) fn insert_delta_object(
        &mut self,
        epoch: u64,
        object: &SpireDeltaPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = self.pages.insert_raw_tuple(encoded)?;
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

    pub(super) fn insert_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = self.pages.insert_raw_tuple(encoded)?;
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

    pub(super) fn insert_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = self.pages.insert_raw_tuple(encoded)?;
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

    pub(super) fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        let raw = self.read_object_bytes(placement)?;
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
    }

    pub(super) fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        self.validate_local_available_placement(placement)?;
        let raw_meta = self.read_raw_tuple(placement.object_tid)?;
        let meta = SpireLeafPartitionObjectV2Meta::decode(raw_meta)?;
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
            let raw_segment = self.read_raw_tuple(next_locator)?;
            let segment = SpireLeafPartitionObjectV2Segment::decode(raw_segment, &meta)?;
            next_locator = segment.next_segment_locator;
            segments.push(segment);
        }
        if next_locator != ItemPointer::INVALID {
            return Err("ec_spire leaf V2 segment chain has trailing locator".to_owned());
        }
        SpireLeafPartitionObjectV2::new(meta, segments)
    }

    pub(super) fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        self.validate_local_available_placement(placement)?;
        let raw = self.read_raw_tuple(placement.object_tid)?;
        let (mut header, format_version, _) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(raw)?;
        match format_version {
            PARTITION_OBJECT_FORMAT_VERSION_V1 => {
                let expected_len = usize::try_from(placement.object_bytes)
                    .map_err(|_| "ec_spire placement object_bytes exceeds usize".to_owned())?;
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

    pub(super) fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        let raw = self.read_object_bytes(placement)?;
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
    }

    pub(super) fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        let raw = self.read_object_bytes(placement)?;
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
    }

    pub(super) fn read_top_graph_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireTopGraphPartitionObject, String> {
        let raw = self.read_object_bytes(placement)?;
        let object = SpireTopGraphPartitionObject::decode(raw)?;
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

    fn read_object_bytes(&self, placement: &SpirePlacementEntry) -> Result<&[u8], String> {
        self.validate_local_available_placement(placement)?;
        let raw = self.read_raw_tuple(placement.object_tid)?;
        let expected_len = usize::try_from(placement.object_bytes)
            .map_err(|_| "ec_spire placement object_bytes exceeds usize".to_owned())?;
        if raw.len() != expected_len {
            return Err(format!(
                "ec_spire object byte length mismatch: placement {}, tuple {}",
                placement.object_bytes,
                raw.len()
            ));
        }
        Ok(raw)
    }

    fn read_raw_tuple(&self, tid: ItemPointer) -> Result<&[u8], String> {
        let page = self
            .pages
            .get_page(tid.block_number)
            .ok_or_else(|| format!("ec_spire object block {} not found", tid.block_number))?;
        page.raw_tuple(tid)
    }

    fn validate_local_available_placement(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<(), String> {
        if placement.node_id != SPIRE_LOCAL_NODE_ID {
            return Err(format!(
                "ec_spire local object store cannot read node_id {}",
                placement.node_id
            ));
        }
        if placement.local_store_id != self.local_store_id {
            return Err(format!(
                "ec_spire placement local_store_id {} does not match local object store id {}",
                placement.local_store_id, self.local_store_id
            ));
        }
        if placement.store_relid != self.store_relid {
            return Err(format!(
                "ec_spire placement store_relid {} does not match local store relid {}",
                placement.store_relid, self.store_relid
            ));
        }
        if placement.state != SpirePlacementState::Available {
            return Err(format!(
                "ec_spire local object store cannot read {:?} placement",
                placement.state
            ));
        }
        Ok(())
    }
}
