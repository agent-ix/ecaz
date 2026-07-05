\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n2048_b1_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('7a49a7269cd213cc', 'hex'), 't139_50k_n2048_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n2048_b1_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('6b52334ab7bf6020', 'hex'), 't139_50k_n2048_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n2048_b1_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('8923c81b95ee89c4', 'hex'), 't139_50k_n2048_b1_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

