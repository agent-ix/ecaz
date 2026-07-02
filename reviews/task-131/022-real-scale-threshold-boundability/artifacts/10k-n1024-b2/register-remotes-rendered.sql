\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_10k_n1024_b2_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('d69a20eed6666e4a', 'hex'), 't131_p3_bound_q20_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_10k_n1024_b2_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('cd015afc23d54e64', 'hex'), 't131_p3_bound_q20_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_10k_n1024_b2_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('ac7c7708f5352b49', 'hex'), 't131_p3_bound_q20_10k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

