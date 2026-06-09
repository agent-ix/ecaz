CREATE OR REPLACE FUNCTION "ec_task87_candidate_batch_scoring_reset"() RETURNS void
STRICT VOLATILE
LANGUAGE c
AS '$libdir/ecaz', 'ec_task87_candidate_batch_scoring_reset_wrapper';

CREATE OR REPLACE FUNCTION "ec_task87_candidate_batch_scoring_snapshot"() RETURNS TABLE (
    "surface" TEXT,
    "flushes" bigint,
    "candidates" bigint,
    "elapsed_nanos" bigint,
    "elapsed_ms" double precision,
    "lut32_flushes" bigint,
    "lut32_candidates" bigint
)
STRICT STABLE
LANGUAGE c
AS '$libdir/ecaz', 'ec_task87_candidate_batch_scoring_snapshot_wrapper';
