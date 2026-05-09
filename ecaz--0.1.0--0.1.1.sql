CREATE TYPE ecvector;

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

CREATE TYPE ecvector (
    INTERNALLENGTH = variable,
    INPUT = ecvector_in,
    OUTPUT = ecvector_out,
    TYPMOD_IN = ecvector_typmod_in,
    RECEIVE = ecvector_recv,
    SEND = ecvector_send,
    STORAGE = external
);

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

CREATE OPERATOR CLASS ecvector_ip_ops
DEFAULT FOR TYPE ecvector USING ec_hnsw AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);
