-- Phase 13b.6 — register one remote on the coordinator.
-- Operator invokes via `ecaz dev sql --file ... --set coord_index=... --set node_id=...`.
-- All variables come from the topology JSON or the Phase 13b register.sh wrapper.

\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor(
  :'coord_index'::regclass::oid,
  (:'node_id')::int,
  (:'descriptor_generation')::bigint,
  :'conninfo_secret',
  decode(lpad(to_hex((:'node_id')::int), 2, '0'), 'hex'),
  :'remote_index',
  :'state',
  (:'served_epoch')::bigint,
  (:'min_retained_epoch')::bigint,
  :'extversion',
  'none'
) AS registered;
