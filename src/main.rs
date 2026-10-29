// main.rs

use std::thread;
use std::time::Duration;

mod compat;
use compat::*;

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

    let pkt_type = unsafe { recvd.__bindgen_anon_1.packet_type } as u32;

    match pkt_type {
      limeade_packet_LIMEADE_PACKET_INTRODUCTION => {
        // TODO
        println!("introduction packet received");
      }
      
      limeade_packet_LIMEADE_PACKET_ACKNOWLEDGE => {
        // TODO
        println!("acknowledgement packet received");
      }

      limeade_packet_LIMEADE_PACKET_EVENTS => {
        // TODO
        println!("events packet received");
      }

      limeade_packet_LIMEADE_PACKET_PROC_GENERIC => {
        // TODO
        println!("generic process packet received");
      }

      limeade_packet_LIMEADE_PACKET_PROC_UPDATE => {
        // TODO
        println!("process update packet received");
      }

      limeade_packet_LIMEADE_PACKET_PERF => {
        // TODO
        println!("performance packet received");
      }

      limeade_packet_LIMEADE_PACKET_CLOSE => {
        // TODO
        println!("close packet received");
      }

      _ => eprintln!("unknown packet type: {pkt_type}"),
    }

    unsafe { limeade_release_recvd(&mut recvd) };
  }

}

