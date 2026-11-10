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


        fn handle_packet(self, pkt: limeade::PacketData) -> Result<(), Box<dyn std::error::Error>> {
          unsafe {
              match pkt.__bindgen_anon_1.packet_type {
                limeade::TYPE_INTRO  => {},
                limeade::TYPE_EVENTS => {},
                limeade::TYPE_PROC   => {},
                limeade::TYPE_PERF   => {},
                _ => {},
              }
          }
          Ok(())
        }
    }
}
