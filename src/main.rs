// main.rs

const VERSION: &'static str = "v0.1";
const MAINTAINER: &'static str = "IngenuineIntel";
const LICENSE: &'static str = "AGPLv3";
const COPYRIGHT: &'static str = "Copyright (c) 2026 Roan Rothrock";

use std::fs::{File, OpenOptions};
use std::{thread, time::Duration};

mod compat;
mod config;
mod db;
mod log;

use compat::limeade;
use config::GlobalConfig;
use db::db::PacketSerializer;

fn main() {
    /* Initializes configuration for the program */
    let config_file = File::open("/etc/limeade/limeade-receiver.ini").ok();
    let config = GlobalConfig::new(config_file);
    log::log_init(
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(config.log_file())
        {
            Ok(fd) => Some(fd),
            Err(_) => Some(File::open("/tmp/limeade-receiver.log").expect("Failed to start log!")),
        },
    );

    info!("Limeade Receiver {VERSION}");
    info!("Courtesy of {MAINTAINER}");
    info!("{COPYRIGHT}");
    warn!("Licensed under the {LICENSE} license");

    let store = match PacketSerializer::connect(config.db_directory()) {
        Ok(store) => store,
        Err(error) => {
            err!("Failed to start database: {error}");
            std::process::exit(1);
        }
    };

    let mut ctx = match limeade::init() {
        Ok(ctx) => ctx,
        Err(error) => {
            err!("{error}");
            std::process::exit(1);
        }
    };

    thread::sleep(Duration::from_millis(1000));

    loop {
        match limeade::recv_wait(&mut ctx, 2000) {
            Ok(mut recvd) => {
                match store.handle_packet(&mut recvd) {
                    Ok(()) => {}
                    Err(error) => {
                        err!("Failed to serialize packet: {error}");
                    }
                };
                limeade::release_recvd(&mut recvd);
            }
            Err(limeade::Error { code, message }) => {
                if code != limeade::ERROR_NO_DATA as u32 {
                    err!("Failed to receive packet: {message}");
                    break;
                }
            }
        }
    }

    limeade::destruct(&mut ctx);

}

/*
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

*/
