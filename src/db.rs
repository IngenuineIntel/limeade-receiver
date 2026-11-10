// db.rs

mod db {
    use std::path::PathBuf;

    use chrono::Local;
    use rusqlite::{Connection, Result};

    fn resolve_db_path(dir: &String) -> PathBuf {
        PathBuf::from(dir).join(Local::now().format("%m-%d-%Y_%H-%M-%S-%.3f.db").to_string())
    }

    struct PacketSerializer {
        conn: rusqlite::Connection,
    }

    impl PacketSerializer {
        fn connect(mut self, db_directory: &String) -> Result<(), Box<dyn std::error::Error>> {
            self.conn = Connection::open(resolve_db_path(db_directory))?;

            self.conn.execute(
                "CREATE TABLE client_data (
                      ts_s INTEGER NOT NULL,
                    , ts_ms INTEGER NOT NULL,
                    , hostname TEXT NOT NULL,
                    , kernelver TEXT NOT NULL,
                    , distro TEXT NOT NULL,
                    , origin_user TEXT NOT NULL,
                    , processor TEXT NOT NULL,
                    , vendor TEXT NOT NULL,
                    , nr_cores INTEGER NOT NULL,
                    , ram_mbs INTEGER NOT NULL,
                    , swap_mbs INTEGER NOT NULL
                )",
                (),
            )?;

            self.conn.execute(
                "CREATE TABLE events (
                      ts_s INTEGER NOT NULL,
                    , ts_ms INTEGER NOT NULL,
                    , pid INTEGER NOT NULL,
                    , syscall TEXT NOT NULL,
                    , arg1 TEXT,
                    , arg2 TEXT,
                    , retval INTEGER NOT NULL
                )",
                (),
            )?;

            self.conn.execute(
                "CREATE TABLE perf (
                    perc_cpu_user INTEGER NOT NULL,
                    perc_cpu_sys INTEGER NOT NULL,
                    mem_total_kb INTEGER NOT NULL,
                    mem_free_kb INTEGER NOT NULL,
                    mem_available_kb INTEGER NOT NULL,
                    mem_cached_kb INTEGER NOT NULL,
                )",
                (),
            )?;

            Ok(())
        }
    }
}
