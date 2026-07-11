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
    coordinator_insert_shape_fingerprint text NOT NULL DEFAULT 'unset'
        CHECK (length(coordinator_insert_shape_fingerprint) > 0),
    remote_insert_shape_fingerprint text NOT NULL DEFAULT 'unset'
        CHECK (length(remote_insert_shape_fingerprint) > 0),
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

CREATE TABLE ec_spire_remote_prepared_xact_intent (
    index_oid oid NOT NULL,
    node_id integer NOT NULL CHECK (node_id > 0),
    served_epoch bigint NOT NULL CHECK (served_epoch >= 0),
    xid bigint NOT NULL CHECK (xid >= 0),
    gid text NOT NULL CHECK (
        length(gid) > 0 AND gid LIKE 'ec_spire_insert_%'
    ),
    intent_state text NOT NULL CHECK (
        intent_state IN (
            'prepare_requested',
            'prepare_acked',
            'commit_local',
            'rollback_local'
        )
    ),
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (gid)
);

CREATE INDEX ec_spire_remote_prepared_xact_intent_by_node
    ON ec_spire_remote_prepared_xact_intent (node_id, intent_state);

CREATE INDEX ec_spire_remote_prepared_xact_intent_by_index
    ON ec_spire_remote_prepared_xact_intent (index_oid, node_id, served_epoch);

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

CREATE INDEX ec_spire_placement_by_index_oid
ON ec_spire_placement (index_oid);

-- Task 179 physical-generation control catalogs. Every row carries both the
-- local relation OID and the never-reused logical UUID from v5 control
-- metadata; production lookups must match both so OID reuse cannot select
-- stale state.
CREATE TABLE ec_distann_participant_identity (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    endpoint_identity text NOT NULL CHECK (
        endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._/-]{0,254}$'
    ),
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (index_oid, logical_index_uuid)
);

CREATE TABLE ec_distann_registry_state (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    revision bigint NOT NULL DEFAULT 0 CHECK (revision >= 0),
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (index_oid, logical_index_uuid)
);

CREATE TABLE ec_distann_node_descriptor (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    roster_ordinal integer NOT NULL CHECK (roster_ordinal >= 0),
    node_id integer NOT NULL CHECK (node_id > 0),
    endpoint_identity text NOT NULL CHECK (
        endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._/-]{0,254}$'
    ),
    conninfo_secret_name text NOT NULL CHECK (
        conninfo_secret_name ~ '^[A-Z][A-Z0-9_]{0,127}$'
    ),
    remote_index_regclass text NOT NULL CHECK (
        remote_index_regclass ~ '^[a-z_][a-z0-9_]{0,62}\.[a-z_][a-z0-9_]{0,62}$'
    ),
    participant_logical_index_uuid uuid NOT NULL,
    compatibility_digest bytea NOT NULL CHECK (octet_length(compatibility_digest) = 32),
    is_local boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (index_oid, logical_index_uuid, roster_ordinal),
    UNIQUE (index_oid, logical_index_uuid, node_id),
    UNIQUE (index_oid, logical_index_uuid, endpoint_identity),
    UNIQUE (index_oid, logical_index_uuid, participant_logical_index_uuid)
);

CREATE UNIQUE INDEX ec_distann_node_descriptor_one_local
    ON ec_distann_node_descriptor (index_oid, logical_index_uuid)
    WHERE is_local;

CREATE TABLE ec_distann_generation (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    epoch bigint NOT NULL CHECK (epoch > 0),
    owner_ordinal integer NOT NULL CHECK (owner_ordinal >= 0),
    node_id integer NOT NULL CHECK (node_id > 0),
    state text NOT NULL CHECK (
        state IN ('Building', 'Ready', 'Published', 'Retired')
    ),
    build_spec_digest bytea NOT NULL CHECK (octet_length(build_spec_digest) = 32),
    roster_digest bytea NOT NULL CHECK (octet_length(roster_digest) = 32),
    generation_descriptor bytea NOT NULL CHECK (octet_length(generation_descriptor) > 0),
    generation_descriptor_digest bytea NOT NULL CHECK (
        octet_length(generation_descriptor_digest) = 32
    ),
    expected_owner_count bigint NOT NULL CHECK (expected_owner_count >= 0),
    expected_owner_digest bytea NOT NULL CHECK (octet_length(expected_owner_digest) = 32),
    row_tier_relid oid NOT NULL CHECK (row_tier_relid <> '0'::oid),
    graph_store_relid oid NOT NULL CHECK (graph_store_relid <> '0'::oid),
    directory_relid oid NOT NULL CHECK (directory_relid <> '0'::oid),
    next_batch_seq bigint NOT NULL DEFAULT 0 CHECK (next_batch_seq >= 0),
    cumulative_record_count bigint NOT NULL DEFAULT 0 CHECK (cumulative_record_count >= 0),
    cumulative_owner_digest bytea NOT NULL CHECK (octet_length(cumulative_owner_digest) = 32),
    last_vec_id_le bytea CHECK (
        last_vec_id_le IS NULL OR octet_length(last_vec_id_le) = 8
    ),
    owner_stream_sha256_state bytea NOT NULL CHECK (
        octet_length(owner_stream_sha256_state) = 107
        AND get_byte(owner_stream_sha256_state, 0) = 1
        AND get_byte(owner_stream_sha256_state, 1) = 0
        AND get_byte(owner_stream_sha256_state, 2) = 1
    ),
    ready_receipt bytea CHECK (
        ready_receipt IS NULL OR (
            octet_length(ready_receipt) = 303
            AND get_byte(ready_receipt, 0) = 1
            AND get_byte(ready_receipt, 1) = 0
        )
    ),
    epoch_fingerprint bytea CHECK (
        epoch_fingerprint IS NULL OR octet_length(epoch_fingerprint) = 34
    ),
    manifest_digest bytea CHECK (
        manifest_digest IS NULL OR octet_length(manifest_digest) = 32
    ),
    epoch_manifest bytea,
    published_at timestamptz,
    successor_activation bytea,
    successor_activation_digest bytea CHECK (
        successor_activation_digest IS NULL
        OR octet_length(successor_activation_digest) = 32
    ),
    retired_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    CHECK ((cumulative_record_count = 0) = (last_vec_id_le IS NULL)),
    CHECK (state = 'Building' OR next_batch_seq > 0),
    CHECK ((state = 'Building') = (ready_receipt IS NULL)),
    CHECK (
        (state IN ('Published', 'Retired')) =
        (epoch_fingerprint IS NOT NULL AND manifest_digest IS NOT NULL
         AND epoch_manifest IS NOT NULL AND published_at IS NOT NULL)
    ),
    CHECK (
        (state = 'Retired') =
        (successor_activation IS NOT NULL
         AND successor_activation_digest IS NOT NULL AND retired_at IS NOT NULL)
    ),
    PRIMARY KEY (index_oid, logical_index_uuid, build_id),
    UNIQUE (row_tier_relid),
    UNIQUE (graph_store_relid),
    UNIQUE (directory_relid)
);

CREATE UNIQUE INDEX ec_distann_generation_fingerprint_unique
    ON ec_distann_generation (index_oid, logical_index_uuid, epoch_fingerprint)
    WHERE epoch_fingerprint IS NOT NULL;

CREATE TABLE ec_distann_generation_batch (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    batch_seq bigint NOT NULL CHECK (batch_seq >= 0),
    batch_digest bytea NOT NULL CHECK (octet_length(batch_digest) = 32),
    encoded_bytes bigint NOT NULL CHECK (encoded_bytes BETWEEN 141 AND 8388608),
    accepted_record_count bigint NOT NULL CHECK (accepted_record_count >= 0),
    cumulative_record_count bigint NOT NULL CHECK (
        cumulative_record_count >= accepted_record_count
    ),
    cumulative_owner_digest bytea NOT NULL CHECK (octet_length(cumulative_owner_digest) = 32),
    acknowledged_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    CHECK (batch_seq <> 0 OR cumulative_record_count = accepted_record_count),
    CHECK (
        accepted_record_count > 0
        OR (batch_seq = 0 AND cumulative_record_count = 0)
    ),
    PRIMARY KEY (index_oid, logical_index_uuid, build_id, batch_seq),
    FOREIGN KEY (index_oid, logical_index_uuid, build_id)
        REFERENCES ec_distann_generation (index_oid, logical_index_uuid, build_id)
        ON DELETE CASCADE
);

CREATE TABLE ec_distann_build_registration (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    source_relid oid NOT NULL CHECK (source_relid <> '0'::oid),
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    epoch bigint NOT NULL CHECK (epoch > 0),
    state text NOT NULL CHECK (
        state IN ('Registered', 'Building', 'Ready', 'Aborted', 'Decided', 'Published')
    ),
    registry_revision bigint NOT NULL CHECK (registry_revision >= 0),
    roster_snapshot bytea NOT NULL CHECK (octet_length(roster_snapshot) > 0),
    roster_digest bytea NOT NULL CHECK (octet_length(roster_digest) = 32),
    row_schema_fingerprint bytea NOT NULL CHECK (octet_length(row_schema_fingerprint) = 32),
    registration_digest bytea NOT NULL CHECK (octet_length(registration_digest) = 32),
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (index_oid, logical_index_uuid, build_id),
    UNIQUE (index_oid, logical_index_uuid, epoch),
    UNIQUE (
        index_oid, logical_index_uuid, build_id, epoch, registration_digest
    )
);

CREATE UNIQUE INDEX ec_distann_build_registration_one_gate_active
    ON ec_distann_build_registration (index_oid, logical_index_uuid)
    WHERE state IN ('Registered', 'Building', 'Ready', 'Decided');

CREATE INDEX ec_distann_build_registration_source_gate
    ON ec_distann_build_registration (source_relid, index_oid, logical_index_uuid)
    WHERE state IN ('Registered', 'Building', 'Ready', 'Decided');

CREATE TABLE ec_distann_build_participant_binding (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    roster_ordinal integer NOT NULL CHECK (roster_ordinal >= 0),
    node_id integer NOT NULL CHECK (node_id > 0),
    endpoint_identity text NOT NULL CHECK (
        endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._/-]{0,254}$'
    ),
    conninfo_secret_name text NOT NULL CHECK (
        conninfo_secret_name ~ '^[A-Z][A-Z0-9_]{0,127}$'
    ),
    remote_index_regclass text NOT NULL CHECK (
        remote_index_regclass ~ '^[a-z_][a-z0-9_]{0,62}\.[a-z_][a-z0-9_]{0,62}$'
    ),
    participant_logical_index_uuid uuid NOT NULL,
    compatibility_digest bytea NOT NULL CHECK (octet_length(compatibility_digest) = 32),
    is_local boolean NOT NULL,
    PRIMARY KEY (index_oid, logical_index_uuid, build_id, roster_ordinal),
    UNIQUE (index_oid, logical_index_uuid, build_id, node_id),
    UNIQUE (index_oid, logical_index_uuid, build_id, endpoint_identity),
    UNIQUE (index_oid, logical_index_uuid, build_id, participant_logical_index_uuid),
    UNIQUE (
        index_oid, logical_index_uuid, build_id, roster_ordinal, node_id,
        endpoint_identity, remote_index_regclass,
        participant_logical_index_uuid
    ),
    FOREIGN KEY (index_oid, logical_index_uuid, build_id)
        REFERENCES ec_distann_build_registration (index_oid, logical_index_uuid, build_id)
        ON DELETE CASCADE
);

CREATE UNIQUE INDEX ec_distann_build_participant_one_local
    ON ec_distann_build_participant_binding (index_oid, logical_index_uuid, build_id)
    WHERE is_local;

CREATE TABLE ec_distann_build_candidate (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    epoch bigint NOT NULL CHECK (epoch > 0),
    registration_digest bytea NOT NULL CHECK (octet_length(registration_digest) = 32),
    build_spec bytea NOT NULL CHECK (octet_length(build_spec) > 0),
    build_spec_digest bytea NOT NULL CHECK (octet_length(build_spec_digest) = 32),
    generation_descriptor bytea NOT NULL CHECK (octet_length(generation_descriptor) > 0),
    generation_descriptor_digest bytea NOT NULL CHECK (
        octet_length(generation_descriptor_digest) = 32
    ),
    source_snapshot bytea NOT NULL CHECK (octet_length(source_snapshot) > 0),
    source_snapshot_digest bytea NOT NULL CHECK (octet_length(source_snapshot_digest) = 32),
    ready_receipt_set bytea NOT NULL CHECK (octet_length(ready_receipt_set) > 0),
    ready_receipt_set_digest bytea NOT NULL CHECK (
        octet_length(ready_receipt_set_digest) = 32
    ),
    epoch_manifest bytea NOT NULL CHECK (octet_length(epoch_manifest) > 0),
    manifest_digest bytea NOT NULL CHECK (octet_length(manifest_digest) = 32),
    epoch_fingerprint bytea NOT NULL CHECK (octet_length(epoch_fingerprint) = 34),
    candidate_digest bytea NOT NULL CHECK (octet_length(candidate_digest) = 32),
    created_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (index_oid, logical_index_uuid, build_id),
    UNIQUE (
        index_oid, logical_index_uuid, build_id, epoch,
        registration_digest, candidate_digest, manifest_digest, epoch_fingerprint
    ),
    FOREIGN KEY (
        index_oid, logical_index_uuid, build_id, epoch, registration_digest
    ) REFERENCES ec_distann_build_registration (
        index_oid, logical_index_uuid, build_id, epoch, registration_digest
    )
        ON DELETE CASCADE
);

CREATE TABLE ec_distann_publish_decision (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    epoch bigint NOT NULL CHECK (epoch > 0),
    epoch_fingerprint bytea NOT NULL CHECK (octet_length(epoch_fingerprint) = 34),
    manifest_digest bytea NOT NULL CHECK (octet_length(manifest_digest) = 32),
    epoch_manifest bytea NOT NULL CHECK (octet_length(epoch_manifest) > 0),
    registration_digest bytea NOT NULL CHECK (octet_length(registration_digest) = 32),
    candidate_digest bytea NOT NULL CHECK (octet_length(candidate_digest) = 32),
    predecessor_build_id uuid CHECK (
        predecessor_build_id IS NULL OR (
            (get_byte(uuid_send(predecessor_build_id), 6) & 240) = 64
            AND (get_byte(uuid_send(predecessor_build_id), 8) & 192) = 128
        )
    ),
    predecessor_epoch bigint,
    predecessor_epoch_fingerprint bytea,
    predecessor_manifest_digest bytea,
    successor_activation bytea NOT NULL CHECK (octet_length(successor_activation) > 0),
    successor_activation_digest bytea NOT NULL CHECK (
        octet_length(successor_activation_digest) = 32
    ),
    decision_state text NOT NULL CHECK (
        decision_state IN ('Pending', 'Activated', 'Applied')
    ),
    committed_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    activated_at timestamptz,
    applied_at timestamptz,
    CHECK (
        (predecessor_build_id IS NULL AND predecessor_epoch IS NULL
         AND predecessor_epoch_fingerprint IS NULL
         AND predecessor_manifest_digest IS NULL)
        OR
        (predecessor_build_id IS NOT NULL AND predecessor_epoch IS NOT NULL
         AND predecessor_epoch > 0
         AND predecessor_epoch_fingerprint IS NOT NULL
         AND predecessor_manifest_digest IS NOT NULL
         AND octet_length(predecessor_epoch_fingerprint) = 34
         AND octet_length(predecessor_manifest_digest) = 32)
    ),
    CHECK ((decision_state = 'Pending') = (activated_at IS NULL)),
    CHECK ((decision_state = 'Applied') = (applied_at IS NOT NULL)),
    PRIMARY KEY (index_oid, logical_index_uuid, build_id),
    UNIQUE (index_oid, logical_index_uuid, epoch_fingerprint),
    UNIQUE (
        index_oid, logical_index_uuid, build_id, epoch,
        epoch_fingerprint, manifest_digest
    ),
    UNIQUE (
        index_oid, logical_index_uuid, build_id, predecessor_build_id,
        predecessor_epoch, predecessor_epoch_fingerprint,
        predecessor_manifest_digest, successor_activation_digest
    ),
    UNIQUE (
        index_oid, logical_index_uuid, build_id, predecessor_build_id,
        predecessor_epoch, predecessor_epoch_fingerprint,
        predecessor_manifest_digest, successor_activation_digest,
        decision_state
    ),
    FOREIGN KEY (index_oid, logical_index_uuid, build_id)
        REFERENCES ec_distann_build_registration (
            index_oid, logical_index_uuid, build_id
        )
        ON DELETE RESTRICT,
    FOREIGN KEY (
        index_oid, logical_index_uuid, build_id, epoch,
        registration_digest, candidate_digest, manifest_digest, epoch_fingerprint
    ) REFERENCES ec_distann_build_candidate (
        index_oid, logical_index_uuid, build_id, epoch,
        registration_digest, candidate_digest, manifest_digest, epoch_fingerprint
    ) ON DELETE RESTRICT,
    CONSTRAINT ec_distann_publish_decision_predecessor_fk FOREIGN KEY (
        index_oid, logical_index_uuid, predecessor_build_id,
        predecessor_epoch, predecessor_epoch_fingerprint,
        predecessor_manifest_digest
    ) REFERENCES ec_distann_publish_decision (
        index_oid, logical_index_uuid, build_id, epoch,
        epoch_fingerprint, manifest_digest
    ) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX ec_distann_publish_decision_one_recovery_active
    ON ec_distann_publish_decision (index_oid, logical_index_uuid)
    WHERE decision_state IN ('Pending', 'Activated');

CREATE INDEX ec_distann_publish_decision_predecessor_lookup
    ON ec_distann_publish_decision (
        index_oid, logical_index_uuid, predecessor_build_id,
        predecessor_epoch, predecessor_epoch_fingerprint,
        predecessor_manifest_digest
    );

CREATE TABLE ec_distann_predecessor_disposition (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    successor_build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(successor_build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(successor_build_id), 8) & 192) = 128
    ),
    predecessor_build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(predecessor_build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(predecessor_build_id), 8) & 192) = 128
    ),
    predecessor_epoch bigint NOT NULL CHECK (predecessor_epoch > 0),
    predecessor_epoch_fingerprint bytea NOT NULL CHECK (
        octet_length(predecessor_epoch_fingerprint) = 34
    ),
    predecessor_manifest_digest bytea NOT NULL CHECK (
        octet_length(predecessor_manifest_digest) = 32
    ),
    predecessor_roster_ordinal integer NOT NULL CHECK (
        predecessor_roster_ordinal >= 0
    ),
    node_id integer NOT NULL CHECK (node_id > 0),
    endpoint_identity text NOT NULL CHECK (
        endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._/-]{0,254}$'
    ),
    remote_index_regclass text NOT NULL CHECK (
        remote_index_regclass ~ '^[a-z_][a-z0-9_]{0,62}\.[a-z_][a-z0-9_]{0,62}$'
    ),
    participant_logical_index_uuid uuid NOT NULL,
    successor_activation_digest bytea NOT NULL CHECK (
        octet_length(successor_activation_digest) = 32
    ),
    disposition text NOT NULL DEFAULT 'Pending' CHECK (
        disposition IN ('Pending', 'Retired', 'Abandoned')
    ),
    retired_activation_digest bytea CHECK (
        retired_activation_digest IS NULL
        OR octet_length(retired_activation_digest) = 32
    ),
    abandon_audit bytea,
    abandon_audit_digest bytea CHECK (
        abandon_audit_digest IS NULL OR octet_length(abandon_audit_digest) = 32
    ),
    abandon_caller_name text,
    abandon_reason text,
    abandon_time_unix_micros bigint,
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    CHECK (
        (disposition = 'Pending'
         AND retired_activation_digest IS NULL
         AND abandon_audit IS NULL AND abandon_audit_digest IS NULL
         AND abandon_caller_name IS NULL AND abandon_reason IS NULL
         AND abandon_time_unix_micros IS NULL)
        OR
        (disposition = 'Retired'
         AND retired_activation_digest IS NOT NULL
         AND retired_activation_digest = successor_activation_digest
         AND abandon_audit IS NULL AND abandon_audit_digest IS NULL
         AND abandon_caller_name IS NULL AND abandon_reason IS NULL
         AND abandon_time_unix_micros IS NULL)
        OR
        (disposition = 'Abandoned'
         AND retired_activation_digest IS NULL
         AND abandon_audit IS NOT NULL AND octet_length(abandon_audit) > 0
         AND abandon_audit_digest IS NOT NULL
         AND abandon_caller_name IS NOT NULL
         AND length(abandon_caller_name) > 0
         AND abandon_reason IS NOT NULL
         AND octet_length(abandon_reason) BETWEEN 1 AND 1024
         AND abandon_time_unix_micros IS NOT NULL)
    ),
    PRIMARY KEY (
        index_oid, logical_index_uuid, successor_build_id,
        predecessor_roster_ordinal
    ),
    UNIQUE (
        index_oid, logical_index_uuid, successor_build_id,
        participant_logical_index_uuid
    ),
    FOREIGN KEY (
        index_oid, logical_index_uuid, successor_build_id,
        predecessor_build_id, predecessor_epoch,
        predecessor_epoch_fingerprint, predecessor_manifest_digest,
        successor_activation_digest
    )
        REFERENCES ec_distann_publish_decision (
            index_oid, logical_index_uuid, build_id,
            predecessor_build_id, predecessor_epoch,
            predecessor_epoch_fingerprint, predecessor_manifest_digest,
            successor_activation_digest
        ) ON DELETE RESTRICT,
    FOREIGN KEY (
        index_oid, logical_index_uuid, predecessor_build_id,
        predecessor_roster_ordinal, node_id, endpoint_identity,
        remote_index_regclass, participant_logical_index_uuid
    ) REFERENCES ec_distann_build_participant_binding (
        index_oid, logical_index_uuid, build_id, roster_ordinal, node_id,
        endpoint_identity, remote_index_regclass,
        participant_logical_index_uuid
    ) ON DELETE RESTRICT
);

CREATE TABLE ec_distann_retire_decision (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    epoch bigint NOT NULL CHECK (epoch > 0),
    epoch_fingerprint bytea NOT NULL CHECK (octet_length(epoch_fingerprint) = 34),
    manifest_digest bytea NOT NULL CHECK (octet_length(manifest_digest) = 32),
    roster_snapshot bytea NOT NULL CHECK (octet_length(roster_snapshot) > 0),
    roster_digest bytea NOT NULL CHECK (octet_length(roster_digest) = 32),
    abandoned_binding_set bytea NOT NULL CHECK (
        octet_length(abandoned_binding_set) >= 4
    ),
    abandoned_binding_set_digest bytea NOT NULL CHECK (
        octet_length(abandoned_binding_set_digest) = 32
    ),
    retire_decision bytea NOT NULL CHECK (octet_length(retire_decision) > 0),
    retire_decision_digest bytea NOT NULL CHECK (octet_length(retire_decision_digest) = 32),
    forced boolean NOT NULL DEFAULT false,
    overridden_in_flight_count bigint NOT NULL DEFAULT 0 CHECK (overridden_in_flight_count >= 0),
    caller_name text NOT NULL CHECK (length(caller_name) > 0),
    reason text NOT NULL DEFAULT 'normal' CHECK (
        octet_length(reason) BETWEEN 1 AND 1024
    ),
    decision_time_unix_micros bigint NOT NULL,
    covering_successor_build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(covering_successor_build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(covering_successor_build_id), 8) & 192) = 128
    ),
    covering_successor_activation_digest bytea NOT NULL CHECK (
        octet_length(covering_successor_activation_digest) = 32
    ),
    covering_successor_decision_state text NOT NULL DEFAULT 'Applied' CHECK (
        covering_successor_decision_state = 'Applied'
    ),
    decision_state text NOT NULL CHECK (decision_state IN ('Pending', 'Applied')),
    committed_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    applied_at timestamptz,
    CHECK ((decision_state = 'Applied') = (applied_at IS NOT NULL)),
    CHECK (
        (NOT forced AND overridden_in_flight_count = 0 AND reason = 'normal')
        OR forced
    ),
    PRIMARY KEY (index_oid, logical_index_uuid, epoch_fingerprint),
    FOREIGN KEY (
        index_oid, logical_index_uuid, build_id, epoch,
        epoch_fingerprint, manifest_digest
    ) REFERENCES ec_distann_publish_decision (
        index_oid, logical_index_uuid, build_id, epoch,
        epoch_fingerprint, manifest_digest
    ) ON DELETE RESTRICT,
    FOREIGN KEY (
        index_oid, logical_index_uuid, covering_successor_build_id,
        build_id, epoch, epoch_fingerprint, manifest_digest,
        covering_successor_activation_digest,
        covering_successor_decision_state
    ) REFERENCES ec_distann_publish_decision (
        index_oid, logical_index_uuid, build_id,
        predecessor_build_id, predecessor_epoch,
        predecessor_epoch_fingerprint, predecessor_manifest_digest,
        successor_activation_digest, decision_state
    ) ON DELETE RESTRICT
);

CREATE TABLE ec_distann_generation_reclaim (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    epoch bigint NOT NULL CHECK (epoch > 0),
    epoch_fingerprint bytea NOT NULL CHECK (octet_length(epoch_fingerprint) = 34),
    manifest_digest bytea NOT NULL CHECK (octet_length(manifest_digest) = 32),
    build_spec_digest bytea NOT NULL CHECK (octet_length(build_spec_digest) = 32),
    generation_descriptor_digest bytea NOT NULL CHECK (
        octet_length(generation_descriptor_digest) = 32
    ),
    record_count bigint NOT NULL CHECK (record_count >= 0),
    row_count bigint NOT NULL CHECK (row_count >= 0),
    successor_activation_digest bytea CHECK (
        successor_activation_digest IS NULL
        OR octet_length(successor_activation_digest) = 32
    ),
    retire_decision bytea NOT NULL CHECK (octet_length(retire_decision) > 0),
    retire_decision_digest bytea NOT NULL CHECK (octet_length(retire_decision_digest) = 32),
    reclaimed_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (index_oid, logical_index_uuid, build_id),
    UNIQUE (index_oid, logical_index_uuid, epoch_fingerprint)
);

CREATE TABLE ec_distann_active_epoch (
    index_oid oid NOT NULL,
    logical_index_uuid uuid NOT NULL,
    build_id uuid NOT NULL CHECK (
        (get_byte(uuid_send(build_id), 6) & 240) = 64
        AND (get_byte(uuid_send(build_id), 8) & 192) = 128
    ),
    epoch bigint NOT NULL CHECK (epoch > 0),
    epoch_fingerprint bytea NOT NULL CHECK (octet_length(epoch_fingerprint) = 34),
    manifest_digest bytea NOT NULL CHECK (octet_length(manifest_digest) = 32),
    updated_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (index_oid, logical_index_uuid),
    UNIQUE (index_oid, logical_index_uuid, epoch_fingerprint),
    FOREIGN KEY (
        index_oid, logical_index_uuid, build_id, epoch,
        epoch_fingerprint, manifest_digest
    ) REFERENCES ec_distann_publish_decision (
        index_oid, logical_index_uuid, build_id, epoch,
        epoch_fingerprint, manifest_digest
    )
);

REVOKE ALL ON TABLE ec_distann_participant_identity FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_registry_state FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_node_descriptor FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_generation FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_generation_batch FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_build_registration FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_build_participant_binding FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_build_candidate FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_publish_decision FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_predecessor_disposition FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_retire_decision FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_active_epoch FROM PUBLIC;
REVOKE ALL ON TABLE ec_distann_generation_reclaim FROM PUBLIC;

CREATE TYPE ec_spire_placement_entry AS (
    pk_value bytea,
    node_id integer,
    centroid_id bigint,
    served_epoch bigint,
    source_identity bytea
);

CREATE FUNCTION ec_spire_coordinator_insert_shape_fingerprint(table_oid regclass)
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

CREATE FUNCTION ec_spire_coordinator_index_shape_fingerprint(index_oid regclass)
RETURNS text
STABLE STRICT
LANGUAGE sql
AS $$
    SELECT ec_spire_coordinator_insert_shape_fingerprint(indrelid::regclass)
      FROM pg_index
     WHERE indexrelid = index_oid::oid
$$;

CREATE FUNCTION ec_spire_remote_index_shape_fingerprint(index_oid regclass)
RETURNS text
STABLE STRICT
LANGUAGE sql
AS $$
    SELECT ec_spire_coordinator_index_shape_fingerprint(index_oid)
$$;

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
    planned_node_id bigint;
    planned_centroid_id bigint;
    planned_served_epoch bigint;
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

    SELECT node_id, centroid_id, served_epoch
      INTO planned_node_id, planned_centroid_id, planned_served_epoch
      FROM ec_spire_plan_coordinator_insert(
           target_index_oid, pk_value, embedding, source_identity);

    CREATE TEMP TABLE IF NOT EXISTS ec_spire_coordinator_insert_tuple_payload_queue (
        table_oid oid NOT NULL,
        index_oid oid NOT NULL,
        queue_order bigserial,
        pk_value bytea NOT NULL,
        node_id integer NOT NULL,
        centroid_id bigint NOT NULL,
        served_epoch bigint NOT NULL,
        source_identity bytea NOT NULL,
        row_payload_json text NOT NULL,
        requested_columns text[] NOT NULL
    ) ON COMMIT DELETE ROWS;

    INSERT INTO ec_spire_coordinator_insert_tuple_payload_queue
        (table_oid, index_oid, pk_value, node_id, centroid_id, served_epoch,
         source_identity, row_payload_json, requested_columns)
    VALUES
        (TG_RELID, target_index_oid, pk_value, planned_node_id::integer,
         planned_centroid_id, planned_served_epoch, source_identity,
         row_payload::text, requested_columns);

    RETURN NULL;
END
$$;

CREATE FUNCTION ec_spire_coordinator_insert_flush_trigger()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    target_index_oid oid;
    requested_columns text[];
BEGIN
    IF TG_WHEN <> 'AFTER' OR TG_LEVEL <> 'STATEMENT' OR TG_OP <> 'INSERT' THEN
        RAISE EXCEPTION 'ec_spire_coordinator_insert_flush_trigger must be an AFTER INSERT statement trigger'
            USING ERRCODE = '0A000';
    END IF;
    IF TG_NARGS <> 1 THEN
        RAISE EXCEPTION 'ec_spire_coordinator_insert_flush_trigger requires 1 trigger argument'
            USING ERRCODE = '22023',
                  HINT = 'Use ec_spire_enable_coordinator_insert(table_oid, index_oid, pk_column, embedding_column, source_identity_column).';
    END IF;

    target_index_oid := TG_ARGV[0]::oid;
    IF to_regclass('pg_temp.ec_spire_coordinator_insert_tuple_payload_queue') IS NULL THEN
        RETURN NULL;
    END IF;

    SELECT q.requested_columns
      INTO requested_columns
      FROM ec_spire_coordinator_insert_tuple_payload_queue q
     WHERE q.table_oid = TG_RELID
       AND q.index_oid = target_index_oid
     ORDER BY q.queue_order
     LIMIT 1;

    IF requested_columns IS NULL THEN
        RETURN NULL;
    END IF;

    PERFORM 1
      FROM ec_spire_prepare_coordinator_insert_tuple_payload_batch(
           target_index_oid,
           ARRAY(
             SELECT encode(q.pk_value, 'hex')
               FROM ec_spire_coordinator_insert_tuple_payload_queue q
              WHERE q.table_oid = TG_RELID AND q.index_oid = target_index_oid
              ORDER BY q.queue_order
           ),
           ARRAY(
             SELECT q.node_id
               FROM ec_spire_coordinator_insert_tuple_payload_queue q
              WHERE q.table_oid = TG_RELID AND q.index_oid = target_index_oid
              ORDER BY q.queue_order
           ),
           ARRAY(
             SELECT q.centroid_id
               FROM ec_spire_coordinator_insert_tuple_payload_queue q
              WHERE q.table_oid = TG_RELID AND q.index_oid = target_index_oid
              ORDER BY q.queue_order
           ),
           ARRAY(
             SELECT q.served_epoch
               FROM ec_spire_coordinator_insert_tuple_payload_queue q
              WHERE q.table_oid = TG_RELID AND q.index_oid = target_index_oid
              ORDER BY q.queue_order
           ),
           ARRAY(
             SELECT encode(q.source_identity, 'hex')
               FROM ec_spire_coordinator_insert_tuple_payload_queue q
              WHERE q.table_oid = TG_RELID AND q.index_oid = target_index_oid
              ORDER BY q.queue_order
           ),
           ARRAY(
             SELECT q.row_payload_json
               FROM ec_spire_coordinator_insert_tuple_payload_queue q
              WHERE q.table_oid = TG_RELID AND q.index_oid = target_index_oid
              ORDER BY q.queue_order
           ),
           requested_columns
      );

    DELETE FROM ec_spire_coordinator_insert_tuple_payload_queue q
     WHERE q.table_oid = TG_RELID
       AND q.index_oid = target_index_oid;

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

    EXECUTE format('DROP TRIGGER IF EXISTS ec_spire_coordinator_insert_flush ON %s', table_oid);
    EXECUTE format('DROP TRIGGER IF EXISTS ec_spire_coordinator_insert_forward ON %s', table_oid);
    EXECUTE format(
        'CREATE TRIGGER ec_spire_coordinator_insert_forward BEFORE INSERT ON %s FOR EACH ROW EXECUTE FUNCTION ec_spire_coordinator_insert_forward_trigger(%L, %L, %L, %L)',
        table_oid,
        index_oid::oid::text,
        pk_column,
        embedding_column,
        source_identity_column
    );
    EXECUTE format(
        'CREATE TRIGGER ec_spire_coordinator_insert_flush AFTER INSERT ON %s FOR EACH STATEMENT EXECUTE FUNCTION ec_spire_coordinator_insert_flush_trigger(%L)',
        table_oid,
        index_oid::oid::text
    );
END
$$;

CREATE FUNCTION ec_spire_remote_catalog_drop_index_cleanup_event()
RETURNS event_trigger
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, @extschema@
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

REVOKE ALL ON FUNCTION ec_spire_remote_catalog_drop_index_cleanup_event()
    FROM PUBLIC;

CREATE EVENT TRIGGER ec_spire_remote_catalog_drop_index_cleanup
ON sql_drop
EXECUTE FUNCTION ec_spire_remote_catalog_drop_index_cleanup_event();

CREATE FUNCTION ec_distann_catalog_drop_index_cleanup_event()
RETURNS event_trigger
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, @extschema@
AS $$
DECLARE
    dropped_object record;
BEGIN
    FOR dropped_object IN
        SELECT *
          FROM pg_event_trigger_dropped_objects()
         WHERE object_type = 'index'
    LOOP
        -- Registry state exists for every v5 logical control and for no
        -- ordinary or hidden directory index. Avoid the full multi-catalog
        -- cleanup path for unrelated database-wide index drops.
        IF EXISTS (
            SELECT 1 FROM ec_distann_registry_state
             WHERE index_oid = dropped_object.objid::oid
        ) THEN
            PERFORM ec_distann_catalog_index_cleanup(dropped_object.objid::oid);
        END IF;
    END LOOP;
END
$$;

REVOKE ALL ON FUNCTION ec_distann_catalog_drop_index_cleanup_event() FROM PUBLIC;

CREATE EVENT TRIGGER ec_distann_catalog_drop_index_cleanup
ON sql_drop
EXECUTE FUNCTION ec_distann_catalog_drop_index_cleanup_event();

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

CREATE FUNCTION ec_distann_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_distann_handler';

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
CREATE ACCESS METHOD ec_distann TYPE INDEX HANDLER ec_distann_handler;

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

CREATE OPERATOR CLASS tqvector_distann_ip_ops
DEFAULT FOR TYPE tqvector USING ec_distann AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);

CREATE OPERATOR CLASS ecvector_distann_ip_ops
DEFAULT FOR TYPE ecvector USING ec_distann AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);
