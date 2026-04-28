\timing on

ALTER INDEX task28_ivf_pqg100k_g8_n128_idx SET (rerank_width = 750);

SELECT relname, reloptions
FROM pg_class
WHERE relname = 'task28_ivf_pqg100k_g8_n128_idx';
