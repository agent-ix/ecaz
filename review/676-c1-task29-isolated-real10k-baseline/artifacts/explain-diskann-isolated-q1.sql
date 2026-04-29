SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET enable_sort = off;
SET ec_diskann.list_size = 64;
SELECT source::text AS qvec FROM task29_diskann_real10k_queries ORDER BY id LIMIT 1 \gset
PREPARE q(real[], bigint) AS
  SELECT id
  FROM task29_diskann_real10k_corpus
  ORDER BY embedding <#> $1::real[]
  LIMIT $2;
EXPLAIN (ANALYZE, COSTS OFF, BUFFERS)
EXECUTE q(:'qvec'::real[], 10);
