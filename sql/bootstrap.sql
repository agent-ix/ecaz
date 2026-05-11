CREATE TYPE tqvector;
CREATE TYPE ecvector;

CREATE FUNCTION tqvector_in(cstring)
RETURNS tqvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_in_wrapper';

CREATE FUNCTION tqvector_out(tqvector)
RETURNS cstring
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_out_wrapper';

CREATE FUNCTION tqvector_recv(internal)
RETURNS tqvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_recv_wrapper';

CREATE FUNCTION tqvector_send(tqvector)
RETURNS bytea
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_send_wrapper';

CREATE FUNCTION ecvector_in(cstring, oid, integer)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_in_wrapper';

CREATE FUNCTION ecvector_out(ecvector)
RETURNS cstring
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_out_wrapper';

CREATE FUNCTION ecvector_typmod_in(cstring[])
RETURNS integer
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_typmod_in';

CREATE FUNCTION ecvector_recv(internal, oid, integer)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_recv_wrapper';

CREATE FUNCTION ecvector_send(ecvector)
RETURNS bytea
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_send_wrapper';

CREATE TYPE tqvector (
    INTERNALLENGTH = variable,
    INPUT = tqvector_in,
    OUTPUT = tqvector_out,
    RECEIVE = tqvector_recv,
    SEND = tqvector_send,
    STORAGE = external
);

CREATE TYPE ecvector (
    INTERNALLENGTH = variable,
    INPUT = ecvector_in,
    OUTPUT = ecvector_out,
    TYPMOD_IN = ecvector_typmod_in,
    RECEIVE = ecvector_recv,
    SEND = ecvector_send,
    STORAGE = external
);

CREATE FUNCTION encode_to_tqvector(real[], integer, bigint)
RETURNS tqvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'encode_to_tqvector_wrapper';

CREATE FUNCTION encode_to_ecvector(real[], integer, bigint)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'encode_to_ecvector_wrapper';

CREATE FUNCTION ecvector(ecvector, integer, boolean)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_coerce_wrapper';

CREATE FUNCTION ecvector_from_real_array(real[], integer, boolean)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_from_real_array_wrapper';

CREATE FUNCTION ecvector_to_real_array(ecvector, integer, boolean)
RETURNS real[]
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_to_real_array_wrapper';

CREATE FUNCTION ecvector_from_bytea(bytea, integer, boolean)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_from_bytea_wrapper';

CREATE FUNCTION ecvector_to_bytea(ecvector, integer, boolean)
RETURNS bytea
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_to_bytea_wrapper';

CREATE CAST (ecvector AS ecvector)
WITH FUNCTION ecvector(ecvector, integer, boolean)
AS IMPLICIT;

CREATE CAST (real[] AS ecvector)
WITH FUNCTION ecvector_from_real_array(real[], integer, boolean)
AS ASSIGNMENT;

CREATE CAST (ecvector AS real[])
WITH FUNCTION ecvector_to_real_array(ecvector, integer, boolean);

CREATE CAST (bytea AS ecvector)
WITH FUNCTION ecvector_from_bytea(bytea, integer, boolean);

CREATE CAST (ecvector AS bytea)
WITH FUNCTION ecvector_to_bytea(ecvector, integer, boolean);

CREATE TABLE ec_spire_remote_node_descriptor (
    coordinator_index_oid oid NOT NULL,
    node_id integer NOT NULL CHECK (node_id > 0),
    descriptor_generation bigint NOT NULL CHECK (descriptor_generation >= 0),
    conninfo_secret_name text NOT NULL CHECK (length(conninfo_secret_name) > 0),
    remote_index_identity bytea NOT NULL CHECK (octet_length(remote_index_identity) > 0),
    remote_index_regclass text NOT NULL CHECK (length(remote_index_regclass) > 0),
    descriptor_state text NOT NULL CHECK (
        descriptor_state IN ('active', 'draining', 'disabled', 'failed')
    ),
    last_seen_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    last_served_epoch bigint NOT NULL CHECK (last_served_epoch >= 0),
    min_retained_epoch bigint NOT NULL CHECK (min_retained_epoch >= 0),
    extension_version text NOT NULL CHECK (length(extension_version) > 0),
    last_error text NOT NULL DEFAULT 'none',
    PRIMARY KEY (coordinator_index_oid, node_id)
);

CREATE TABLE ec_spire_remote_epoch_manifest (
    coordinator_index_oid oid NOT NULL,
    active_epoch bigint NOT NULL CHECK (active_epoch > 0),
    manifest_scope text NOT NULL CHECK (length(manifest_scope) > 0),
    manifest_decision text NOT NULL CHECK (length(manifest_decision) > 0),
    manifest_entry_count bigint NOT NULL CHECK (manifest_entry_count >= 0),
    included_remote_node_count bigint NOT NULL CHECK (included_remote_node_count >= 0),
    remote_placement_count bigint NOT NULL CHECK (remote_placement_count >= 0),
    publish_decision text NOT NULL CHECK (length(publish_decision) > 0),
    status text NOT NULL CHECK (length(status) > 0),
    persisted_at_micros bigint NOT NULL CHECK (persisted_at_micros > 0),
    PRIMARY KEY (coordinator_index_oid, active_epoch)
);

CREATE TABLE ec_spire_remote_epoch_manifest_entry (
    coordinator_index_oid oid NOT NULL,
    active_epoch bigint NOT NULL CHECK (active_epoch > 0),
    node_id integer NOT NULL CHECK (node_id > 0),
    descriptor_state text NOT NULL CHECK (length(descriptor_state) > 0),
    placement_count bigint NOT NULL CHECK (placement_count > 0),
    required_last_served_epoch bigint NOT NULL CHECK (required_last_served_epoch >= 0),
    required_min_retained_epoch bigint NOT NULL CHECK (required_min_retained_epoch >= 0),
    last_served_epoch bigint NOT NULL CHECK (last_served_epoch >= 0),
    min_retained_epoch bigint NOT NULL CHECK (min_retained_epoch >= 0),
    epoch_window_status text NOT NULL CHECK (length(epoch_window_status) > 0),
    manifest_action text NOT NULL CHECK (manifest_action = 'include_remote_node'),
    status text NOT NULL CHECK (length(status) > 0),
    PRIMARY KEY (coordinator_index_oid, active_epoch, node_id),
    FOREIGN KEY (coordinator_index_oid, active_epoch)
        REFERENCES ec_spire_remote_epoch_manifest (coordinator_index_oid, active_epoch)
        ON DELETE CASCADE
);

CREATE TABLE ec_spire_remote_epoch_manifest_applied (
    remote_index_oid oid NOT NULL,
    active_epoch bigint NOT NULL CHECK (active_epoch > 0),
    manifest_payload_format text NOT NULL CHECK (length(manifest_payload_format) > 0),
    manifest_scope text NOT NULL CHECK (length(manifest_scope) > 0),
    manifest_decision text NOT NULL CHECK (length(manifest_decision) > 0),
    manifest_entry_count bigint NOT NULL CHECK (manifest_entry_count >= 0),
    included_remote_node_count bigint NOT NULL CHECK (included_remote_node_count >= 0),
    remote_placement_count bigint NOT NULL CHECK (remote_placement_count >= 0),
    publish_decision text NOT NULL CHECK (length(publish_decision) > 0),
    status text NOT NULL CHECK (length(status) > 0),
    applied_at_micros bigint NOT NULL CHECK (applied_at_micros > 0),
    PRIMARY KEY (remote_index_oid, active_epoch)
);

CREATE TABLE ec_spire_remote_epoch_manifest_applied_entry (
    remote_index_oid oid NOT NULL,
    active_epoch bigint NOT NULL CHECK (active_epoch > 0),
    node_id integer NOT NULL CHECK (node_id > 0),
    descriptor_state text NOT NULL CHECK (length(descriptor_state) > 0),
    placement_count bigint NOT NULL CHECK (placement_count > 0),
    required_last_served_epoch bigint NOT NULL CHECK (required_last_served_epoch >= 0),
    required_min_retained_epoch bigint NOT NULL CHECK (required_min_retained_epoch >= 0),
    last_served_epoch bigint NOT NULL CHECK (last_served_epoch >= 0),
    min_retained_epoch bigint NOT NULL CHECK (min_retained_epoch >= 0),
    epoch_window_status text NOT NULL CHECK (length(epoch_window_status) > 0),
    manifest_action text NOT NULL CHECK (manifest_action = 'include_remote_node'),
    status text NOT NULL CHECK (length(status) > 0),
    PRIMARY KEY (remote_index_oid, active_epoch, node_id),
    FOREIGN KEY (remote_index_oid, active_epoch)
        REFERENCES ec_spire_remote_epoch_manifest_applied (remote_index_oid, active_epoch)
        ON DELETE CASCADE
);

CREATE TABLE ec_spire_remote_row_materialization (
    coordinator_index_oid oid NOT NULL,
    requested_epoch bigint NOT NULL CHECK (requested_epoch > 0),
    served_epoch bigint NOT NULL CHECK (served_epoch > 0),
    origin_node_id integer NOT NULL CHECK (origin_node_id > 0),
    vec_id bytea NOT NULL CHECK (octet_length(vec_id) > 0),
    row_locator bytea NOT NULL CHECK (octet_length(row_locator) > 0),
    scan_heap_relation_oid oid NOT NULL,
    materialized_heap_block bigint NOT NULL CHECK (
        materialized_heap_block >= 0 AND materialized_heap_block <= 4294967295
    ),
    materialized_heap_offset integer NOT NULL CHECK (
        materialized_heap_offset > 0 AND materialized_heap_offset <= 65535
    ),
    status text NOT NULL CHECK (length(status) > 0),
    materialized_at_micros bigint NOT NULL CHECK (materialized_at_micros > 0),
    PRIMARY KEY (
        coordinator_index_oid,
        requested_epoch,
        served_epoch,
        origin_node_id,
        vec_id,
        row_locator
    )
);

CREATE TABLE ec_spire_placement (
    index_oid oid NOT NULL,
    pk_value bytea NOT NULL CHECK (octet_length(pk_value) > 0),
    node_id integer NOT NULL CHECK (node_id >= 0),
    centroid_id bigint NOT NULL CHECK (centroid_id >= 0),
    served_epoch bigint NOT NULL CHECK (served_epoch > 0),
    source_identity bytea NOT NULL CHECK (octet_length(source_identity) = 16),
    PRIMARY KEY (index_oid, pk_value)
);

CREATE INDEX ec_spire_placement_by_identity
ON ec_spire_placement (index_oid, source_identity);

CREATE TYPE ec_spire_placement_entry AS (
    pk_value bytea,
    node_id integer,
    centroid_id bigint,
    served_epoch bigint,
    source_identity bytea
);

CREATE FUNCTION ec_spire_register_placement_batch(
    index_oid oid,
    entries ec_spire_placement_entry[]
)
RETURNS bigint
STRICT
LANGUAGE plpgsql
AS $$
DECLARE
    input_index_oid ALIAS FOR $1;
    input_entries ALIAS FOR $2;
    inserted_count bigint;
    null_entry_ordinal bigint;
BEGIN
    SELECT entry_position
      INTO null_entry_ordinal
      FROM generate_subscripts(input_entries, 1) AS entry_position
     WHERE input_entries[entry_position]::text IS NULL
     LIMIT 1;

    IF null_entry_ordinal IS NOT NULL THEN
        RAISE EXCEPTION 'ec_spire_register_placement_batch entries[%] is NULL',
            null_entry_ordinal
            USING ERRCODE = '22004',
                  HINT = 'Pass only non-NULL ec_spire_placement_entry values.';
    END IF;

    INSERT INTO ec_spire_placement
        (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity)
    SELECT
        input_index_oid,
        entry.pk_value,
        entry.node_id,
        entry.centroid_id,
        entry.served_epoch,
        entry.source_identity
    FROM unnest(input_entries) AS entry;

    GET DIAGNOSTICS inserted_count = ROW_COUNT;
    RETURN inserted_count;
END
$$;

CREATE FUNCTION ec_spire_coordinator_insert_forward_trigger()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    target_index_oid oid;
    pk_column text;
    embedding_column text;
    source_identity_column text;
    pk_value bytea;
    embedding real[];
    source_identity bytea;
    row_payload jsonb;
    requested_columns text[];
BEGIN
    IF TG_WHEN <> 'BEFORE' OR TG_LEVEL <> 'ROW' OR TG_OP <> 'INSERT' THEN
        RAISE EXCEPTION 'ec_spire_coordinator_insert_forward_trigger must be a BEFORE INSERT row trigger'
            USING ERRCODE = '0A000';
    END IF;
    IF TG_NARGS <> 4 THEN
        RAISE EXCEPTION 'ec_spire_coordinator_insert_forward_trigger requires 4 trigger arguments'
            USING ERRCODE = '22023',
                  HINT = 'Use ec_spire_enable_coordinator_insert(table_oid, index_oid, pk_column, embedding_column, source_identity_column).';
    END IF;

    target_index_oid := TG_ARGV[0]::oid;
    pk_column := TG_ARGV[1];
    embedding_column := TG_ARGV[2];
    source_identity_column := TG_ARGV[3];

    EXECUTE format(
        'SELECT int8send(($1).%1$I::bigint)::bytea, (($1).%2$I)::real[], (($1).%3$I)::bytea, to_jsonb($1)',
        pk_column,
        embedding_column,
        source_identity_column
    )
    USING NEW
    INTO pk_value, embedding, source_identity, row_payload;

    SELECT array_agg(attname ORDER BY attnum)
      INTO requested_columns
      FROM pg_attribute
     WHERE attrelid = TG_RELID
       AND attnum > 0
       AND NOT attisdropped;

    PERFORM 1
      FROM ec_spire_prepare_coordinator_insert_tuple_payload(
           target_index_oid,
           pk_value,
           embedding,
           source_identity,
           row_payload,
           requested_columns
      );

    RETURN NULL;
END
$$;

CREATE FUNCTION ec_spire_enable_coordinator_insert(
    table_oid regclass,
    index_oid regclass,
    pk_column text,
    embedding_column text,
    source_identity_column text DEFAULT 'source_identity'
)
RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    indexed_table_oid oid;
    index_am_name name;
    pk_type oid;
    embedding_type oid;
    source_identity_type oid;
BEGIN
    SELECT i.indrelid, am.amname
      INTO indexed_table_oid, index_am_name
      FROM pg_index i
      JOIN pg_class c ON c.oid = i.indexrelid
      JOIN pg_am am ON am.oid = c.relam
     WHERE i.indexrelid = index_oid::oid;

    IF indexed_table_oid IS NULL THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert index_oid must reference an index'
            USING ERRCODE = '42809';
    END IF;
    IF indexed_table_oid <> table_oid::oid THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert index % does not belong to table %',
            index_oid::text, table_oid::text
            USING ERRCODE = '42809';
    END IF;
    IF index_am_name <> 'ec_spire' THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert requires an ec_spire index, got %',
            index_am_name
            USING ERRCODE = '42809';
    END IF;

    SELECT atttypid INTO pk_type
      FROM pg_attribute
     WHERE attrelid = table_oid::oid AND attname = pk_column
       AND attnum > 0 AND NOT attisdropped;
    SELECT atttypid INTO embedding_type
      FROM pg_attribute
     WHERE attrelid = table_oid::oid AND attname = embedding_column
       AND attnum > 0 AND NOT attisdropped;
    SELECT atttypid INTO source_identity_type
      FROM pg_attribute
     WHERE attrelid = table_oid::oid AND attname = source_identity_column
       AND attnum > 0 AND NOT attisdropped;

    IF pk_type IS NULL THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert missing pk column %', pk_column
            USING ERRCODE = '42703';
    END IF;
    IF embedding_type IS NULL THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert missing embedding column %', embedding_column
            USING ERRCODE = '42703';
    END IF;
    IF source_identity_type IS NULL THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert missing source_identity column %',
            source_identity_column
            USING ERRCODE = '42703';
    END IF;
    IF pk_type <> 'bigint'::regtype THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert v1 requires a bigint pk column, got %',
            pk_type::regtype::text
            USING ERRCODE = '42804';
    END IF;
    IF embedding_type <> 'ecvector'::regtype THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert requires an ecvector embedding column, got %',
            embedding_type::regtype::text
            USING ERRCODE = '42804';
    END IF;
    IF source_identity_type <> 'bytea'::regtype THEN
        RAISE EXCEPTION 'ec_spire_enable_coordinator_insert requires a bytea source_identity column, got %',
            source_identity_type::regtype::text
            USING ERRCODE = '42804';
    END IF;

    EXECUTE format('DROP TRIGGER IF EXISTS ec_spire_coordinator_insert_forward ON %s', table_oid);
    EXECUTE format(
        'CREATE TRIGGER ec_spire_coordinator_insert_forward BEFORE INSERT ON %s FOR EACH ROW EXECUTE FUNCTION ec_spire_coordinator_insert_forward_trigger(%L, %L, %L, %L)',
        table_oid,
        index_oid::oid::text,
        pk_column,
        embedding_column,
        source_identity_column
    );
END
$$;

CREATE FUNCTION ec_spire_remote_catalog_drop_index_cleanup_event()
RETURNS event_trigger
LANGUAGE plpgsql
AS $$
DECLARE
    dropped_object record;
BEGIN
    FOR dropped_object IN
        SELECT *
          FROM pg_event_trigger_dropped_objects()
         WHERE object_type = 'index'
    LOOP
        PERFORM 1
          FROM ec_spire_remote_catalog_index_cleanup(dropped_object.objid::oid);
    END LOOP;
END
$$;

CREATE EVENT TRIGGER ec_spire_remote_catalog_drop_index_cleanup
ON sql_drop
EXECUTE FUNCTION ec_spire_remote_catalog_drop_index_cleanup_event();

CREATE FUNCTION tqvector_inner_product(tqvector, tqvector)
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_inner_product_wrapper';

CREATE FUNCTION tqvector_negative_inner_product(tqvector, tqvector)
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_negative_inner_product_wrapper';

CREATE FUNCTION tqvector_query_inner_product(tqvector, real[])
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_query_inner_product_wrapper';

CREATE FUNCTION tqvector_negative_query_inner_product(tqvector, real[])
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_negative_query_inner_product_wrapper';

CREATE FUNCTION ecvector_inner_product(ecvector, ecvector)
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_inner_product_wrapper';

CREATE FUNCTION ecvector_negative_inner_product(ecvector, ecvector)
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_negative_inner_product_wrapper';

CREATE FUNCTION ecvector_query_inner_product(ecvector, real[])
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_query_inner_product_wrapper';

CREATE FUNCTION ecvector_negative_query_inner_product(ecvector, real[])
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_negative_query_inner_product_wrapper';

CREATE FUNCTION ec_hnsw_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_hnsw_handler';

CREATE FUNCTION ec_ivf_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_ivf_handler';

CREATE FUNCTION ec_spire_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_spire_handler';

CREATE FUNCTION ec_diskann_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_diskann_handler';

CREATE OPERATOR <#> (
    PROCEDURE = tqvector_negative_inner_product,
    LEFTARG = tqvector,
    RIGHTARG = tqvector,
    COMMUTATOR = <#>
);

CREATE OPERATOR <#> (
    PROCEDURE = tqvector_negative_query_inner_product,
    LEFTARG = tqvector,
    RIGHTARG = real[]
);

CREATE OPERATOR <#> (
    PROCEDURE = ecvector_negative_inner_product,
    LEFTARG = ecvector,
    RIGHTARG = ecvector,
    COMMUTATOR = <#>
);

CREATE OPERATOR <#> (
    PROCEDURE = ecvector_negative_query_inner_product,
    LEFTARG = ecvector,
    RIGHTARG = real[]
);

CREATE ACCESS METHOD ec_hnsw TYPE INDEX HANDLER ec_hnsw_handler;
CREATE ACCESS METHOD ec_diskann TYPE INDEX HANDLER ec_diskann_handler;

CREATE ACCESS METHOD ec_ivf TYPE INDEX HANDLER ec_ivf_handler;
CREATE ACCESS METHOD ec_spire TYPE INDEX HANDLER ec_spire_handler;

CREATE OPERATOR CLASS tqvector_ip_ops
DEFAULT FOR TYPE tqvector USING ec_hnsw AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);

CREATE OPERATOR CLASS ecvector_ip_ops
DEFAULT FOR TYPE ecvector USING ec_hnsw AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);

CREATE OPERATOR CLASS tqvector_ip_ops
DEFAULT FOR TYPE tqvector USING ec_ivf AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);

CREATE OPERATOR CLASS ecvector_ip_ops
DEFAULT FOR TYPE ecvector USING ec_ivf AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);

CREATE OPERATOR CLASS tqvector_spire_ip_ops
DEFAULT FOR TYPE tqvector USING ec_spire AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);

CREATE OPERATOR CLASS ecvector_spire_ip_ops
DEFAULT FOR TYPE ecvector USING ec_spire AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);

CREATE OPERATOR CLASS tqvector_diskann_ip_ops
DEFAULT FOR TYPE tqvector USING ec_diskann AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);

CREATE OPERATOR CLASS ecvector_diskann_ip_ops
DEFAULT FOR TYPE ecvector USING ec_diskann AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);
