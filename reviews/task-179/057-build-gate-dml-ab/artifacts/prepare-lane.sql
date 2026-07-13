SET synchronous_commit = off;
SET jit = off;

DROP TABLE IF EXISTS task179_dml_gate_probe;
DROP TABLE IF EXISTS task179_dml_gate_results;
CREATE UNLOGGED TABLE task179_dml_gate_probe (
    id bigint NOT NULL,
    payload bigint NOT NULL
);
CREATE UNLOGGED TABLE task179_dml_gate_results (
    lane text NOT NULL,
    round_id integer NOT NULL,
    trial integer NOT NULL,
    statements integer NOT NULL,
    total_us numeric NOT NULL CHECK (total_us > 0),
    us_per_statement numeric NOT NULL CHECK (us_per_statement > 0)
);

SELECT format(
    '[suite-result] dml_gate_setup lane=%s extension_installed=%s preload_ok=%s',
    CASE WHEN count(*) = 1 THEN 'installed' ELSE 'control' END,
    count(*),
    CASE
        WHEN current_setting('shared_preload_libraries') ~ '(^|,)\s*ecaz\s*(,|$)'
        THEN 1 ELSE 0
    END
)
FROM pg_catalog.pg_extension
WHERE extname = 'ecaz';
