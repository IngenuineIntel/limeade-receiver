// main.rs

use std::fs::File;
use std::thread;
use std::time::Duration;

mod compat;
mod config;
mod log;

use compat::*;
use config::GlobalConfig;

#[allow(nonstandard_style)]
fn handle_packet(pkt: limeade_packet_data) -> bool {

  match unsafe { pkt.__bindgen_anon_1.packet_type } as u32 {
    limeade_packet_LIMEADE_PACKET_INTRODUCTION => {
      // TODO
      info!("INTRO from client");
    }

    limeade_packet_LIMEADE_PACKET_ACKNOWLEDGE => {
      info!("ACK from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_EVENTS => {
      info!("EVENTS from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_PROC_GENERIC => {
      info!("PROC GENERIC from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_PROC_UPDATE => {
      info!("PROC UPDATE from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_PERF => {
      info!("PERF from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_CLOSE => {
      info!("CLOSE from client");
      // TODO
    }

    _ => return false
  }
  true
}

// due to the FFI I don't really have control over this, so
#[allow(nonstandard_style)]
fn main() {
  let config = GlobalConfig::new();
  let log_file = match File::create(config.log_file()) {
    Ok(file) => file,
    Err(err) => {
      eprintln!("failed to open log file '{}': {err}", config.log_file());
      std::process::exit(1);
    }
  };
  log::log_init(Some(log_file));

  dbg!("db_directory: {}", config.db_directory());
  dbg!("log_file:     {}", config.log_file());
  

  let mut ctx:   limeade_context     = unsafe { std::mem::zeroed() };
  let mut recvd: limeade_packet_data = unsafe { std::mem::zeroed() };

  let ret = unsafe { limeade_init(&mut ctx,
    (limeade_mode_LIMEADE_MODE_HOST_SSH |
     limeade_compression_mode_LIMEADE_MODE_NO_COMPRESSION) as i32) };

  if ret != limeade_error_LIMEADE_SUCCESS as i32 {
    err!("limeade_init failed: {ret}");
  }

  let ret = unsafe { limeade_connect(&mut ctx) };
  if ret != limeade_error_LIMEADE_SUCCESS as i32 {
    err!("limeade_connect failed: {ret}");
  }
    
  loop {
    let ret = unsafe { limeade_recv_wait(&mut ctx, &mut recvd, 2000) };

    if ret != limeade_error_LIMEADE_SUCCESS as i32 {
      err!("limeade_recv_wait failed: {ret}");
      thread::sleep(Duration::from_millis(100));
      continue;
    }

    handle_packet(recvd).then(|| {

      // TODO
    });

    unsafe { limeade_release_recvd(&mut recvd) };
  }

}

