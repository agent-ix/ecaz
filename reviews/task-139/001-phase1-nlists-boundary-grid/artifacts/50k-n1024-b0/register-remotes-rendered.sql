\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n1024_b0_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('9e8e0286df06a8d2', 'hex'), 't139_50k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n1024_b0_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('4d902d963a3c6553', 'hex'), 't139_50k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n1024_b0_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('8212c3c277b03de8', 'hex'), 't139_50k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

