use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

// https://instagram-engineering.com/sharding-ids-at-instagram-1cf5a71e5a5c

// number of milliseconds since UTC epoch time
const JANUARY_1ST_2024_AS_MS: u64 = 1704067200541;
const JANUARY_1ST_2024_AS_DURATION: Duration = Duration::from_millis(JANUARY_1ST_2024_AS_MS);

const TICKET_BITS: u64 = 10;
const TICKET_MAX_COUNT: u64 = 2 ^ 10;
const DECOMPOSE_TICKET_BITS: u64 = (1 << TICKET_BITS) - 1;
const LOGICAL_SHARD_BITS: u64 = 13;
const LOGICAL_SHARD_MAX_COUNT: u64 = 2 ^ 13;
const DECOMPOSE_LOGICAL_SHARD_BITS: u64 = ((1 << LOGICAL_SHARD_BITS) - 1) << TICKET_BITS;
const TIME_BITS: u64 = 41;

pub struct Snowprint {
    logical_volume_id: u64,
    sequence_id: u64,
}

impl Snowprint {
    pub fn new() -> Snowprint {
        Snowprint {
            logical_volume_id: 0,
            sequence_id: 0,
        }
    }

    pub fn get_snowprint(&mut self) -> u64 {
        let JANUARY_1ST_2024_AS_SYSTEM_TIME: SystemTime = UNIX_EPOCH + JANUARY_1ST_2024_AS_DURATION;

        let now = SystemTime::now();
        let since = match now.duration_since(JANUARY_1ST_2024_AS_SYSTEM_TIME) {
            Ok(duration) => duration,
            _ => Duration::from_millis(0),
        };

        compose_snowprint(
            since.as_millis() as u64,
            self.logical_volume_id,
            self.sequence_id,
        )
    }
}
//1714717502501
//1716421518012

// at it's core this is a snowprint
pub fn compose_snowprint(ms_timestamp: u64, logical_id: u64, ticket_id: u64) -> u64 {
    ms_timestamp << (LOGICAL_SHARD_BITS + TICKET_BITS) | logical_id << TICKET_BITS | ticket_id
}

pub fn decompose_snowprint(snowprint: u64) -> (u64, u64, u64) {
    let time = snowprint >> (LOGICAL_SHARD_BITS + TICKET_BITS);
    let logical_id = (snowprint & DECOMPOSE_LOGICAL_SHARD_BITS) >> TICKET_BITS;
    let ticket_id = snowprint & DECOMPOSE_TICKET_BITS;

    (time, logical_id, ticket_id)
}
