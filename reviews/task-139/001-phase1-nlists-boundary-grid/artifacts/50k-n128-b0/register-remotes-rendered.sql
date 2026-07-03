\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n128_b0_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('c933e53fcd4064c0', 'hex'), 't139_50k_n128_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n128_b0_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('15bb3b6109266a0e', 'hex'), 't139_50k_n128_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n128_b0_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('72dc181fff25bdf1', 'hex'), 't139_50k_n128_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

