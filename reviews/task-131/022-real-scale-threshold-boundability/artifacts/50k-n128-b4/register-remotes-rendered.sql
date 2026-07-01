\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_50k_n128_b4_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('79b7985c39488e1c', 'hex'), 't131_p3_bound_q20_50k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_50k_n128_b4_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('72dd5a20ad8519c0', 'hex'), 't131_p3_bound_q20_50k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_50k_n128_b4_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('5406357ffb765e0f', 'hex'), 't131_p3_bound_q20_50k_n128_b4_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

