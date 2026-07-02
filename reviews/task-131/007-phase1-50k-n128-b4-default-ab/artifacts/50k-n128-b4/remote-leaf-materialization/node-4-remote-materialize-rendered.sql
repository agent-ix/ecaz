-- Phase 13e.1 — materialize one remote node's coordinator-owned leaf base
-- assignment export into the remote SPIRE index. The assignment TSV is read
-- from the local psql client machine and joined to the remote corpus table by
-- stable row id so remote heap CTIDs, not coordinator CTIDs, are stored in the
-- remote leaf objects.

\set ON_ERROR_STOP on
\if :{?consistency_mode}
\else
\set consistency_mode strict
\endif

CREATE TEMP TABLE ec_spire_leaf_base_assignment_import (
  active_epoch bigint NOT NULL,
  leaf_pid bigint NOT NULL,
  parent_pid bigint NOT NULL,
  object_version bigint NOT NULL,
  row_index bigint NOT NULL,
  assignment_flags int NOT NULL,
  vec_id_hex text NOT NULL,
  row_locator_hex text NOT NULL,
  coordinator_heap_block bigint NOT NULL,
  coordinator_heap_offset int NOT NULL,
  coordinator_heap_ctid text NOT NULL,
  row_id bigint NOT NULL,
  payload_format int NOT NULL,
  gamma real NOT NULL,
  encoded_payload_hex text NOT NULL
);

\copy ec_spire_leaf_base_assignment_import FROM 'reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n128-b4/distributed-correctness/node-4/coordinator-base-assignments.tsv' WITH (FORMAT text, DELIMITER E'\t')

CREATE TEMP TABLE ec_spire_leaf_base_materialization_input AS
SELECT src.leaf_pid,
       src.parent_pid,
       src.object_version,
       src.row_index,
       src.assignment_flags,
       src.vec_id_hex,
       split_part(trim(both '()' from heap_row.ctid::text), ',', 1)::bigint AS remote_heap_block,
       split_part(trim(both '()' from heap_row.ctid::text), ',', 2)::int AS remote_heap_offset,
       src.payload_format,
       src.gamma,
       src.encoded_payload_hex
  FROM ec_spire_leaf_base_assignment_import AS src
  JOIN :remote_table AS heap_row
    ON heap_row.id = src.row_id
 ORDER BY src.leaf_pid, src.row_index;

DO $$
DECLARE
  imported_rows bigint;
  matched_rows bigint;
  min_active_epoch bigint;
  max_active_epoch bigint;
BEGIN
  SELECT count(*) INTO imported_rows FROM ec_spire_leaf_base_assignment_import;
  SELECT count(*) INTO matched_rows FROM ec_spire_leaf_base_materialization_input;
  IF imported_rows <> matched_rows THEN
    RAISE EXCEPTION
      'remote heap materialization matched % rows for % imported coordinator assignments',
      matched_rows,
      imported_rows;
  END IF;
  SELECT min(active_epoch), max(active_epoch)
    INTO min_active_epoch, max_active_epoch
    FROM ec_spire_leaf_base_assignment_import;
  IF min_active_epoch IS NULL OR min_active_epoch <> max_active_epoch THEN
    RAISE EXCEPTION
      'remote heap materialization requires exactly one exported coordinator active epoch, got min % max %',
      min_active_epoch,
      max_active_epoch;
  END IF;
END $$;

SELECT materialized.*
  FROM ec_spire_materialize_static_remote_leaf_assignments_with_mode(
       :'remote_index'::regclass::oid,
       (SELECT min(active_epoch) FROM ec_spire_leaf_base_assignment_import),
       (SELECT array_agg(leaf_pid ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(parent_pid ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(object_version ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(row_index ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(assignment_flags ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(vec_id_hex ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(remote_heap_block ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(remote_heap_offset ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(payload_format ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(gamma ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(encoded_payload_hex ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       :'consistency_mode'
  ) AS materialized;
