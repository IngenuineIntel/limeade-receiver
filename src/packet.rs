use std::ffi::CStr;
use std::fmt::Write as _;
use std::os::raw::c_char;
use std::slice;

use crate::compat::*;
use crate::store::RecordStore;
use crate::{info, timestamp};

fn c_string(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() }
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                write!(output, "\\u{:04x}", character as u32).unwrap();
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

fn json_proc(proc: &limeade_indiv_proc) -> String {
    format!(
        "{{\"pid\":{},\"ppid\":{},\"ruid\":{},\"euid\":{},\"suid\":{},\"fsuid\":{},\"rgid\":{},\"egid\":{},\"sgid\":{},\"fsgid\":{},\"threads\":{},\"cpu_user\":{},\"cpu_sys\":{},\"ram_kb\":{},\"swap_kb\":{},\"state\":{},\"name\":{},\"command\":{}}}",
        proc.pid,
        proc.ppid,
        proc.ruid,
        proc.euid,
        proc.suid,
        proc.fsuid,
        proc.rgid,
        proc.egid,
        proc.sgid,
        proc.fsgid,
        proc.threads,
        proc.cpu_user,
        proc.cpu_sys,
        proc.ram_kb,
        proc.swap_kb,
        json_string(&(proc.state as u8 as char).to_string()),
        json_string(&c_string(proc.name)),
        json_string(&c_string(proc.command)),
    )
}

fn check_parse(kind: &str, result: i32) -> Result<(), String> {
    if result == limeade_error_LIMEADE_SUCCESS as i32 {
        Ok(())
    } else {
        Err(format!("failed to parse {kind} packet: error {result}"))
    }
}

fn c_slice<'a, T>(pointer: *const T, length: usize) -> &'a [T] {
    if length == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(pointer, length) }
    }
}

pub fn handle_packet(
    packet: &mut limeade_packet_data,
    store: &mut RecordStore,
) -> Result<bool, String> {
    let packet_type = unsafe { packet.__bindgen_anon_1.packet_type } as u32;
    let record = match packet_type {
        value if value == limeade_packet_LIMEADE_PACKET_INTRODUCTION => {
            let mut introduction: limeade_introduction = unsafe { std::mem::zeroed() };
            let result = unsafe { limeade_parse_intro(&mut introduction, packet) };
            check_parse("introduction", result)?;
            let hostname = c_string(introduction.hostname);
            let record = format!(
                "{{\"type\":\"introduction\",\"hostname\":{},\"kernel\":{},\"distro\":{},\"user\":{},\"processor\":{},\"vendor\":{},\"cores\":{},\"ram_mbs\":{},\"swap_mbs\":{}}}",
                json_string(&hostname),
                json_string(&c_string(introduction.kernelver)),
                json_string(&c_string(introduction.distro)),
                json_string(&c_string(introduction.origin_user)),
                json_string(&c_string(introduction.processor)),
                json_string(&c_string(introduction.vendor)),
                introduction.nr_cores,
                introduction.ram_mbs,
                introduction.swap_mbs,
            );
            unsafe { limeade_release_intro(&mut introduction) };
            store
                .set_client(&hostname)
                .map_err(|error| format!("failed to open client record file: {error}"))?;
            record
        }
        value if value == limeade_packet_LIMEADE_PACKET_ACKNOWLEDGE => {
            let mut ack: limeade_acknowledge = unsafe { std::mem::zeroed() };
            let result = unsafe { limeade_parse_ack(&mut ack, packet) };
            check_parse("acknowledgment", result)?;
            let record = format!(
                "{{\"type\":\"acknowledgment\",\"send_ts_s\":{},\"send_ts_ms\":{},\"recv_ts_s\":{},\"recv_ts_ms\":{}}}",
                ack.send_ts_s, ack.send_ts_ms, ack.recv_ts_s, ack.recv_ts_ms
            );
            unsafe { limeade_release_ack(&mut ack) };
            record
        }
        value if value == limeade_packet_LIMEADE_PACKET_EVENTS => {
            let mut events: limeade_events = unsafe { std::mem::zeroed() };
            let result = unsafe { limeade_parse_events(&mut events, packet) };
            check_parse("events", result)?;
            let items = c_slice(events.events, events.nr_events as usize);
            let mut record = String::from("{\"type\":\"events\",\"items\":[");
            for (index, event) in items.iter().enumerate() {
                if index > 0 {
                    record.push(',');
                }
                write!(
                    record,
                    "{{\"ts_s\":{},\"ts_ms\":{},\"pid\":{},\"syscall\":{},\"arg1\":{},\"arg2\":{},\"retval\":{}}}",
                    event.ts_s,
                    event.ts_ms,
                    event.pid,
                    json_string(&c_string(event.syscall)),
                    json_string(&c_string(event.arg1)),
                    json_string(&c_string(event.arg2)),
                    event.retval,
                )
                .unwrap();
            }
            record.push_str("]}");
            unsafe { limeade_release_events(&mut events) };
            record
        }
        value if value == limeade_packet_LIMEADE_PACKET_PROC_GENERIC => {
            let mut data: limeade_proc_generic = unsafe { std::mem::zeroed() };
            let result = unsafe { limeade_parse_proc_generic(&mut data, packet) };
            check_parse("generic process", result)?;
            let items = c_slice(data.procs, data.total as usize);
            let mut record = format!(
                "{{\"type\":\"proc_generic\",\"ts_s\":{},\"ts_ms\":{},\"items\":[",
                data.ts_s, data.ts_ms
            );
            for (index, proc) in items.iter().enumerate() {
                if index > 0 {
                    record.push(',');
                }
                record.push_str(&json_proc(proc));
            }
            record.push_str("]}");
            unsafe { limeade_release_proc_generic(&mut data) };
            record
        }
        value if value == limeade_packet_LIMEADE_PACKET_PROC_UPDATE => {
            let mut data: limeade_proc_update = unsafe { std::mem::zeroed() };
            let result = unsafe { limeade_parse_proc_update(&mut data, packet) };
            check_parse("process update", result)?;
            let died = c_slice(data.died, data.total_died as usize);
            let altered = c_slice(data.altered, data.total_altered as usize);
            let mut record = format!(
                "{{\"type\":\"proc_update\",\"ts_s\":{},\"ts_ms\":{},\"died\":[",
                data.ts_s, data.ts_ms
            );
            for (index, pid) in died.iter().enumerate() {
                if index > 0 {
                    record.push(',');
                }
                write!(record, "{pid}").unwrap();
            }
            record.push_str("],\"altered\":[");
            for (index, proc) in altered.iter().enumerate() {
                if index > 0 {
                    record.push(',');
                }
                record.push_str(&json_proc(proc));
            }
            record.push_str("]}");
            unsafe { limeade_release_proc_update(&mut data) };
            record
        }
        value if value == limeade_packet_LIMEADE_PACKET_PERF => {
            let mut perf: limeade_perf = unsafe { std::mem::zeroed() };
            let result = unsafe { limeade_parse_perf(&mut perf, packet) };
            check_parse("performance", result)?;
            format!(
                "{{\"type\":\"perf\",\"cpu_user\":{},\"cpu_sys\":{},\"mem_total_kb\":{},\"mem_free_kb\":{},\"mem_available_kb\":{},\"mem_cached_kb\":{}}}",
                perf.perc_cpu_user,
                perf.perc_cpu_sys,
                perf.mem_total_kb,
                perf.mem_free_kb,
                perf.mem_available_kb,
                perf.mem_cached_kb,
            )
        }
        value if value == limeade_packet_LIMEADE_PACKET_CLOSE => {
            let mut close: limeade_close = unsafe { std::mem::zeroed() };
            let result = unsafe { limeade_parse_close(&mut close, packet) };
            check_parse("close", result)?;
            let explanation = c_string(close.explanation);
            let record = format!(
                "{{\"type\":\"close\",\"explanation\":{}}}",
                json_string(&explanation)
            );
            unsafe { limeade_release_close(&mut close) };
            store
                .write_record(&record)
                .map_err(|error| format!("failed to write client record: {error}"))?;
            info!("CLOSE from client: {explanation}");
            return Ok(false);
        }
        _ => return Err(format!("unrecognized packet type {packet_type}")),
    };

    store
        .write_record(&record)
        .map_err(|error| format!("failed to write client record: {error}"))?;
    info!("recorded {} packet", packet_type);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_strings_escape_control_characters() {
        assert_eq!(json_string("a\n\"b"), "\"a\\n\\\"b\"");
    }
}