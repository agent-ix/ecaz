\timing on
ALTER INDEX task28_ivf_pqg100k_g8_n128_idx
  SET (rerank_width = 500);

SELECT
  c.relname,
  c.reloptions
FROM pg_class c
WHERE c.relname = 'task28_ivf_pqg100k_g8_n128_idx';
