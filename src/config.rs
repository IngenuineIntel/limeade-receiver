// config.rs
// config file management

use std::env;
use std::fs;

use configparser::ini::Ini;

const CONFIG_FILE: &'static str = "/etc/limeade/limeade-receiver.ini";

struct GlobalConfig {
  db_directory: String,
  log_file: String,
}

const CONFIG_DFLT_DB_DIRECTORY: &'static str = "./limeade-receiver-db";
const CONFIG_DFLT_LOG_FILE:     &'static str = "./limeade-logfile.log";

impl GlobalConfig {

  fn populate_from_config(mut self) -> Self {
    let mut config = Ini::new();

    let Ok(data) = fs::read_to_string(CONFIG_FILE) else {
      return self;
    };
    if let Err(_) = config.read(data) {
      return self
    }
    if let Some(db_dir) = config.get("limeade", "db_directory") {
      self.db_directory = db_dir;
    }
    if let Some(logf) = config.get("limeade", "log_file") {
      self.log_file = logf;
    }
    self
  }

  fn populate_from_dflt(mut self) -> Self {
    if self.db_directory.is_empty() {
      self.db_directory = CONFIG_DFLT_DB_DIRECTORY.to_string();
    }
    if self.log_file.is_empty() {
      self.log_file = CONFIG_DFLT_LOG_FILE.to_string();
    }
    self
  }

  pub fn new() -> Self {
    GlobalConfig {
      db_directory: String::new(),
      log_file: String::new(),
    }
    .populate_from_config()
    .populate_from_dflt()
  }

  pub fn reload(mut self) -> Self {
    self.populate_from_config()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_config() {
    let c = GlobalConfig::new();
    println!("db_directory from conf: {0}", c.db_directory);
    println!("logfile from conf: {0}", c.log_file);
  }
}

