\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_10k_n128_b4_ctl66_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('137068320a1b1214', 'hex'), 't131_p3_bound_q20_10k_n128_b4_ctl66_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_10k_n128_b4_ctl66_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('f25c96085a0d69fe', 'hex'), 't131_p3_bound_q20_10k_n128_b4_ctl66_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t131_p3_bound_q20_10k_n128_b4_ctl66_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('88564f292e5e106c', 'hex'), 't131_p3_bound_q20_10k_n128_b4_ctl66_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

