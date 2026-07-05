\timing on

ALTER INDEX task28_ivf_pqg25k_g8_idx SET (rerank_width = 500);

SELECT relname, reloptions
FROM pg_class
WHERE relname = 'task28_ivf_pqg25k_g8_idx';
