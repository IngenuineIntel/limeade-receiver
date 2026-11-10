// db.rs

mod db {
    use std::path::PathBuf;

    use chrono::Local;
    use rusqlite::{Connection, Result};

    use crate::compat::limeade;

    const DB_INIT: &str    = include_str!("sql/init.sql");
    const INS_INTRO: &str  = include_str!("sql/ins_intro.sql");
    const INS_EVENTS: &str = include_str!("sql/ins_events.sql");

    fn resolve_db_path(dir: &String) -> PathBuf {
        PathBuf::from(dir).join(Local::now().format("%m-%d-%Y_%H-%M-%S-%.3f.db").to_string())
    }

    struct PacketSerializer {
        conn: rusqlite::Connection,
    }

    impl PacketSerializer {
        fn connect(mut self, db_directory: &String) -> Result<(), Box<dyn std::error::Error>> {
            self.conn = Connection::open(resolve_db_path(db_directory))?;
            self.conn.execute(DB_INIT, ())?;

            Ok(())
        }


        fn handle_packet(self, pkt: &mut limeade::PacketData) -> Result<(), Box<dyn std::error::Error>> {
            match limeade::pkt_type(pkt) {
                limeade::TYPE_INTRO  => {
                    let intro = limeade::parse_intro(pkt)?;
                    // TODO shove intro attributes into SQL
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
