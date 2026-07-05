\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('04afca4e8ea81c66', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('cec80156fc61f5e4', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('0ca3055d68098a9f', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

