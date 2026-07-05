\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t142_release_50k_n1024_b0_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('7c1b30c4ca472c53', 'hex'), 't142_release_50k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t142_release_50k_n1024_b0_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('6e7d7a9b25da5e5c', 'hex'), 't142_release_50k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t142_release_50k_n1024_b0_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('00808a32af4b5ad2', 'hex'), 't142_release_50k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;
