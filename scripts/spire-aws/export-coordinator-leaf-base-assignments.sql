-- Phase 13e.1 — export coordinator-owned leaf base assignment rows for one
-- remote node. The output is tuples-only TSV under `ecaz dev sql` defaults.

\set ON_ERROR_STOP on

WITH assigned_leaf_pids AS (
  SELECT leaf_pid
    FROM ec_spire_index_leaf_snapshot(:'coord_index'::regclass::oid)
   WHERE placement_state = 'available'
     AND node_id = :node_id::int
   ORDER BY leaf_pid
),
selected_assignments AS (
  SELECT *
    FROM ec_spire_index_leaf_base_assignment_snapshot(
         :'coord_index'::regclass::oid,
         (SELECT COALESCE(array_agg(leaf_pid::bigint ORDER BY leaf_pid), ARRAY[]::bigint[])
            FROM assigned_leaf_pids)
    )
)
SELECT active_epoch,
       leaf_pid,
       parent_pid,
       object_version,
       row_index,
       assignment_flags,
       encode(vec_id, 'hex') AS vec_id_hex,
       encode(row_locator, 'hex') AS row_locator_hex,
       heap_block,
       heap_offset,
       heap_ctid,
       heap_row.id AS row_id,
       payload_format,
       gamma,
       encode(encoded_payload, 'hex') AS encoded_payload_hex
  FROM selected_assignments
  JOIN :coord_table AS heap_row
    ON heap_row.ctid = selected_assignments.heap_ctid::tid
 ORDER BY leaf_pid, row_index;
