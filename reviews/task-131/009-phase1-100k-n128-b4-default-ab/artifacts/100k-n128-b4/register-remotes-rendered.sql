\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_100k_n128_b4_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('5834b0a39d5f9219', 'hex'), 't131_p4_mi_100k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_100k_n128_b4_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('df01516eac6136a8', 'hex'), 't131_p4_mi_100k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_100k_n128_b4_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('3ae97b1703df7b9a', 'hex'), 't131_p4_mi_100k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

