\set ON_ERROR_STOP on
WITH remote_nodes AS (
  SELECT ordinality::int AS remote_ordinal, (remote->>'node_id')::int AS node_id
    FROM jsonb_array_elements(:'remotes_json'::jsonb) WITH ORDINALITY AS t(remote, ordinality)
),
remote_count AS (
  SELECT count(*)::int AS value FROM remote_nodes
),
assigned_leaf_pids AS (
  SELECT leaf_plan.leaf_pid
    FROM (
      SELECT leaf_pid, (((row_number() OVER (ORDER BY leaf_pid))::int - 1) % remote_count.value) + 1 AS remote_ordinal
        FROM ec_spire_index_leaf_snapshot(:'coord_index'::regclass::oid)
        CROSS JOIN remote_count
       WHERE placement_state = 'available' AND remote_count.value > 0
    ) AS leaf_plan
    JOIN remote_nodes USING (remote_ordinal)
   WHERE remote_nodes.node_id = :node_id::int
   ORDER BY leaf_plan.leaf_pid
),
selected_assignments AS (
  SELECT *
    FROM ec_spire_index_leaf_base_assignment_snapshot(
         :'coord_index'::regclass::oid,
         (SELECT COALESCE(array_agg(leaf_pid::bigint ORDER BY leaf_pid), ARRAY[]::bigint[]) FROM assigned_leaf_pids))
)
SELECT active_epoch, leaf_pid, parent_pid, object_version, row_index, assignment_flags,
       encode(vec_id, 'hex') AS vec_id_hex,
       encode(row_locator, 'hex') AS row_locator_hex,
       heap_block, heap_offset, heap_ctid, heap_row.id AS row_id,
       payload_format, gamma, encode(encoded_payload, 'hex') AS encoded_payload_hex
  FROM selected_assignments
  JOIN :coord_table AS heap_row ON heap_row.ctid = selected_assignments.heap_ctid::tid
 ORDER BY leaf_pid, row_index;
