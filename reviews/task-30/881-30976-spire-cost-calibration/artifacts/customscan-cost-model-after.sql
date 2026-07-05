WITH constants AS (
    SELECT
        current_setting('cpu_operator_cost')::float8 AS cpu_operator_cost,
        current_setting('cpu_tuple_cost')::float8 AS cpu_tuple_cost,
        64.0::float8 AS routing_score_bound,
        1024.0::float8 AS remote_dispatch_cpu_units,
        0.5::float8 AS merge_cpu_units,
        0.001::float8 AS tuple_byte_cpu_units
),
eligibility AS (
    SELECT *
    FROM ec_spire_custom_scan_index_eligibility('phase12_tuple_measure_coord_idx'::regclass)
),
cases AS (
    SELECT *
    FROM (VALUES
        ('id_only'::text, 8.0::float8, 10.0::float8),
        ('id_only', 8.0, 50.0),
        ('id_only', 8.0, 100.0),
        ('title_body', 175.0, 10.0),
        ('title_body', 175.0, 50.0),
        ('title_body', 175.0, 100.0)
    ) AS t(projection, target_width, output_rows)
)
SELECT
    c.projection,
    c.output_rows::integer AS output_rows,
    e.remote_available_node_count AS remote_fanout,
    e.remote_available_placement_count AS remote_placements,
    round(
        (
            LEAST(e.remote_available_placement_count::float8, constants.routing_score_bound)
            * constants.cpu_operator_cost
        )::numeric,
        6
    ) AS routing_traversal_cost,
    round(
        (
            GREATEST(e.remote_available_node_count, 1)::float8
            * constants.remote_dispatch_cpu_units
            * constants.cpu_operator_cost
        )::numeric,
        6
    ) AS remote_dispatch_cost,
    round(
        (
            (c.output_rows * GREATEST(e.remote_available_node_count, 1)::float8)
            * (constants.cpu_tuple_cost + constants.cpu_operator_cost)
        )::numeric,
        6
    ) AS heap_rerank_cost,
    round(
        (
            c.output_rows
            * GREATEST(log(2, GREATEST(e.remote_available_node_count, 1)::numeric), 1)::float8
            * constants.merge_cpu_units
            * constants.cpu_operator_cost
        )::numeric,
        6
    ) AS merge_cost,
    round((c.output_rows * constants.cpu_tuple_cost)::numeric, 6) AS tuple_delivery_cost,
    round(
        (
            c.output_rows
            * c.target_width
            * constants.tuple_byte_cpu_units
            * constants.cpu_operator_cost
        )::numeric,
        6
    ) AS tuple_width_cost,
    round(
        (
            LEAST(e.remote_available_placement_count::float8, constants.routing_score_bound)
            * constants.cpu_operator_cost
            + GREATEST(e.remote_available_node_count, 1)::float8
              * constants.remote_dispatch_cpu_units
              * constants.cpu_operator_cost
        )::numeric,
        6
    ) AS modeled_startup_cost,
    round(
        (
            LEAST(e.remote_available_placement_count::float8, constants.routing_score_bound)
            * constants.cpu_operator_cost
            + GREATEST(e.remote_available_node_count, 1)::float8
              * constants.remote_dispatch_cpu_units
              * constants.cpu_operator_cost
            + (c.output_rows * GREATEST(e.remote_available_node_count, 1)::float8)
              * (constants.cpu_tuple_cost + constants.cpu_operator_cost)
            + c.output_rows
              * GREATEST(log(2, GREATEST(e.remote_available_node_count, 1)::numeric), 1)::float8
              * constants.merge_cpu_units
              * constants.cpu_operator_cost
            + c.output_rows * constants.cpu_tuple_cost
            + c.output_rows
              * c.target_width
              * constants.tuple_byte_cpu_units
              * constants.cpu_operator_cost
        )::numeric,
        6
    ) AS modeled_total_cost
FROM cases c
CROSS JOIN eligibility e
CROSS JOIN constants
ORDER BY c.projection, c.output_rows;
