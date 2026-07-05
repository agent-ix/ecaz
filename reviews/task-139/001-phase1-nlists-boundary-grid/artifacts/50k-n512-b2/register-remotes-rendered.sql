\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n512_b2_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('f65479dfb4d58a7f', 'hex'), 't139_50k_n512_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n512_b2_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('6be32b60bb096706', 'hex'), 't139_50k_n512_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n512_b2_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('257023da05627f27', 'hex'), 't139_50k_n512_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

