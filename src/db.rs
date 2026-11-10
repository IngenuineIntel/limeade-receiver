// db.rs

pub mod db {
    use std::ffi::CStr;
    use std::fs::{OpenOptions, Permissions};
    use std::os::raw::c_char;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    use std::path::PathBuf;

    use chrono::Local;
    use rusqlite::{Connection, Result, params};

    use crate::compat::limeade;

    const DB_INIT: &str = include_str!("sql/init.sql");
    const INS_INTRO: &str = include_str!("sql/ins_intro.sql");
    const INS_EVENTS: &str = include_str!("sql/ins_events.sql");
    const INS_PROC: &str = include_str!("sql/ins_proc.sql");
    const INS_PERF: &str = include_str!("sql/ins_perf.sql");

    fn c_string(value: *const c_char) -> String {
        if value.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() }
    }

    macro_rules! c_arr {
        ($arr:expr, $sz:expr) => {
            unsafe { std::slice::from_raw_parts($arr, $sz as usize) }
        };
    }

    fn resolve_db_path(dir: &String) -> PathBuf {
        PathBuf::from(dir).join(Local::now().format("%m-%d-%Y_%H-%M-%S-%.3f.db").to_string())
    }

    pub struct PacketSerializer {
        conn: rusqlite::Connection,
    }

    impl PacketSerializer {
        pub fn connect(db_directory: &String) -> Result<Self, Box<dyn std::error::Error>> {
            let path = resolve_db_path(db_directory);
            let file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path)?;
            file.set_permissions(Permissions::from_mode(0o660))?;
            let conn = Connection::open(path)?;
            conn.execute_batch(DB_INIT)?;

            Ok(Self { conn })
        }

        pub fn handle_packet(
            &self,
            pkt: &mut limeade::PacketData,
        ) -> Result<(), Box<dyn std::error::Error>> {
            match limeade::pkt_type(pkt) {
                limeade::TYPE_INTRO => {
                    let intro = limeade::parse_intro(pkt)?;
                    let timestamp = Local::now();

                    self.conn.execute(
                        INS_INTRO,
                        params![
                            timestamp.timestamp(),
                            timestamp.timestamp_subsec_millis(),
                            c_string(intro.hostname),
                            c_string(intro.kernelver),
                            c_string(intro.distro),
                            c_string(intro.origin_user),
                            c_string(intro.processor),
                            c_string(intro.vendor),
                            intro.nr_cores,
                            intro.ram_mbs,
                            intro.swap_mbs,
                        ],
                    )?;
                }
                limeade::TYPE_EVENTS => {
                    let events = limeade::parse_events(pkt)?;
                    if events.events.is_null() {
                        return Err("events array is NULL".into());
                    }
                    let transaction = self.conn.unchecked_transaction()?;
                    for event in c_arr!(events.events, events.nr_events) {
                        transaction.execute(
                            INS_EVENTS,
                            params![
                                event.ts_s,
                                event.ts_ms,
                                event.pid,
                                c_string(event.syscall),
                                c_string(event.arg1),
                                c_string(event.arg2),
                                event.retval
                            ],
                        )?;
                    }
                    transaction.commit()?;
                }
                limeade::TYPE_PROC => {
                    let procs = limeade::parse_proc(pkt)?;
                    if procs.procs.is_null() {
                        return Err("procs array is NULL".into());
                    }
                    let transaction = self.conn.unchecked_transaction()?;
                    for proc in c_arr!(procs.procs, procs.total) {
                        transaction.execute(
                            INS_PROC,
                            params![
                                procs.ts_s,
                                procs.ts_ms,
                                proc.pid,
                                proc.ppid,
                                proc.ruid,
                                proc.euid,
                                proc.suid,
                                proc.fsuid,
                                proc.rgid,
                                proc.egid,
                                proc.sgid,
                                proc.fsgid,
                                proc.threads,
                                proc.ram_kb,
                                proc.swap_kb,
                                proc.state.to_string(),
                                proc.cpu_user,
                                proc.cpu_sys,
                                c_string(proc.name),
                                c_string(proc.command)
                            ]
                        )?;
                    }
                    transaction.commit()?;
                }
                limeade::TYPE_PERF => {
                    let perf = limeade::parse_perf(pkt)?;
                    self.conn.execute(
                        INS_PERF,
                        params![
                            perf.ts_s,
                            perf.ts_ms,
                            perf.perc_cpu_user,
                            perf.perc_cpu_sys,
                            perf.mem_total_kb,
                            perf.mem_free_kb,
                            perf.mem_available_kb,
                            perf.mem_cached_kb,
                        ],
                    )?;
                }
                _ => {}
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn packet_inserts_match_schema() {
            let conn = Connection::open_in_memory().unwrap();
            conn.execute_batch(DB_INIT).unwrap();
            conn.execute(INS_INTRO, params![1, 2, "host", "kernel", "distro", "user", "cpu", "vendor", 4, 8, 16]).unwrap();
            conn.execute(INS_EVENTS, params![1, 2, 3, "open", "arg1", "arg2", 0]).unwrap();
            conn.execute(INS_PROC, params![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, "R", 16, 17, "name", "command"]).unwrap();
            conn.execute(INS_PERF, params![1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
            let syscall: String = conn.query_row("SELECT syscall FROM events", [], |row| row.get(0)).unwrap();
            assert_eq!(syscall, "open");
            let timestamp: (u32, u32) = conn.query_row("SELECT ts_s, ts_ms FROM perf", [], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
            assert_eq!(timestamp, (1, 2));
        }

        #[test]
        fn initializes_database_schema() {
            let directory = std::env::temp_dir().join(format!(
                "limeade-receiver-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
            ));
            std::fs::create_dir(&directory).unwrap();
            let store = PacketSerializer::connect(&directory.to_string_lossy().into_owned()).unwrap();
            let entry = std::fs::read_dir(&directory).unwrap().next().unwrap().unwrap();
            assert_eq!(entry.metadata().unwrap().permissions().mode() & 0o777, 0o660);
            drop(store);
            std::fs::remove_dir_all(directory).unwrap();
        }
    }
}
