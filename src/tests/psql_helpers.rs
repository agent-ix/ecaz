    struct PsqlTestConnection {
        psql_bin: PathBuf,
        host: String,
        port: String,
        database: String,
        user: String,
    }

    fn executable_on_path(binary: &str) -> Option<PathBuf> {
        let paths = std::env::var_os("PATH")?;
        std::env::split_paths(&paths)
            .map(|path| path.join(binary))
            .find(|candidate| candidate.is_file())
    }

    fn pgrx_psql_candidate(root: &Path, server_version: &str) -> PathBuf {
        root.join(server_version)
            .join("pgrx-install")
            .join("bin")
            .join("psql")
    }

    fn resolve_pg_test_psql_bin(server_version: &str) -> PathBuf {
        if let Some(configured) = std::env::var_os("TQV_PSQL_BIN") {
            return PathBuf::from(configured);
        }
        if let Some(psql) = executable_on_path("psql") {
            return psql;
        }
        if let Some(pgrx_home) = std::env::var_os("PGRX_HOME") {
            let candidate = pgrx_psql_candidate(Path::new(&pgrx_home), server_version);
            if candidate.is_file() {
                return candidate;
            }
        }
        if let Some(home) = std::env::var_os("HOME") {
            let candidate = pgrx_psql_candidate(&PathBuf::from(home).join(".pgrx"), server_version);
            if candidate.is_file() {
                return candidate;
            }
        }
        PathBuf::from("psql")
    }

    fn pg_test_psql_connection() -> PsqlTestConnection {
        let socket_dirs = Spi::get_one::<String>("SHOW unix_socket_directories")
            .expect("SPI query should succeed")
            .expect("unix socket setting should exist");
        let host = socket_dirs
            .split(',')
            .map(str::trim)
            .find(|entry| !entry.is_empty())
            .unwrap_or("localhost")
            .to_owned();
        let port = Spi::get_one::<String>("SHOW port")
            .expect("SPI query should succeed")
            .expect("port setting should exist");
        let database = Spi::get_one::<String>("SELECT current_database()::text")
            .expect("SPI query should succeed")
            .expect("current database should exist");
        let user = Spi::get_one::<String>("SELECT current_user::text")
            .expect("SPI query should succeed")
            .expect("current user should exist");
        let server_version = Spi::get_one::<String>("SHOW server_version")
            .expect("SPI query should succeed")
            .expect("server_version setting should exist");
        let psql_bin = resolve_pg_test_psql_bin(&server_version);

        PsqlTestConnection {
            psql_bin,
            host,
            port,
            database,
            user,
        }
    }

    fn psql_command(connection: &PsqlTestConnection) -> Command {
        let mut command = Command::new(&connection.psql_bin);
        command
            .arg("-X")
            .arg("-v")
            .arg("ON_ERROR_STOP=1")
            .arg("-q")
            .arg("-h")
            .arg(&connection.host)
            .arg("-p")
            .arg(&connection.port)
            .arg("-d")
            .arg(&connection.database)
            .arg("-U")
            .arg(&connection.user);
        command
    }

    fn assert_psql_success(label: &str, output: Output) {
        assert!(
            output.status.success(),
            "{label} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn run_psql_script(connection: &PsqlTestConnection, label: &str, sql: &str) {
        let output = psql_command(connection)
            .arg("-c")
            .arg(sql)
            .output()
            .unwrap_or_else(|e| panic!("{label} could not start psql: {e}"));
        assert_psql_success(label, output);
    }

    fn spawn_psql_script(connection: &PsqlTestConnection, label: &str, sql: &str) -> Child {
        psql_command(connection)
            .arg("-c")
            .arg(sql)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("{label} could not start psql: {e}"))
    }

    fn spawn_psql_commands(
        connection: &PsqlTestConnection,
        label: &str,
        sql_commands: &[String],
    ) -> Child {
        let mut command = psql_command(connection);
        for sql in sql_commands {
            command.arg("-c").arg(sql);
        }
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("{label} could not start psql: {e}"))
    }

    fn wait_for_advisory_lock_waiters(barrier_key: i64, expected_waiters: i64) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let waiters = Spi::get_one::<i64>(&format!(
                "SELECT count(*)::bigint FROM pg_locks
                 WHERE locktype = 'advisory'
                   AND mode = 'ShareLock'
                   AND classid = 0
                   AND objid = {barrier_key}
                   AND objsubid = 1
                   AND NOT granted"
            ))
            .expect("pg_locks waiter query should succeed")
            .expect("pg_locks waiter count should exist");
            if waiters >= expected_waiters {
                return;
            }
            if Instant::now() >= deadline {
                panic!(
                    "timed out waiting for {expected_waiters} advisory-lock waiters on key \
                     {barrier_key}; observed {waiters}"
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
