\timing on

ALTER INDEX task28_ivf_pqg10k_g8_idx
  SET (rerank_width = 750);

ALTER INDEX task28_ivf_pqg25k_g8_idx
  SET (rerank_width = 750);

SELECT
  relname,
  reloptions
FROM pg_class
WHERE relname IN ('task28_ivf_pqg10k_g8_idx', 'task28_ivf_pqg25k_g8_idx')
ORDER BY relname;
