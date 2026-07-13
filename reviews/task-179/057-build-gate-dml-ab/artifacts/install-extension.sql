CREATE EXTENSION ecaz CASCADE;

SELECT format(
    '[suite-result] dml_gate_extension lane=installed extension_installed=%s',
    count(*)
)
FROM pg_catalog.pg_extension
WHERE extname = 'ecaz';
