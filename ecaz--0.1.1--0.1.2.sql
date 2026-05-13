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
