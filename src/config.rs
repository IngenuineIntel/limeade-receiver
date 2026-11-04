// config.rs
// config file management

use std::env;
use std::fs::File;
use std::io::Read;

use configparser::ini::Ini;

const CONFIG_FILE: &'static str = "/etc/limeade/limeade-receiver.ini";

pub struct GlobalConfig {
  db_directory: String,
  log_file: String,
}

const CONFIG_DFLT_DB_DIRECTORY: &'static str = "/etc/limeade/history";
const CONFIG_DFLT_LOG_FILE:     &'static str = "/etc/limeade/history/limeade-receiver.log";

impl GlobalConfig {

  fn populate_from_config(mut self, file: Option<File>) -> Self {
    let mut config = Ini::new();

    let mut data = String::new();
    let Some(mut file) = file.or_else(|| File::open(CONFIG_FILE).ok()) else {
      return self;
    };
    if file.read_to_string(&mut data).is_err() {
      return self;
    }
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

  pub fn new(file: Option<File>) -> Self {
    GlobalConfig {
      db_directory: String::new(),
      log_file: String::new(),
    }
    .populate_from_config(file)
    .populate_from_dflt()
  }

  pub fn db_directory(&self) -> &str {
    &self.db_directory
  }

  pub fn log_file(&self) -> &str {
    &self.log_file
  }

  pub fn reload(mut self) -> Self {
    self.populate_from_config(None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_config() {
    let c = GlobalConfig::new(None);
    println!("db_directory from conf: {0}", c.db_directory);
    println!("logfile from conf: {0}", c.log_file);
  }

  #[test]
  fn test_output_defaults_use_history_directory() {
    let c = GlobalConfig {
      db_directory: String::new(),
      log_file: String::new(),
    }
    .populate_from_dflt();

    assert_eq!(c.db_directory(), "/etc/limeade/history");
    assert_eq!(c.log_file(), "/etc/limeade/history/limeade-receiver.log");
  }
}

