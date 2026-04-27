\pset pager off
\timing on

SELECT version() AS postgres_version;

SELECT
  (SELECT count(*) FROM task28_ivf_anchor10k1536_corpus) AS corpus_rows,
  (SELECT count(*) FROM task28_ivf_anchor10k1536_queries) AS query_rows,
  cardinality((SELECT source FROM task28_ivf_anchor10k1536_corpus ORDER BY id LIMIT 1)) AS source_dimensions,
  pg_relation_size('task28_ivf_anchor10k1536_n32_idx'::regclass) AS index_bytes;

SET enable_indexscan = off;
SET enable_bitmapscan = off;
SET enable_seqscan = on;

CREATE TEMP TABLE task28_ivf_align_exact_top200 AS
SELECT q.id AS query_id, exact.id AS corpus_id, exact.sql_score, exact.exact_rank
FROM task28_ivf_anchor10k1536_queries q
CROSS JOIN LATERAL (
  SELECT
    c.id,
    c.embedding <#> q.source AS sql_score,
    row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS exact_rank
  FROM task28_ivf_anchor10k1536_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 200
) exact;

RESET enable_indexscan;
RESET enable_bitmapscan;
RESET enable_seqscan;

SET enable_seqscan = off;
SET ec_ivf.nprobe = 32;

CREATE TEMP TABLE task28_ivf_align_ivf_top200 AS
SELECT q.id AS query_id, ann.id AS corpus_id, ann.sql_score, ann.ivf_rank
FROM task28_ivf_anchor10k1536_queries q
CROSS JOIN LATERAL (
  SELECT
    ranked.id,
    ranked.sql_score,
    row_number() OVER () AS ivf_rank
  FROM (
    SELECT c.id, c.embedding <#> q.source AS sql_score
    FROM task28_ivf_anchor10k1536_corpus c
    ORDER BY c.embedding <#> q.source
    LIMIT 200
  ) ranked
) ann;

RESET enable_seqscan;
RESET ec_ivf.nprobe;

SELECT
  count(*) FILTER (WHERE e.exact_rank <= 10 AND i.ivf_rank <= 10) AS top10_hits,
  count(*) FILTER (WHERE e.exact_rank <= 10) AS exact_top10_rows,
  round(
    count(*) FILTER (WHERE e.exact_rank <= 10 AND i.ivf_rank <= 10)::numeric
    / NULLIF(count(*) FILTER (WHERE e.exact_rank <= 10), 0),
    4
  ) AS recall_at_10,
  count(*) FILTER (WHERE e.exact_rank <= 10 AND i.ivf_rank IS NULL) AS exact_top10_missing_from_ivf_top200,
  max(i.ivf_rank) FILTER (WHERE e.exact_rank <= 10) AS worst_ivf_rank_for_exact_top10
FROM task28_ivf_align_exact_top200 e
LEFT JOIN task28_ivf_align_ivf_top200 i
  ON i.query_id = e.query_id
 AND i.corpus_id = e.corpus_id;

WITH exact_top10 AS (
  SELECT * FROM task28_ivf_align_exact_top200 WHERE exact_rank <= 10
),
ivf_top10 AS (
  SELECT * FROM task28_ivf_align_ivf_top200 WHERE ivf_rank <= 10
),
per_query AS (
  SELECT
    q.id AS query_id,
    count(DISTINCT i.corpus_id) FILTER (WHERE i.ivf_rank <= 10) AS hits,
    max(e.sql_score) AS exact_worst_top10_score,
    max(v.sql_score) AS ivf_worst_top10_score,
    min(i.ivf_rank) FILTER (WHERE i.ivf_rank > 10) AS first_demoted_exact_rank
  FROM task28_ivf_anchor10k1536_queries q
  LEFT JOIN exact_top10 e ON e.query_id = q.id
  LEFT JOIN task28_ivf_align_ivf_top200 i
    ON i.query_id = e.query_id
   AND i.corpus_id = e.corpus_id
  LEFT JOIN ivf_top10 v ON v.query_id = q.id
  GROUP BY q.id
)
SELECT
  query_id,
  hits,
  exact_worst_top10_score,
  ivf_worst_top10_score,
  first_demoted_exact_rank
FROM per_query
WHERE hits < 10
ORDER BY hits, query_id
LIMIT 20;

WITH exact_top10 AS (
  SELECT * FROM task28_ivf_align_exact_top200 WHERE exact_rank <= 10
),
ivf_top10 AS (
  SELECT * FROM task28_ivf_align_ivf_top200 WHERE ivf_rank <= 10
),
missing AS (
  SELECT
    e.query_id,
    e.corpus_id,
    e.exact_rank,
    i.ivf_rank,
    e.sql_score AS exact_sql_score
  FROM exact_top10 e
  LEFT JOIN task28_ivf_align_ivf_top200 i
    ON i.query_id = e.query_id
   AND i.corpus_id = e.corpus_id
  WHERE COALESCE(i.ivf_rank, 1000000) > 10
),
extra AS (
  SELECT
    v.query_id,
    v.corpus_id,
    v.ivf_rank,
    e.exact_rank,
    v.sql_score AS ivf_sql_score
  FROM ivf_top10 v
  LEFT JOIN task28_ivf_align_exact_top200 e
    ON e.query_id = v.query_id
   AND e.corpus_id = v.corpus_id
  WHERE COALESCE(e.exact_rank, 1000000) > 10
)
SELECT
  m.query_id,
  m.corpus_id AS missing_exact_corpus_id,
  m.exact_rank AS missing_exact_rank,
  m.ivf_rank AS missing_ivf_rank,
  m.exact_sql_score AS missing_exact_sql_score,
  x.corpus_id AS extra_ivf_corpus_id,
  x.ivf_rank AS extra_ivf_rank,
  x.exact_rank AS extra_exact_rank,
  x.ivf_sql_score AS extra_ivf_sql_score,
  x.ivf_sql_score - m.exact_sql_score AS sql_score_gap
FROM missing m
JOIN LATERAL (
  SELECT *
  FROM extra x
  WHERE x.query_id = m.query_id
  ORDER BY x.ivf_sql_score DESC, x.ivf_rank
  LIMIT 1
) x ON true
ORDER BY m.query_id, m.exact_rank
LIMIT 40;

WITH exact_top10 AS (
  SELECT * FROM task28_ivf_align_exact_top200 WHERE exact_rank <= 10
),
ivf_top10 AS (
  SELECT * FROM task28_ivf_align_ivf_top200 WHERE ivf_rank <= 10
),
missing AS (
  SELECT
    e.query_id,
    e.corpus_id,
    e.exact_rank,
    i.ivf_rank,
    e.sql_score AS exact_sql_score
  FROM exact_top10 e
  LEFT JOIN task28_ivf_align_ivf_top200 i
    ON i.query_id = e.query_id
   AND i.corpus_id = e.corpus_id
  WHERE COALESCE(i.ivf_rank, 1000000) > 10
),
extra AS (
  SELECT
    v.query_id,
    v.corpus_id,
    v.ivf_rank,
    e.exact_rank,
    v.sql_score AS ivf_sql_score
  FROM ivf_top10 v
  LEFT JOIN task28_ivf_align_exact_top200 e
    ON e.query_id = v.query_id
   AND e.corpus_id = v.corpus_id
  WHERE COALESCE(e.exact_rank, 1000000) > 10
),
paired AS (
  SELECT
    m.query_id,
    m.exact_rank,
    m.ivf_rank,
    m.exact_sql_score,
    x.ivf_sql_score,
    x.ivf_sql_score - m.exact_sql_score AS sql_score_gap
  FROM missing m
  JOIN LATERAL (
    SELECT *
    FROM extra x
    WHERE x.query_id = m.query_id
    ORDER BY x.ivf_sql_score DESC, x.ivf_rank
    LIMIT 1
  ) x ON true
)
SELECT
  count(*) AS demoted_exact_top10_rows,
  min(ivf_rank) AS best_demoted_ivf_rank,
  max(ivf_rank) AS worst_demoted_ivf_rank,
  avg(sql_score_gap) AS avg_sql_score_gap,
  max(sql_score_gap) AS max_sql_score_gap
FROM paired;
