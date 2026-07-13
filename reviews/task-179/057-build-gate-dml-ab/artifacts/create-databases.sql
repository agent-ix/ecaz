SELECT pg_catalog.pg_terminate_backend(pid)
  FROM pg_catalog.pg_stat_activity
 WHERE datname IN ('task179_gate_control', 'task179_gate_installed')
   AND pid <> pg_catalog.pg_backend_pid();

DROP DATABASE IF EXISTS task179_gate_control;
DROP DATABASE IF EXISTS task179_gate_installed;
CREATE DATABASE task179_gate_control TEMPLATE template0;
CREATE DATABASE task179_gate_installed TEMPLATE template0;

SELECT '[suite-result] dml_gate_databases created=2';
