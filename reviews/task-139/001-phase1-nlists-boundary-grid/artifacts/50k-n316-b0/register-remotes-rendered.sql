\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n316_b0_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('02b050c5f376454d', 'hex'), 't139_50k_n316_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n316_b0_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('facef7ef7f58f089', 'hex'), 't139_50k_n316_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n316_b0_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('5cd5b5de194fb6f1', 'hex'), 't139_50k_n316_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

