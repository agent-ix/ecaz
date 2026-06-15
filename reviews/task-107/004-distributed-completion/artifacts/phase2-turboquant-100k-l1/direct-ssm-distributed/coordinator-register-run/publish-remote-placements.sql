\set ON_ERROR_STOP on
\set consistency_mode strict

WITH remote_nodes AS (
  SELECT
      ordinality::int AS remote_ordinal,
      (remote->>'node_id')::int AS node_id
    FROM jsonb_array_elements(:'remotes_json'::jsonb) WITH ORDINALITY AS t(remote, ordinality)
),
remote_count AS (
  SELECT count(*)::int AS value FROM remote_nodes
),
leaf_plan AS (
  SELECT
      leaf_pid,
      (((row_number() OVER (ORDER BY leaf_pid))::int - 1) % remote_count.value) + 1 AS remote_ordinal
    FROM ec_spire_index_leaf_snapshot(:'coord_index'::regclass::oid)
    CROSS JOIN remote_count
   WHERE placement_state = 'available'
     AND remote_count.value > 0
),
assignment AS (
  SELECT
      leaf_plan.leaf_pid,
      remote_nodes.node_id
    FROM leaf_plan
    JOIN remote_nodes USING (remote_ordinal)
)
SELECT *
  FROM ec_spire_publish_static_remote_placement_nodes_with_mode(
       :'coord_index'::regclass::oid,
       (SELECT COALESCE(array_agg(leaf_pid::bigint ORDER BY leaf_pid), ARRAY[]::bigint[]) FROM assignment),
       (SELECT COALESCE(array_agg(node_id::int ORDER BY leaf_pid), ARRAY[]::int[]) FROM assignment),
       :'consistency_mode'
  );
