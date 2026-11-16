use std::io::{self, Write};

const MAGIC: &[u8; 4] = b"LPS1";
const HEADER_SIZE: usize = 6;
const RECORD_SIZE: usize = 27;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    pub attribute_id: i64,
    pub pid: i32,
    pub threads: u32,
    pub ram_kb: u32,
    pub swap_kb: u32,
    pub state: i8,
    pub cpu_user: i8,
    pub cpu_sys: i8,
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub fn encode(samples: &[Sample]) -> io::Result<Vec<u8>> {
    let count = u16::try_from(samples.len()).map_err(|_| invalid("too many process samples"))?;
    let mut raw = Vec::with_capacity(HEADER_SIZE + samples.len() * RECORD_SIZE);
    raw.extend_from_slice(MAGIC);
    raw.extend_from_slice(&count.to_le_bytes());
    for sample in samples {
        if sample.attribute_id <= 0 {
            return Err(invalid("invalid process attribute ID"));
        }
        raw.extend_from_slice(&sample.attribute_id.to_le_bytes());
        raw.extend_from_slice(&sample.pid.to_le_bytes());
        raw.extend_from_slice(&sample.threads.to_le_bytes());
        raw.extend_from_slice(&sample.ram_kb.to_le_bytes());
        raw.extend_from_slice(&sample.swap_kb.to_le_bytes());
        raw.extend_from_slice(&[sample.state as u8, sample.cpu_user as u8, sample.cpu_sys as u8]);
    }
    let mut encoder = zstd::stream::Encoder::new(Vec::new(), 3)?;
    encoder.include_checksum(true)?;
    encoder.write_all(&raw)?;
    encoder.finish()
}

pub fn decode(compressed: &[u8]) -> io::Result<Vec<Sample>> {
    let raw = zstd::bulk::decompress(compressed, HEADER_SIZE + u16::MAX as usize * RECORD_SIZE)?;
    if raw.len() < HEADER_SIZE || &raw[..4] != MAGIC {
        return Err(invalid("invalid process batch format"));
    }
    let count = u16::from_le_bytes([raw[4], raw[5]]) as usize;
    if raw.len() != HEADER_SIZE + count * RECORD_SIZE {
        return Err(invalid("invalid process batch length"));
    }
    let mut samples = Vec::with_capacity(count);
    for record in raw[HEADER_SIZE..].chunks_exact(RECORD_SIZE) {
        let attribute_id = i64::from_le_bytes(record[0..8].try_into().unwrap());
        if attribute_id <= 0 {
            return Err(invalid("invalid process attribute ID"));
        }
        samples.push(Sample {
            attribute_id,
            pid: i32::from_le_bytes(record[8..12].try_into().unwrap()),
            threads: u32::from_le_bytes(record[12..16].try_into().unwrap()),
            ram_kb: u32::from_le_bytes(record[16..20].try_into().unwrap()),
            swap_kb: u32::from_le_bytes(record[20..24].try_into().unwrap()),
            state: record[24] as i8,
            cpu_user: record[25] as i8,
            cpu_sys: record[26] as i8,
        });
    }
    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_full_width_values_and_empty_batches() {
        let samples = vec![Sample {
            attribute_id: i64::MAX, pid: i32::MIN, threads: u32::MAX,
            ram_kb: u32::MAX, swap_kb: u32::MAX, state: i8::MIN,
            cpu_user: i8::MIN, cpu_sys: i8::MAX,
        }];
        assert_eq!(decode(&encode(&samples).unwrap()).unwrap(), samples);
        assert!(decode(&encode(&[]).unwrap()).unwrap().is_empty());
    }

    #[test]
    fn rejects_corrupted_and_truncated_batches() {
        let mut batch = encode(&[]).unwrap();
        let last = batch.len() - 1;
        batch[last] ^= 0xff;
        assert!(decode(&batch).is_err());
        assert!(decode(&batch[..last]).is_err());
    }

    #[test]
    fn rejects_invalid_lengths_and_attribute_ids() {
        let invalid_header = zstd::bulk::compress(b"LPS1\x01\x00", 3).unwrap();
        assert!(decode(&invalid_header).is_err());
        let invalid_version = zstd::bulk::compress(b"LPS2\x00\x00", 3).unwrap();
        assert!(decode(&invalid_version).is_err());
        let sample = Sample { attribute_id: 0, pid: 0, threads: 0, ram_kb: 0, swap_kb: 0, state: 0, cpu_user: 0, cpu_sys: 0 };
        assert!(encode(&[sample]).is_err());
    }
}
