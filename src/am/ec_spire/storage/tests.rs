#[cfg(test)]
mod tests {
    use super::super::meta::{
        SpireLocalStoreDescriptor, SpireLocalStoreState, SpirePlacementEntry, SpirePlacementState,
    };
    use super::{
        decode_leaf_v2_local_vec_id, is_delete_delta_assignment, is_visible_primary_assignment,
        is_visible_primary_assignment_ref, is_visible_scored_assignment,
        local_store_config_from_relation_plan, plan_local_store_relations,
        relation_object_prefetch_groups,
        spire_local_store_relation_name, SpireDeltaPartitionObject, SpireLeafAssignmentRow,
        SpireLeafPartitionObject, SpireLocalObjectStore, SpirePartitionObjectHeader,
        SpirePartitionObjectKind, SpireRoutingChildEntry, SpireRoutingPartitionObject, SpireVecId,
        SpireVecIdKind,
        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
        SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, SPIRE_LOCAL_VEC_ID_DISCRIMINATOR,
        SPIRE_PAYLOAD_FORMAT_NONE, SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT, SPIRE_VEC_ID_MAX_BYTES,
    };
    use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

    fn routing_children() -> Vec<SpireRoutingChildEntry> {
        vec![
            SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid: 17,
                centroid: vec![1.0, 0.0],
            },
            SpireRoutingChildEntry {
                centroid_index: 1,
                child_pid: 18,
                centroid: vec![-1.0, 0.0],
            },
        ]
    }

    fn leaf_v2_assignment(local_vec_seq: u64, payload_len: usize) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(local_vec_seq),
            heap_tid: ItemPointer {
                block_number: 100 + local_vec_seq as u32,
                offset_number: local_vec_seq as u16,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: local_vec_seq as f32 / 10.0,
            encoded_payload: vec![local_vec_seq as u8; payload_len],
        }
    }


    include!("tests/vec_and_routing.rs");
    include!("tests/local_store_plan.rs");
    include!("tests/assignment.rs");
    include!("tests/leaf.rs");
    include!("tests/delta.rs");
    include!("tests/local_store.rs");
    include!("tests/relation_prefetch.rs");
}
