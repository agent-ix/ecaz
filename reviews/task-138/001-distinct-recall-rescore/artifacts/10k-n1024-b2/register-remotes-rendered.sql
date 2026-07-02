\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t138_10k_n1024_b2_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('d69d86eed6695173', 'hex'), 't138_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t138_10k_n1024_b2_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('cd04c0fc23d8318d', 'hex'), 't138_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t138_10k_n1024_b2_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('ac791108f5324820', 'hex'), 't138_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

