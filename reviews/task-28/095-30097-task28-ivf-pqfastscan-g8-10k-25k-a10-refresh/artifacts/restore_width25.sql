\timing on

ALTER INDEX task28_ivf_pqg10k_g8_idx
  SET (rerank_width = 25);

ALTER INDEX task28_ivf_pqg25k_g8_idx
  SET (rerank_width = 25);

ALTER INDEX task28_ivf_qcmp10k_turboquant_idx
  SET (rerank_width = 25);

ALTER INDEX task28_ivf_postopt25k_n64w25_idx
  SET (rerank_width = 25);

SELECT
  relname,
  reloptions
FROM pg_class
WHERE relname IN (
  'task28_ivf_pqg10k_g8_idx',
  'task28_ivf_pqg25k_g8_idx',
  'task28_ivf_qcmp10k_turboquant_idx',
  'task28_ivf_postopt25k_n64w25_idx'
)
ORDER BY relname;
