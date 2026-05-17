\timing on

ALTER INDEX task28_ivf_pqg10k_g8_idx SET (rerank_width = 250);

SELECT relname, reloptions
FROM pg_class
WHERE relname = 'task28_ivf_pqg10k_g8_idx';
