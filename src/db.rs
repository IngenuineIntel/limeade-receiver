// db.rs

pub mod db {
    use std::ffi::CStr;
    use std::os::raw::c_char;
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
            let conn = Connection::open(resolve_db_path(db_directory))?;
            conn.execute(DB_INIT, ())?;

            Ok(Self { conn })
        }

        pub fn handle_packet(
            &self,
            pkt: &mut limeade::PacketData,
        ) -> Result<(), Box<dyn std::error::Error>> {
            match limeade::pkt_type(pkt) {
                limeade::TYPE_INTRO => {
                    let intro = limeade::parse_intro(pkt)?;

                    self.conn.execute(
                        INS_INTRO,
                        params![
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
                    for event in c_arr!(events.events, events.nr_events) {
                        self.conn.execute(
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
                }
                limeade::TYPE_PROC => {
                    let procs = limeade::parse_proc(pkt)?;
                    if procs.procs.is_null() {
                        return Err("procs array is NULL".into());
                    }
                    for proc in c_arr!(procs.procs, procs.total) {
                        self.conn.execute(
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
}
