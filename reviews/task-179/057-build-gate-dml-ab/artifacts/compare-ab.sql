\connect task179_gate_control
SELECT
    count(*) AS control_samples,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY us_per_statement) AS control_median_us,
    percentile_cont(0.95) WITHIN GROUP (ORDER BY us_per_statement) AS control_p95_us
FROM task179_dml_gate_results
\gset

\connect task179_gate_installed
SELECT
    count(*) AS installed_samples,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY us_per_statement) AS installed_median_us,
    percentile_cont(0.95) WITHIN GROUP (ORDER BY us_per_statement) AS installed_p95_us
FROM task179_dml_gate_results
\gset

SELECT format(
    '[suite-result] dml_gate_ab control_samples=%s installed_samples=%s control_median_us=%s installed_median_us=%s delta_us=%s ratio=%s overhead_pct=%s control_p95_us=%s installed_p95_us=%s p95_ratio=%s',
    :'control_samples',
    :'installed_samples',
    round(:'control_median_us'::numeric, 3),
    round(:'installed_median_us'::numeric, 3),
    round(:'installed_median_us'::numeric - :'control_median_us'::numeric, 3),
    round(:'installed_median_us'::numeric / :'control_median_us'::numeric, 3),
    round(
        (:'installed_median_us'::numeric / :'control_median_us'::numeric - 1) * 100,
        1
    ),
    round(:'control_p95_us'::numeric, 3),
    round(:'installed_p95_us'::numeric, 3),
    round(:'installed_p95_us'::numeric / :'control_p95_us'::numeric, 3)
);
