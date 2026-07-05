\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n1024_b1_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('a2ec497f24c48b00', 'hex'), 't139_50k_n1024_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n1024_b1_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('d1df4f7968255a47', 'hex'), 't139_50k_n1024_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n1024_b1_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('38e69579a23e103e', 'hex'), 't139_50k_n1024_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

