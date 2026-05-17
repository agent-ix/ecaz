WITH idx(name, oid) AS (
  VALUES
    ('n32', 'task28_ivf_a3_100k_slack50_n32_idx'::regclass),
    ('n64', 'task28_ivf_a3_100k_slack50_n64_idx'::regclass)
),
ownership AS (
  SELECT
    idx.name,
    own.*
  FROM idx
  CROSS JOIN LATERAL ec_ivf_index_page_ownership(idx.oid) AS own
)
SELECT
  name AS idx,
  count(*) FILTER (WHERE posting_tuples > 0) AS posting_blocks,
  sum(unused_line_pointers) AS unused_lps,
  sum(deleted_posting_tuples) AS deleted_postings,
  sum(posting_tuples) AS posting_tuples,
  sum(heap_tid_refs) AS heap_tid_refs,
  count(*) FILTER (WHERE distinct_lists > 1) AS cross_list_blocks,
  count(*) FILTER (WHERE posting_tuples > 0 AND non_posting_tuples > 0) AS mixed_blocks
FROM ownership
GROUP BY name
ORDER BY name;
