\timing on

ALTER INDEX task28_ivf_qcmp10k_pqfastscan_idx SET (rerank_width = 1000);

SELECT
  relname,
  reloptions
FROM pg_class
WHERE relname = 'task28_ivf_qcmp10k_pqfastscan_idx';
