CREATE FUNCTION ec_block_kernel_scoring_reset() RETURNS void
STRICT VOLATILE
LANGUAGE c
AS '$libdir/ecaz', 'ec_block_kernel_scoring_reset_wrapper';

CREATE FUNCTION ec_block_kernel_scoring_snapshot() RETURNS TABLE (
    surface text,
    quant_kind text,
    isa text,
    flushes bigint,
    candidates bigint,
    elapsed_nanos bigint,
    elapsed_ms double precision,
    kernel_flushes bigint,
    kernel_candidates bigint,
    kernel_elapsed_nanos bigint,
    kernel_elapsed_ms double precision,
    scalar_flushes bigint,
    scalar_candidates bigint,
    scalar_elapsed_nanos bigint,
    scalar_elapsed_ms double precision
)
STRICT STABLE
LANGUAGE c
AS '$libdir/ecaz', 'ec_block_kernel_scoring_snapshot_wrapper';
