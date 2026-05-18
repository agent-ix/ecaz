\timing on
ALTER INDEX task28_ivf_postopt10k_n64w25_idx SET (rerank_width = 10);
ALTER INDEX task28_ivf_postopt25k_n64w25_idx SET (rerank_width = 10);
SELECT relname, reloptions
FROM pg_class
WHERE relname IN (
  'task28_ivf_postopt10k_n64w25_idx',
  'task28_ivf_postopt25k_n64w25_idx'
)
ORDER BY relname;
