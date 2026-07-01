\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_10k_n128_b4_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('0a645f3204df57c4', 'hex'), 't131_p4_mi_10k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_10k_n128_b4_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('f25264085a04c083', 'hex'), 't131_p4_mi_10k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_10k_n128_b4_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('915b8c293394046a', 'hex'), 't131_p4_mi_10k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

