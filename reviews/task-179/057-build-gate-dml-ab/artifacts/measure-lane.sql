SET synchronous_commit = off;
SET jit = off;
SET client_min_messages = warning;

CREATE TEMP TABLE task179_dml_gate_run_config (round_id integer NOT NULL);
INSERT INTO task179_dml_gate_run_config VALUES (:'round'::integer);

DO $measure$
DECLARE
    lane_name text;
    measured_round integer;
    trial_id integer;
    statement_id integer;
    statement_count constant integer := 25000;
    started_at timestamptz;
    elapsed_us numeric;
BEGIN
    SELECT CASE WHEN count(*) = 1 THEN 'installed' ELSE 'control' END
      INTO lane_name
      FROM pg_catalog.pg_extension
     WHERE extname = 'ecaz';
    SELECT round_id INTO measured_round FROM task179_dml_gate_run_config;

    FOR trial_id IN 1..5 LOOP
        TRUNCATE task179_dml_gate_probe;
        started_at := clock_timestamp();
        FOR statement_id IN 1..statement_count LOOP
            INSERT INTO task179_dml_gate_probe (id, payload)
            VALUES (
                measured_round::bigint * 1000000000
                    + trial_id::bigint * 1000000
                    + statement_id,
                statement_id
            );
        END LOOP;
        elapsed_us := extract(epoch FROM clock_timestamp() - started_at) * 1000000;
        IF elapsed_us <= 0 THEN
            RAISE EXCEPTION
                'nonpositive elapsed time from host wall clock: % microseconds',
                elapsed_us;
        END IF;

        -- Trial 1 warms the PL/pgSQL plan and relation caches. Persist only the
        -- four subsequent samples from each ABBA round.
        IF trial_id > 1 THEN
            INSERT INTO task179_dml_gate_results
                (lane, round_id, trial, statements, total_us, us_per_statement)
            VALUES (
                lane_name,
                measured_round,
                trial_id - 1,
                statement_count,
                elapsed_us,
                elapsed_us / statement_count
            );
        END IF;
    END LOOP;
END
$measure$;

SELECT format(
    '[suite-result] dml_gate_latency lane=%s round=%s trial=%s statements=%s total_ms=%s us_per_statement=%s',
    lane,
    round_id,
    trial,
    statements,
    round(total_us / 1000, 3),
    round(us_per_statement, 3)
)
FROM task179_dml_gate_results
WHERE round_id = :'round'::integer
ORDER BY trial;

SELECT format(
    '[suite-result] dml_gate_round lane=%s round=%s samples=%s median_us=%s p95_us=%s',
    min(lane),
    min(round_id),
    count(*),
    round(percentile_cont(0.5) WITHIN GROUP (ORDER BY us_per_statement)::numeric, 3),
    round(percentile_cont(0.95) WITHIN GROUP (ORDER BY us_per_statement)::numeric, 3)
)
FROM task179_dml_gate_results
WHERE round_id = :'round'::integer;
