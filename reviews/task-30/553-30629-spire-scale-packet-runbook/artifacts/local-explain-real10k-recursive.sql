SET enable_seqscan = off;
SET ec_spire.nprobe = 24;
SET ec_spire.rerank_width = 25;

EXPLAIN (ANALYZE, BUFFERS, COSTS)
SELECT id
FROM task30_spire_scale_local_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_spire_scale_local_queries
  ORDER BY id
  LIMIT 1
)
LIMIT 10;

SELECT recursive_fanout,
       effective_nprobe_per_level,
       nprobe_policy_per_level,
       active_leaf_count
FROM ec_spire_index_options_snapshot('task30_spire_scale_local_idx'::regclass);
