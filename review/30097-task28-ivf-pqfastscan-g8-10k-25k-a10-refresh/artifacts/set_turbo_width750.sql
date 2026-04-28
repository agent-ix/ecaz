\timing on

ALTER INDEX task28_ivf_qcmp10k_turboquant_idx
  SET (rerank_width = 750);

ALTER INDEX task28_ivf_postopt25k_n64w25_idx
  SET (rerank_width = 750);

SELECT
  relname,
  reloptions
FROM pg_class
WHERE relname IN (
  'task28_ivf_qcmp10k_turboquant_idx',
  'task28_ivf_postopt25k_n64w25_idx'
)
ORDER BY relname;
