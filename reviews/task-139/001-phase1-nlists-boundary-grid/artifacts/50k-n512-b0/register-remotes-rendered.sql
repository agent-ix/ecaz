\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n512_b0_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('e4321d572778657a', 'hex'), 't139_50k_n512_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n512_b0_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('cd9903603857aea1', 'hex'), 't139_50k_n512_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t139_50k_n512_b0_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('b0699afca59711da', 'hex'), 't139_50k_n512_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

