\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_10k_n1024_b2_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('ddec5deeda2acdc8', 'hex'), 't131_p4_mi_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_10k_n1024_b2_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('d62f5ffc292de84e', 'hex'), 't131_p4_mi_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t131_p4_mi_10k_n1024_b2_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('a384d208f00ac3ef', 'hex'), 't131_p4_mi_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

