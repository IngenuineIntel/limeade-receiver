// db.rs

pub mod db {
    use std::ffi::CStr;
    use std::fs::{OpenOptions, Permissions};
    use std::io::Write;
    use std::os::raw::c_char;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    use std::path::PathBuf;

    use chrono::Local;
    use rusqlite::{Connection, Result, params};

    use crate::compat::limeade;

    const DB_INIT: &str = include_str!("sql/init.sql");
    const INS_INTRO: &str = include_str!("sql/ins_intro.sql");
    const INS_EVENTS: &str = include_str!("sql/ins_events.sql");
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

    pub fn export_procs(
        conn: &Connection,
        output: impl Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let transaction = conn.unchecked_transaction()?;
        let mut writer = csv::Writer::from_writer(output);
        writer.write_record([
            "id", "ts_s", "ts_ms", "pid", "ppid", "ruid", "euid", "suid", "fsuid", "rgid", "egid",
            "sgid", "fsgid", "threads", "ram_kb", "swap_kb", "state", "cpu_user", "cpu_sys",
            "name", "command",
        ])?;
        {
            let mut batches = transaction.prepare(
                "SELECT b.first_sample_id, b.sample_count, b.data, s.ts_s, s.ts_ms \
                 FROM proc_snapshots s LEFT JOIN proc_batches b ON b.snapshot_id = s.id ORDER BY s.id"
            )?;
            let mut attributes = transaction.prepare(
                "SELECT ppid, ruid, euid, suid, fsuid, rgid, egid, sgid, fsgid, name, command \
                 FROM proc_attributes WHERE id = ?1",
            )?;
            let mut rows = batches.query([])?;
            let mut next_id = 1i64;
            while let Some(row) = rows.next()? {
                let first_id: i64 = row.get(0)?;
                let count: u16 = row.get(1)?;
                let data: Vec<u8> = row.get(2)?;
                let ts_s: i64 = row.get(3)?;
                let ts_ms: i64 = row.get(4)?;
                let samples = crate::proc_batch::decode(&data)?;
                if samples.len() != count as usize || first_id != next_id {
                    return Err("invalid process batch count or sample IDs".into());
                }
                for sample in samples {
                    let (ids, name, command) =
                        attributes.query_row([sample.attribute_id], |attr| {
                            Ok((
                                (0..9)
                                    .map(|index| attr.get::<_, i64>(index))
                                    .collect::<Result<Vec<_>>>()?,
                                attr.get::<_, String>(9)?,
                                attr.get::<_, String>(10)?,
                            ))
                        })?;
                    let mut record = vec![
                        next_id.to_string(),
                        ts_s.to_string(),
                        ts_ms.to_string(),
                        sample.pid.to_string(),
                    ];
                    record.extend(ids.into_iter().map(|value| value.to_string()));
                    record.extend([
                        sample.threads.to_string(),
                        sample.ram_kb.to_string(),
                        sample.swap_kb.to_string(),
                        sample.state.to_string(),
                        sample.cpu_user.to_string(),
                        sample.cpu_sys.to_string(),
                        name,
                        command,
                    ]);
                    writer.write_record(&record)?;
                    next_id = next_id.checked_add(1).ok_or("process sample ID overflow")?;
                }
            }
        }
        writer.flush()?;
        transaction.commit()?;
        Ok(())
    }

    pub struct PacketSerializer {
        conn: rusqlite::Connection,
    }

    impl PacketSerializer {
        pub fn connect(db_directory: &String) -> Result<Self, Box<dyn std::error::Error>> {
            let path = resolve_db_path(db_directory);
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)?;
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
            let first_sample_id: i64 = transaction.query_row(
                "SELECT COALESCE((SELECT first_sample_id + sample_count FROM proc_batches ORDER BY snapshot_id DESC LIMIT 1), 1)",
                [], |row| row.get(0),
            )?;
            first_sample_id
                .checked_add(i64::from(procs.total))
                .ok_or("process sample ID overflow")?;
            transaction.execute(
                "INSERT INTO proc_snapshots (ts_s, ts_ms) VALUES (?1, ?2)",
                params![procs.ts_s, procs.ts_ms],
            )?;
            let snapshot_id = transaction.last_insert_rowid();
            let mut samples = Vec::with_capacity(procs.total as usize);
            {
                let mut insert_attributes = transaction.prepare_cached(INS_PROC_ATTRIBUTES)?;
                let mut get_attributes = transaction.prepare_cached(GET_PROC_ATTRIBUTES)?;
                for proc in c_arr!(procs.procs, procs.total) {
                    let name = c_string(proc.name);
                    let command = c_string(proc.command);
                    let attributes = params![
                        proc.ppid, proc.ruid, proc.euid, proc.suid, proc.fsuid, proc.rgid,
                        proc.egid, proc.sgid, proc.fsgid, name, command
                    ];
                    insert_attributes.execute(attributes)?;
                    let attribute_id: i64 =
                        get_attributes.query_row(attributes, |row| row.get(0))?;
                    samples.push(crate::proc_batch::Sample {
                        attribute_id,
                        pid: proc.pid,
                        threads: proc.threads,
                        ram_kb: proc.ram_kb,
                        swap_kb: proc.swap_kb,
                        state: proc.state,
                        cpu_user: proc.cpu_user,
                        cpu_sys: proc.cpu_sys,
                    });
                }
            }
            let data = crate::proc_batch::encode(&samples)?;
            transaction.execute(
                "INSERT INTO proc_batches (snapshot_id, first_sample_id, sample_count, data) VALUES (?1, ?2, ?3, ?4)",
                params![snapshot_id, first_sample_id, procs.total, data],
            )?;
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

        fn exported_rows(store: &PacketSerializer) -> Vec<Vec<String>> {
            let mut output = Vec::new();
            export_procs(&store.conn, &mut output).unwrap();
            csv::Reader::from_reader(output.as_slice())
                .records()
                .map(|record| record.unwrap().iter().map(str::to_owned).collect())
                .collect()
        }

        #[test]
        fn process_snapshots_reconstruct_and_deduplicate() {
            let store = in_memory_store();
            let name = CString::new("worker").unwrap();
            let command = CString::new("/bin/worker --foreground").unwrap();
            let changed_command = CString::new("/bin/worker --reload").unwrap();
            let mut process = limeade::IndividualProc {
                pid: 3,
                ppid: 4,
                ruid: 5,
                euid: 6,
                suid: 7,
                fsuid: 8,
                rgid: 9,
                egid: 10,
                sgid: 11,
                fsgid: 12,
                threads: 13,
                ram_kb: 14,
                swap_kb: 15,
                state: 82,
                cpu_user: 16,
                cpu_sys: 17,
                name: name.as_ptr().cast_mut(),
                command: command.as_ptr().cast_mut(),
            };
            let mut snapshot = limeade::Procs {
                ts_s: 100,
                ts_ms: 200,
                total: 1,
                procs: &mut process,
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
                store
                    .conn
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                        row.get(0)
                    })
                    .unwrap()
            };
            assert_eq!(count("proc_snapshots"), 4);
            assert_eq!(count("proc_attributes"), 3);
            assert_eq!(count("proc_batches"), 4);
            let rows = exported_rows(&store);
            assert_eq!(
                rows[0],
                vec![
                    "1",
                    "100",
                    "200",
                    "3",
                    "4",
                    "5",
                    "6",
                    "7",
                    "8",
                    "9",
                    "10",
                    "11",
                    "12",
                    "13",
                    "14",
                    "15",
                    "82",
                    "16",
                    "17",
                    "worker",
                    "/bin/worker --foreground"
                ]
            );
            let mut expected = rows[0].clone();
            expected[0] = "2".into();
            expected[14] = "42".into();
            assert_eq!(rows[1], expected);
            expected[0] = "3".into();
            expected[1] = "101".into();
            expected[6] = "99".into();
            expected[20] = "/bin/worker --reload".into();
            assert_eq!(rows[2], expected);
            expected[0] = "4".into();
            expected[19] = "".into();
            expected[20] = "".into();
            assert_eq!(rows[3], expected);
        }

        #[test]
        fn process_packet_keeps_each_sample_in_order() {
            let store = in_memory_store();
            let mut processes: [limeade::IndividualProc; 2] = unsafe { std::mem::zeroed() };
            processes[0].pid = 20;
            processes[1].pid = 10;
            let snapshot = limeade::Procs {
                ts_s: 123,
                ts_ms: 456,
                total: 2,
                procs: processes.as_mut_ptr(),
            };
            store.insert_procs(&snapshot).unwrap();
            let rows = exported_rows(&store);
            assert_eq!(
                (&rows[0][1], &rows[0][2], &rows[0][3]),
                (&"123".into(), &"456".into(), &"20".into())
            );
            assert_eq!(
                (&rows[1][1], &rows[1][2], &rows[1][3]),
                (&"123".into(), &"456".into(), &"10".into())
            );
            let attributes: i64 = store
                .conn
                .query_row("SELECT count(*) FROM proc_attributes", [], |row| row.get(0))
                .unwrap();
            assert_eq!(attributes, 1);
        }

        #[test]
        fn process_snapshot_failure_rolls_back() {
            let store = in_memory_store();
            store.conn.execute_batch("CREATE TRIGGER reject_sample BEFORE INSERT ON proc_batches BEGIN SELECT RAISE(FAIL, 'test failure'); END;").unwrap();
            let mut process: limeade::IndividualProc = unsafe { std::mem::zeroed() };
            let snapshot = limeade::Procs {
                ts_s: 1,
                ts_ms: 2,
                total: 1,
                procs: &mut process,
            };
            assert!(store.insert_procs(&snapshot).is_err());
            for table in ["proc_snapshots", "proc_attributes", "proc_batches"] {
                let count: i64 = store
                    .conn
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                        row.get(0)
                    })
                    .unwrap();
                assert_eq!(count, 0);
            }
        }

        #[test]
        fn export_handles_empty_snapshots_and_csv_escaping() {
            let store = in_memory_store();
            let command = CString::new("command,\"quoted\"\nnext").unwrap();
            let mut process: limeade::IndividualProc = unsafe { std::mem::zeroed() };
            process.command = command.as_ptr().cast_mut();
            let mut snapshot = limeade::Procs {
                ts_s: 1,
                ts_ms: 2,
                total: 0,
                procs: &mut process,
            };
            store.insert_procs(&snapshot).unwrap();
            snapshot.total = 1;
            store.insert_procs(&snapshot).unwrap();
            let rows = exported_rows(&store);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "1");
            assert_eq!(rows[0][20], "command,\"quoted\"\nnext");
        }

        #[test]
        fn export_rejects_corruption_and_missing_attributes() {
            let store = in_memory_store();
            let mut process: limeade::IndividualProc = unsafe { std::mem::zeroed() };
            let snapshot = limeade::Procs {
                ts_s: 1,
                ts_ms: 2,
                total: 1,
                procs: &mut process,
            };
            store.insert_procs(&snapshot).unwrap();
            let original: Vec<u8> = store
                .conn
                .query_row("SELECT data FROM proc_batches", [], |row| row.get(0))
                .unwrap();
            let mut damaged = original.clone();
            let last = damaged.len() - 1;
            damaged[last] ^= 0xff;
            store
                .conn
                .execute("UPDATE proc_batches SET data = ?1", [&damaged])
                .unwrap();
            assert!(export_procs(&store.conn, Vec::new()).is_err());
            store
                .conn
                .execute(
                    "UPDATE proc_batches SET data = ?1, sample_count = 0",
                    [&original],
                )
                .unwrap();
            assert!(export_procs(&store.conn, Vec::new()).is_err());
            store
                .conn
                .execute("UPDATE proc_batches SET sample_count = 1", [])
                .unwrap();
            store
                .conn
                .execute("DELETE FROM proc_attributes", [])
                .unwrap();
            assert!(export_procs(&store.conn, Vec::new()).is_err());
        }

        #[test]
        fn packet_inserts_match_schema() {
            let conn = Connection::open_in_memory().unwrap();
            conn.execute_batch(DB_INIT).unwrap();
            conn.execute(
                INS_INTRO,
                params![
                    1, 2, "host", "kernel", "distro", "user", "cpu", "vendor", 4, 8, 16, 123
                ],
            )
            .unwrap();
            conn.execute(INS_EVENTS, params![1, 2, 3, "open", "arg1", "arg2", 0])
                .unwrap();
            conn.execute(INS_PERF, params![1, 2, 3, 4, 5, 6, 7, 8])
                .unwrap();
            let syscall: String = conn
                .query_row("SELECT syscall FROM events", [], |row| row.get(0))
                .unwrap();
            assert_eq!(syscall, "open");
            let timestamp: (u32, u32) = conn
                .query_row("SELECT ts_s, ts_ms FROM perf", [], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })
                .unwrap();
            assert_eq!(timestamp, (1, 2));
        }

        #[test]
        fn initializes_database_schema() {
            let directory = std::env::temp_dir().join(format!(
                "limeade-receiver-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&directory).unwrap();
            let store =
                PacketSerializer::connect(&directory.to_string_lossy().into_owned()).unwrap();
            let entry = std::fs::read_dir(&directory)
                .unwrap()
                .next()
                .unwrap()
                .unwrap();
            assert_eq!(
                entry.metadata().unwrap().permissions().mode() & 0o777,
                0o660
            );
            drop(store);
            std::fs::remove_dir_all(directory).unwrap();
        }
    }
}
