-- Phase 13b.6 legacy helper — register one remote on the coordinator.
-- Prefer `scripts/spire-aws/register.sh` with a distributed placement plan so
-- the remote identity is queried from the live remote endpoint.

\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor(
  :'coord_index'::regclass::oid,
  (:'node_id')::int,
  (:'descriptor_generation')::bigint,
  :'conninfo_secret',
  decode(:'remote_index_identity_hex', 'hex'),
  :'remote_index',
  :'state',
  (:'served_epoch')::bigint,
  (:'min_retained_epoch')::bigint,
  :'extversion',
  'none'
) AS registered;
