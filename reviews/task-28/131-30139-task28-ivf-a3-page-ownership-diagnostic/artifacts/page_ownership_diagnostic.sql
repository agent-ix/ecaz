\pset pager off
\timing on

CREATE OR REPLACE FUNCTION ec_ivf_index_page_ownership(index_oid oid)
RETURNS TABLE (
  block_number bigint,
  line_pointer_count int,
  unused_line_pointers int,
  non_posting_tuples int,
  posting_tuples int,
  live_posting_tuples int,
  deleted_posting_tuples int,
  heap_tid_refs bigint,
  distinct_lists int,
  min_list_id int,
  max_list_id int,
  list_ids text
)
STRICT STABLE
LANGUAGE c
AS '/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz',
   'ec_ivf_index_page_ownership_wrapper';

SELECT
  current_setting('server_version') AS server_version,
  pg_relation_size('task28_ivf_same_slice_n32_idx'::regclass) AS n32_index_bytes,
  pg_relation_size('task28_ivf_same_slice_n64_idx'::regclass) AS n64_index_bytes;

WITH ownership AS (
  SELECT 'n32' AS nlists, *
  FROM ec_ivf_index_page_ownership('task28_ivf_same_slice_n32_idx'::regclass::oid)
  UNION ALL
  SELECT 'n64' AS nlists, *
  FROM ec_ivf_index_page_ownership('task28_ivf_same_slice_n64_idx'::regclass::oid)
)
SELECT
  nlists,
  count(*) AS blocks_with_items,
  count(*) FILTER (WHERE posting_tuples > 0) AS posting_blocks,
  count(*) FILTER (WHERE distinct_lists > 1) AS cross_list_blocks,
  count(*) FILTER (WHERE posting_tuples > 0 AND non_posting_tuples > 0) AS mixed_metadata_posting_blocks,
  count(*) FILTER (WHERE unused_line_pointers > 0) AS blocks_with_unused_line_pointers,
  sum(unused_line_pointers) AS unused_line_pointers,
  sum(posting_tuples) AS posting_tuples,
  sum(live_posting_tuples) AS live_posting_tuples,
  sum(deleted_posting_tuples) AS deleted_posting_tuples,
  sum(heap_tid_refs) AS heap_tid_refs
FROM ownership
GROUP BY nlists
ORDER BY nlists;

WITH ownership AS (
  SELECT 'n32' AS nlists, *
  FROM ec_ivf_index_page_ownership('task28_ivf_same_slice_n32_idx'::regclass::oid)
  UNION ALL
  SELECT 'n64' AS nlists, *
  FROM ec_ivf_index_page_ownership('task28_ivf_same_slice_n64_idx'::regclass::oid)
)
SELECT
  nlists,
  block_number,
  posting_tuples,
  live_posting_tuples,
  deleted_posting_tuples,
  unused_line_pointers,
  non_posting_tuples,
  distinct_lists,
  min_list_id,
  max_list_id,
  list_ids
FROM ownership
WHERE distinct_lists > 1
   OR (posting_tuples > 0 AND non_posting_tuples > 0)
   OR deleted_posting_tuples > 0
   OR unused_line_pointers > 0
ORDER BY nlists, block_number
LIMIT 200;

WITH ownership AS (
  SELECT 'n32' AS nlists, *
  FROM ec_ivf_index_page_ownership('task28_ivf_same_slice_n32_idx'::regclass::oid)
  UNION ALL
  SELECT 'n64' AS nlists, *
  FROM ec_ivf_index_page_ownership('task28_ivf_same_slice_n64_idx'::regclass::oid)
),
per_list AS (
  SELECT
    nlists,
    list_id::int AS list_id,
    count(*) AS block_refs,
    sum(posting_tuples) AS posting_tuples,
    sum(live_posting_tuples) AS live_posting_tuples,
    sum(deleted_posting_tuples) AS deleted_posting_tuples,
    sum(unused_line_pointers) AS unused_line_pointers
  FROM ownership
  CROSS JOIN LATERAL regexp_split_to_table(NULLIF(list_ids, ''), ',') AS list_id
  GROUP BY nlists, list_id
)
SELECT *
FROM per_list
ORDER BY nlists, list_id
LIMIT 160;
