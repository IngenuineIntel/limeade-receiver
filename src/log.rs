use std::fs::File;
use std::io::{self, Write};
use std::sync::Mutex;

use chrono::{Local, Utc};

static LOGGING_FD: Mutex<Option<Box<dyn Write + Send>>> = Mutex::new(None);

pub fn log_init(stream: Option<File>) {
    let output: Box<dyn Write + Send> = match stream {
        Some(file) => Box::new(file),
        None => Box::new(io::stdout()),
    };
    *LOGGING_FD.lock().unwrap() = Some(output);
}

const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_RESET: &str = "\x1b[39m";

fn get_time() -> String {
    Utc::now().with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
}

macro_rules! log_out {
  () => {
    LOGGING_FD.lock()
              .unwrap()
              .get_or_insert_with(|| Box::new(io::stdout()))
  }
}

macro_rules! timestamp {
  () => { format_args!("[{}{}{}]", ANSI_BLUE, get_time(), ANSI_RESET) }
}

pub(crate) fn write_log(level: &str, message: std::fmt::Arguments<'_>) {
    let color = match level {
        "DBUG" => ANSI_CYAN,
        "INFO" => ANSI_GREEN,
        "WARN" => ANSI_YELLOW,
        "CRIT" | "ERRR" => ANSI_RED,
        _ => ANSI_RESET,
    };
    let mut output = LOGGING_FD.lock().unwrap();
    writeln!(output.get_or_insert_with(|| Box::new(io::stdout())), "[{}{}{}][{}{}{}]: {message}", ANSI_BLUE, get_time(), ANSI_RESET, color, level, ANSI_RESET).unwrap();
}

pub(crate) fn dbg(level: &str, message: std::fmt::Arguments<'_>) {
  writeln!(log_out!(), "{}[{}DBUG{}]: {}", timestamp!(), ANSI_CYAN, ANSI_RESET, message);
}

pub(crate) fn info(level: &str, message: std::fmt::Arguments<'_>) {
  writeln!(log_out!(), "{}[{}INFO{}]: {}", timestamp!(), ANSI_GREEN, ANSI_RESET, message);
}

pub(crate) fn warn(level: &str, message: std::fmt::Arguments<'_>) {
  writeln!(log_out!(), "{}[{}WARN{}]: {}", timestamp!(), ANSI_YELLOW, ANSI_RESET, message);
}

pub(crate) fn crit(level: &str, message: std::fmt::Arguments<'_>) {
  writeln!(log_out!(), "{}[{}CRIT{}]: {}", timestamp!(), ANSI_RED, ANSI_RESET, message);
}

pub(crate) fn err(level: &str, message: std::fmt::Arguments<'_>) {
  writeln!(log_out!(), "{}[{}ERRR{}]: {}", timestamp!(), ANSI_RED, ANSI_RESET, message);
}
