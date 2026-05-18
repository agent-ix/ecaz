\timing on
\d task28_ivf_anchor10k1536_exact100_top10
\d task28_ivf_anchor25k_exact100_top10
SELECT count(*) AS q10k20 FROM task28_ivf_anchor10k1536_queries;
SELECT count(*) AS q10k100 FROM task28_ivf_anchor10k1536_queries100;
SELECT count(DISTINCT query_id) AS exact10k_queries FROM task28_ivf_anchor10k1536_exact100_top10;
SELECT count(DISTINCT query_id) AS exact25k_queries FROM task28_ivf_anchor25k_exact100_top10;
