use std::fs::File;
use std::io::{self, Write};
use std::process;
use std::sync::Mutex;

use chrono::{Local, Utc};

pub(crate) static LOGGING_FD: Mutex<Option<Box<dyn Write + Send>>> = Mutex::new(None);
pub(crate) static PID: Mutex<u32> = Mutex::new(0);

pub fn log_init(stream: Option<File>) {
    *PID.lock().unwrap() = process::id();
    let output: Box<dyn Write + Send> = match stream {
        Some(file) => Box::new(file),
        None => Box::new(io::stderr()),
    };
    *LOGGING_FD.lock().unwrap() = Some(output);
}

// COLORS
// unfortunately, these logs go into a file, so colors aren't super useful
/*
pub(crate) const ANSI_BLUE: &str = "\x1b[34m";
pub(crate) const ANSI_CYAN: &str = "\x1b[36m";
pub(crate) const ANSI_GREEN: &str = "\x1b[32m";
pub(crate) const ANSI_RED: &str = "\x1b[31m";
pub(crate) const ANSI_YELLOW: &str = "\x1b[33m";
pub(crate) const ANSI_RESET: &str = "\x1b[39m";
*/

pub(crate) const ANSI_BLUE: &str = "";
pub(crate) const ANSI_CYAN: &str = "";
pub(crate) const ANSI_GREEN: &str = "";
pub(crate) const ANSI_RED: &str = "";
pub(crate) const ANSI_YELLOW: &str = "";
pub(crate) const ANSI_RESET: &str = "";

pub(crate) fn get_time() -> String {
    Utc::now()
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

#[macro_export]
macro_rules! log_out {
    () => {
        crate::log::LOGGING_FD
            .lock()
            .unwrap()
            .get_or_insert_with(|| Box::new(std::io::stdout()))
    };
}

#[macro_export]
macro_rules! timestamp {
    () => {
        format_args!(
            "[{}{}{}]",
            crate::log::ANSI_BLUE,
            crate::log::get_time(),
            crate::log::ANSI_RESET
        )
    };
}

#[macro_export]
macro_rules! pid_sect {
    () => {
        format_args!(
            "[{}{}{}]",
            crate::log::ANSI_CYAN,
            std::process::id(),
            crate::log::ANSI_RESET
        )
    };
}

#[macro_export]
macro_rules! dbg {
  ($($args:tt)*) => {
    let _ = writeln!(crate::log_out!(), "{}{}[{}DBUG{}]: {}", timestamp!(), pid_sect!(), crate::log::ANSI_CYAN, crate::log::ANSI_RESET, format_args!($($args)*));
  }
}

#[macro_export]
macro_rules! info {
  ($($args:tt)*) => {
    let _ = writeln!(crate::log_out!(), "{}{}[{}INFO{}]: {}", timestamp!(), pid_sect!(), crate::log::ANSI_GREEN, crate::log::ANSI_RESET, format_args!($($args)*));
  }
}

#[macro_export]
macro_rules! warn {
  ($($args:tt)*) => {
    let _ = writeln!(crate::log_out!(), "{}{}[{}WARN{}]: {}", timestamp!(), pid_sect!(), crate::log::ANSI_YELLOW, crate::log::ANSI_RESET, format_args!($($args)*));
  }
}

#[macro_export]
macro_rules! crit {
  ($($args:tt)*) => {
    let _ = writeln!(crate::log_out!(), "{}{}[{}CRIT{}]: {}", timestamp!(), pid_sect!(), crate::log::ANSI_RED, crate::log::ANSI_RESET, format_args!($($args)*));
  }
}

#[macro_export]
macro_rules! err {
  ($($args:tt)*) => {
    let _ = writeln!(crate::log_out!(), "{}{}[{}ERRR{}]: {}", timestamp!(), pid_sect!(), crate::log::ANSI_RED, crate::log::ANSI_RESET, format_args!($($args)*));
  }
}

// TODO build test
