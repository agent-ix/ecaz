\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_50k_n1024_b2_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('48adc34d622f6b7e', 'hex'), 't131_p3_bound_q20_50k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_50k_n1024_b2_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('dc69d45857329a66', 'hex'), 't131_p3_bound_q20_50k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_50k_n1024_b2_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('91f08da2da44dd7f', 'hex'), 't131_p3_bound_q20_50k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

