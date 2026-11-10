// db.rs

pub mod db {
    use std::ffi::CStr;
    use std::os::raw::c_char;
    use std::path::PathBuf;

    use chrono::Local;
    use rusqlite::{Connection, Result};

    use crate::compat::limeade;

    const DB_INIT: &str    = include_str!("sql/init.sql");
    const INS_INTRO: &str  = include_str!("sql/ins_intro.sql");
    const INS_EVENTS: &str = include_str!("sql/ins_events.sql");
    const INS_PROC: &str   = include_str!("sql/ins_proc.sql");
    const INS_PERF: &str   = include_str!("sql/ins_perf.sql");

    fn c_string(value: *const c_char) -> String {
        if value.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() }
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


        pub fn handle_packet(&self, pkt: &mut limeade::PacketData) -> Result<(), Box<dyn std::error::Error>> {
            match limeade::pkt_type(pkt) {
                limeade::TYPE_INTRO  => {
                    let intro = limeade::parse_intro(pkt)?;

                    self.conn.execute(INS_INTRO, (
                        c_string(intro.hostname),
                        c_string(intro.kernelver),
                        c_string(intro.distro),
                        c_string(intro.origin_user),
                        c_string(intro.processor),
                        c_string(intro.vendor),
                        intro.nr_cores,
                        intro.ram_mbs,
                        intro.swap_mbs
                    ))?;
                },
                limeade::TYPE_EVENTS => {
                    let events = limeade::parse_events(pkt)?;
                    // TODO shove event attributes into SQL
                },
                limeade::TYPE_PROC   => {
                    let procs = limeade::parse_proc(pkt)?;
                    // TODO shove proc attributes into SQL
                },
                limeade::TYPE_PERF   => {
                    let perf = limeade::parse_perf(pkt)?;
                    // TODO shove perf attributes into SQL
                },
                _ => {},
            }
          Ok(())
        }
    }
}
