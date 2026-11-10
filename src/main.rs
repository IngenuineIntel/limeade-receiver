// main.rs

use std::fs::{File, OpenOptions};
use std::{thread, time::Duration};

mod compat;
mod config;
mod log;
mod packet;
mod store;

use compat::*;
use config::GlobalConfig;

// due to the FFI I don't really have control over this, so
#[allow(nonstandard_style)]
fn main() {
  let config_file = File::open("/tmp/limeade-receiver.ini").ok();
  let config = GlobalConfig::new(config_file);
  let log_file = match OpenOptions::new().create(true).append(true).open(config.log_file()) {
    Ok(file) => file,
    Err(err) => {
      eprintln!("failed to open log file '{}': {err}", config.log_file());
      std::process::exit(1);
    }
  };
  log::log_init(Some(log_file));

  let mut store = match store::RecordStore::new(config.db_directory()) {
    Ok(store) => store,
    Err(err) => {
      err!("failed to open record directory '{}': {err}", config.db_directory());
      std::process::exit(1);
    }
  };

  dbg!("db_directory: {}", config.db_directory());
  dbg!("log_file:     {}", config.log_file());
  dbg!("IM HERE TOO");

  //thread::sleep(Duration::from_millis(1000));

  let mut ctx: limeade_context = unsafe { std::mem::zeroed() };
  let mut recvd: limeade_packet_data = unsafe { std::mem::zeroed() };

  let ret = unsafe { limeade_init(&mut ctx,
    (limeade_mode_LIMEADE_MODE_HOST_SSH |
     limeade_compression_mode_LIMEADE_MODE_NO_COMPRESSION) as i32) };

  if ret != limeade_error_LIMEADE_SUCCESS as i32 {
    err!("limeade_init failed: {ret}");
    return;
  }

  let ret = unsafe { limeade_connect(&mut ctx) };
  if ret != limeade_error_LIMEADE_SUCCESS as i32 {
    err!("limeade_connect failed: {ret}");
    return;
  }

  loop {
    let ret = unsafe { limeade_recv_wait(&mut ctx, &mut recvd, 2000) };

    if ret != limeade_error_LIMEADE_SUCCESS as i32 {
      if ret == limeade_error_LIMEADE_ERROR_NO_DATA as i32
        || ret == limeade_error_LIMEADE_ERROR_INTERRUPT as i32 {
        continue;
      }
      err!("limeade_recv_wait failed: {ret}");
      break;
    }

    let keep_running = match packet::handle_packet(&mut recvd, &mut store) {
      Ok(keep_running) => keep_running,
      Err(err) => {
        err!("{err}");
        true
      }
    };
    unsafe { limeade_release_recvd(&mut recvd) };
    if !keep_running {
      break;
    }
  }

  unsafe { limeade_destruct(&mut ctx) };
}

