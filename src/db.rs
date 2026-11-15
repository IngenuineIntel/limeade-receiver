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
    const INS_PROC_ATTRIBUTES: &str = include_str!("sql/ins_proc_attributes.sql");
    const GET_PROC_ATTRIBUTES: &str = include_str!("sql/get_proc_attributes.sql");
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

        fn insert_procs(&self, procs: &limeade::Procs) -> Result<(), Box<dyn std::error::Error>> {
            if procs.procs.is_null() {
                return Err("procs array is NULL".into());
            }
            let transaction = self.conn.unchecked_transaction()?;
            transaction.execute(
                "INSERT INTO proc_snapshots (ts_s, ts_ms) VALUES (?1, ?2)",
                params![procs.ts_s, procs.ts_ms],
            )?;
            let snapshot_id = transaction.last_insert_rowid();
            {
                let mut insert_attributes = transaction.prepare_cached(INS_PROC_ATTRIBUTES)?;
                let mut get_attributes = transaction.prepare_cached(GET_PROC_ATTRIBUTES)?;
                let mut insert_sample = transaction.prepare_cached(INS_PROC)?;
                for proc in c_arr!(procs.procs, procs.total) {
                    let name = c_string(proc.name);
                    let command = c_string(proc.command);
                    let attributes = params![
                        proc.ppid, proc.ruid, proc.euid, proc.suid, proc.fsuid,
                        proc.rgid, proc.egid, proc.sgid, proc.fsgid, name, command
                    ];
                    insert_attributes.execute(attributes)?;
                    let attribute_id: i64 = get_attributes.query_row(attributes, |row| row.get(0))?;
                    insert_sample.execute(params![
                        snapshot_id, attribute_id, proc.pid, proc.threads, proc.ram_kb,
                        proc.swap_kb, proc.state.to_string(), proc.cpu_user, proc.cpu_sys
                    ])?;
                }
            }
            transaction.commit()?;
            Ok(())
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
                            std::process::id()
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
                    self.insert_procs(&procs)?;
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
        use std::ffi::CString;

        fn in_memory_store() -> PacketSerializer {
            let conn = Connection::open_in_memory().unwrap();
            conn.execute_batch(DB_INIT).unwrap();
            PacketSerializer { conn }
        }

        #[test]
        fn process_snapshots_reconstruct_and_deduplicate() {
            let store = in_memory_store();
            let name = CString::new("worker").unwrap();
            let command = CString::new("/bin/worker --foreground").unwrap();
            let changed_command = CString::new("/bin/worker --reload").unwrap();
            let mut process = limeade::IndividualProc {
                pid: 3, ppid: 4, ruid: 5, euid: 6, suid: 7, fsuid: 8,
                rgid: 9, egid: 10, sgid: 11, fsgid: 12, threads: 13,
                ram_kb: 14, swap_kb: 15, state: 82, cpu_user: 16, cpu_sys: 17,
                name: name.as_ptr().cast_mut(), command: command.as_ptr().cast_mut(),
            };
            let mut snapshot = limeade::Procs {
                ts_s: 100, ts_ms: 200, total: 1, procs: &mut process,
            };
            store.insert_procs(&snapshot).unwrap();
            process.ram_kb = 42;
            snapshot.procs = &mut process;
            store.insert_procs(&snapshot).unwrap();
            process.command = changed_command.as_ptr().cast_mut();
            process.euid = 99;
            snapshot.ts_s = 101;
            snapshot.procs = &mut process;
            store.insert_procs(&snapshot).unwrap();
            process.name = std::ptr::null_mut();
            process.command = std::ptr::null_mut();
            snapshot.procs = &mut process;
            store.insert_procs(&snapshot).unwrap();

            let count = |table: &str| -> i64 {
                store.conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get(0)).unwrap()
            };
            assert_eq!(count("proc_snapshots"), 4);
            assert_eq!(count("proc_attributes"), 3);
            assert_eq!(count("proc_samples"), 4);
            let mut statement = store.conn.prepare("SELECT * FROM procs ORDER BY id").unwrap();
            let rows: Vec<Vec<rusqlite::types::Value>> = statement.query_map([], |row| {
                (0..21).map(|column| row.get(column)).collect()
            }).unwrap().map(Result::unwrap).collect();
            use rusqlite::types::Value::{Integer, Text};
            assert_eq!(rows[0], vec![
                Integer(1), Integer(100), Integer(200), Integer(3), Integer(4),
                Integer(5), Integer(6), Integer(7), Integer(8), Integer(9),
                Integer(10), Integer(11), Integer(12), Integer(13), Integer(14),
                Integer(15), Text("82".into()), Integer(16), Integer(17),
                Text("worker".into()), Text("/bin/worker --foreground".into()),
            ]);
            let mut expected = rows[0].clone();
            expected[0] = Integer(2);
            expected[14] = Integer(42);
            assert_eq!(rows[1], expected);
            expected[0] = Integer(3);
            expected[1] = Integer(101);
            expected[6] = Integer(99);
            expected[20] = Text("/bin/worker --reload".into());
            assert_eq!(rows[2], expected);
            expected[0] = Integer(4);
            expected[19] = Text("".into());
            expected[20] = Text("".into());
            assert_eq!(rows[3], expected);
        }

        #[test]
        fn process_packet_keeps_each_sample_in_order() {
            let store = in_memory_store();
            let mut processes: [limeade::IndividualProc; 2] = unsafe { std::mem::zeroed() };
            processes[0].pid = 20;
            processes[1].pid = 10;
            let snapshot = limeade::Procs {
                ts_s: 123, ts_ms: 456, total: 2, procs: processes.as_mut_ptr(),
            };
            store.insert_procs(&snapshot).unwrap();
            let mut statement = store.conn.prepare("SELECT ts_s, ts_ms, pid FROM procs ORDER BY id").unwrap();
            let rows: Vec<(i64, i64, i64)> = statement.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            }).unwrap().map(Result::unwrap).collect();
            assert_eq!(rows, vec![(123, 456, 20), (123, 456, 10)]);
            let attributes: i64 = store.conn.query_row("SELECT count(*) FROM proc_attributes", [], |row| row.get(0)).unwrap();
            assert_eq!(attributes, 1);
        }

        #[test]
        fn process_snapshot_failure_rolls_back() {
            let store = in_memory_store();
            store.conn.execute_batch("CREATE TRIGGER reject_sample BEFORE INSERT ON proc_samples BEGIN SELECT RAISE(FAIL, 'test failure'); END;").unwrap();
            let mut process: limeade::IndividualProc = unsafe { std::mem::zeroed() };
            let snapshot = limeade::Procs { ts_s: 1, ts_ms: 2, total: 1, procs: &mut process };
            assert!(store.insert_procs(&snapshot).is_err());
            for table in ["proc_snapshots", "proc_attributes", "proc_samples"] {
                let count: i64 = store.conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get(0)).unwrap();
                assert_eq!(count, 0);
            }
        }

        #[test]
        fn packet_inserts_match_schema() {
            let conn = Connection::open_in_memory().unwrap();
            conn.execute_batch(DB_INIT).unwrap();
            conn.execute(INS_INTRO, params![1, 2, "host", "kernel", "distro", "user", "cpu", "vendor", 4, 8, 16, 123]).unwrap();
            conn.execute(INS_EVENTS, params![1, 2, 3, "open", "arg1", "arg2", 0]).unwrap();
            conn.execute("INSERT INTO proc_snapshots (ts_s, ts_ms) VALUES (?1, ?2)", params![1, 2]).unwrap();
            conn.execute(INS_PROC_ATTRIBUTES, params![4, 5, 6, 7, 8, 9, 10, 11, 12, "name", "command"]).unwrap();
            conn.execute(INS_PROC, params![1, 1, 3, 13, 14, 15, "R", 16, 17]).unwrap();
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
