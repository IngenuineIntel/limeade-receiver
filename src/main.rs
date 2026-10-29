// main.rs

use std::thread;
use std::time::Duration;

mod compat;
mod config;
use compat::*;



#[allow(nonstandard_style)]
fn handle_packet(pkt: limeade_packet_data) -> bool {

  match unsafe { pkt.__bindgen_anon_1.packet_type } as u32 {
    limeade_packet_LIMEADE_PACKET_INTRODUCTION => {
      // TODO
      println!("INTRO from client");
    }

    limeade_packet_LIMEADE_PACKET_ACKNOWLEDGE => {
      println!("ACK from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_EVENTS => {
      println!("EVENTS from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_PROC_GENERIC => {
      println!("PROC GENERIC from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_PROC_UPDATE => {
      println!("PROC UPDATE from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_PERF => {
      println!("PERF from client");
      // TODO
    }

    limeade_packet_LIMEADE_PACKET_CLOSE => {
      println!("CLOSE from client");
      // TODO
    }

    _ => return false
  }
  true
}

// due to the FFI I don't really have control over this, so
#[allow(nonstandard_style)]
fn main() {
  let mut ctx:   limeade_context     = unsafe { std::mem::zeroed() };
  let mut recvd: limeade_packet_data = unsafe { std::mem::zeroed() };

  let ret = unsafe { limeade_init(&mut ctx,
    (limeade_mode_LIMEADE_MODE_HOST_SSH |
     limeade_compression_mode_LIMEADE_MODE_NO_COMPRESSION) as i32) };

  if ret != limeade_error_LIMEADE_SUCCESS as i32 {
    eprintln!("limeade_init failed: {ret}");
  }

  let ret = unsafe { limeade_connect(&mut ctx) };
  if ret != limeade_error_LIMEADE_SUCCESS as i32 {
    eprintln!("limeade_connect failed: {ret}");
  }
    
  loop {
    let ret = unsafe { limeade_recv_wait(&mut ctx, &mut recvd, 2000) };

    if ret != limeade_error_LIMEADE_SUCCESS as i32 {
      eprintln!("limeade_recv_wait failed: {ret}");
      thread::sleep(Duration::from_millis(100));
      continue;
    }

    handle_packet(recvd).then(|| {

      // TODO
    });

    unsafe { limeade_release_recvd(&mut recvd) };
  }

}

