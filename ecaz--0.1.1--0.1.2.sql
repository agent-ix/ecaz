-- Remove the superseded SPIRE coordinator-side row materialization catalog.
--
-- 0.1.1 introduced ec_spire_remote_row_materialization for the Shape-A
-- distributed-read design: remote-origin AM scan rows needed a coordinator
-- heap mirror row before the index AM could return a visible heap TID. The
-- 0.1.2 Shape-B CustomScan pivot returns remote tuple payloads directly
-- through EcSpireDistributedScan instead, and ADR-069 coordinator-routed
-- writes track ownership in ec_spire_placement. The old catalog is therefore
-- intentionally dropped during the upgrade.

DROP TABLE IF EXISTS ec_spire_remote_row_materialization;

-- Bound the DML PK-select CustomScan planner gate. The planner only needs to
-- know whether ec_spire_placement has any row for a candidate index_oid, so a
-- narrow leading-column index avoids scanning unrelated placement rows as the
-- placement directory grows.
CREATE INDEX IF NOT EXISTS ec_spire_placement_by_index_oid
ON ec_spire_placement (index_oid);

-- Bind coordinator-routed INSERT descriptors to the coordinator heap column
-- shape observed when the descriptor is registered or refreshed, and bind
-- them to the remote heap column shape echoed by the remote index. Coordinator-
-- only or remote-only DDL now fails closed before remote dispatch until
-- operators apply matching DDL and refresh descriptors.
ALTER TABLE ec_spire_remote_node_descriptor
ADD COLUMN IF NOT EXISTS coordinator_insert_shape_fingerprint text NOT NULL DEFAULT 'unset'
    CHECK (length(coordinator_insert_shape_fingerprint) > 0);

ALTER TABLE ec_spire_remote_node_descriptor
ADD COLUMN IF NOT EXISTS remote_insert_shape_fingerprint text NOT NULL DEFAULT 'unset'
    CHECK (length(remote_insert_shape_fingerprint) > 0);

CREATE OR REPLACE FUNCTION ec_spire_coordinator_insert_shape_fingerprint(table_oid regclass)
RETURNS text
STABLE STRICT
LANGUAGE sql
AS $$
    SELECT md5(COALESCE(string_agg(
               attnum::text || ':' ||
               quote_ident(attname) || ':' ||
               atttypid::text || ':' ||
               atttypmod::text || ':' ||
               attcollation::text || ':' ||
               attnotnull::text,
               ',' ORDER BY attnum), ''))
      FROM pg_attribute
     WHERE attrelid = table_oid::oid
       AND attnum > 0
       AND NOT attisdropped
$$;

CREATE OR REPLACE FUNCTION ec_spire_coordinator_index_shape_fingerprint(index_oid regclass)
RETURNS text
STABLE STRICT
LANGUAGE sql
AS $$
    SELECT ec_spire_coordinator_insert_shape_fingerprint(indrelid::regclass)
      FROM pg_index
     WHERE indexrelid = index_oid::oid
$$;

CREATE OR REPLACE FUNCTION ec_spire_remote_index_shape_fingerprint(index_oid regclass)
RETURNS text
STABLE STRICT
LANGUAGE sql
AS $$
    SELECT ec_spire_coordinator_index_shape_fingerprint(index_oid)
$$;

UPDATE ec_spire_remote_node_descriptor d
   SET coordinator_insert_shape_fingerprint =
       ec_spire_coordinator_index_shape_fingerprint(d.coordinator_index_oid::regclass)
 WHERE d.coordinator_insert_shape_fingerprint = 'unset'
   AND EXISTS (
       SELECT 1
         FROM pg_index i
        WHERE i.indexrelid = d.coordinator_index_oid
   );

-- Task 182 binds the production DistANN head policy and its immutable training
-- input to each physical generation. Existing rows represent the legacy
-- current-sample graph policy and are backfilled accordingly.
ALTER TABLE ec_distann_generation_head_state
ADD COLUMN IF NOT EXISTS head_policy smallint NOT NULL DEFAULT 0;

ALTER TABLE ec_distann_generation_head_state
ADD COLUMN IF NOT EXISTS training_query_count integer NOT NULL DEFAULT 0;

ALTER TABLE ec_distann_generation_head_state
ADD COLUMN IF NOT EXISTS training_query_digest bytea NOT NULL
    DEFAULT decode(repeat('00', 32), 'hex');

ALTER TABLE ec_distann_generation_head_state
ADD CONSTRAINT ec_distann_generation_head_state_policy_check
CHECK (head_policy IN (0, 1));

ALTER TABLE ec_distann_generation_head_state
ADD CONSTRAINT ec_distann_generation_head_state_training_count_check
CHECK (training_query_count >= 0);

ALTER TABLE ec_distann_generation_head_state
ADD CONSTRAINT ec_distann_generation_head_state_training_digest_check
CHECK (octet_length(training_query_digest) = 32);

ALTER TABLE ec_distann_generation_head_state
ADD CONSTRAINT ec_distann_generation_head_state_training_policy_check
CHECK (
    (head_policy = 0 AND training_query_count = 0
        AND training_query_digest = decode(repeat('00', 32), 'hex'))
    OR
    (head_policy = 1 AND training_query_count = 200
        AND training_query_digest <> decode(repeat('00', 32), 'hex'))
);

ALTER TABLE ec_distann_generation_head_state
ALTER COLUMN head_policy DROP DEFAULT;

ALTER TABLE ec_distann_generation_head_state
ALTER COLUMN training_query_count DROP DEFAULT;

ALTER TABLE ec_distann_generation_head_state
ALTER COLUMN training_query_digest DROP DEFAULT;

CREATE FUNCTION ec_distann_build_epoch_with_training(
    index_regclass regclass,
    epoch bigint,
    build_id uuid,
    training_relation regclass
) RETURNS bytea
STRICT VOLATILE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_distann_build_epoch_with_training_wrapper';

CREATE FUNCTION ec_distann_active_head_policy(index_regclass regclass)
RETURNS TABLE (
    head_policy text,
    scoring_mode text,
    training_query_count integer,
    training_query_digest bytea,
    head_index_cap integer,
    returned_seed_count integer,
    sample_count integer,
    head_sample_digest bytea
)
STRICT STABLE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_distann_active_head_policy_wrapper';

ALTER FUNCTION ec_distann_build_epoch_with_training(regclass, bigint, uuid, regclass)
    SECURITY DEFINER;
ALTER FUNCTION ec_distann_build_epoch_with_training(regclass, bigint, uuid, regclass)
    SET search_path TO pg_catalog, @extschema@, pg_temp;
REVOKE ALL ON FUNCTION ec_distann_build_epoch_with_training(regclass, bigint, uuid, regclass)
    FROM PUBLIC;

ALTER FUNCTION ec_distann_active_head_policy(regclass) SECURITY DEFINER;
ALTER FUNCTION ec_distann_active_head_policy(regclass)
    SET search_path TO pg_catalog, @extschema@, pg_temp;
REVOKE ALL ON FUNCTION ec_distann_active_head_policy(regclass) FROM PUBLIC;
