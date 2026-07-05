\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n316_b1_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('1584b5c7ca091177', 'hex'), 't139_50k_n316_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n316_b1_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('0add53b318c8cf73', 'hex'), 't139_50k_n316_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n316_b1_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('7291417504d64b93', 'hex'), 't139_50k_n316_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

