\set ON_ERROR_STOP on
SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET enable_sort = off;
SET ec_diskann.list_size = 64;
SELECT source::text AS qvec FROM ec_hnsw_real_10k_queries ORDER BY id LIMIT 1 \gset
PREPARE q(real[], integer, bigint, bigint) AS
  SELECT id
  FROM ec_hnsw_real_10k_corpus
  ORDER BY embedding <#> encode_to_ecvector($1::real[], $2::integer, $3::bigint)
  LIMIT $4;
EXPLAIN (ANALYZE, COSTS OFF, BUFFERS)
EXECUTE q(:'qvec'::real[], 4, 42, 10);
