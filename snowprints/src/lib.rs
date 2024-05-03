use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

// https://instagram-engineering.com/sharding-ids-at-instagram-1cf5a71e5a5c

// number of milliseconds since UTC epoch time
const JANUARY_1ST_2024_AS_MS: u64 = 1704070800;

const TICKET_BITS: u64 = 10;
const DECOMPOSE_TICKET_BITS: u64 = (1 << TICKET_BITS) - 1;
const LOGICAL_SHARD_BITS: u64 = 13;
const DECOMPOSE_LOGICAL_SHARD_BITS: u64 = ((1 << LOGICAL_SHARD_BITS) - 1) << TICKET_BITS;
const TIME_BITS: u64 = 41;

// at it's core this is a snowprint
pub fn compose_snowprint(ms_timestamp: u64, logical_id: u64, ticket_id: u64) -> u64 {
    let origin =
        ms_timestamp << (LOGICAL_SHARD_BITS + TICKET_BITS) | logical_id << TICKET_BITS | ticket_id;

    origin
}

pub fn decompose_snowprint(snowprint: u64) -> (u64, u64, u64) {
    let time = snowprint >> (LOGICAL_SHARD_BITS + TICKET_BITS);
    let logical_id = (snowprint & DECOMPOSE_LOGICAL_SHARD_BITS) >> TICKET_BITS;
    let ticket_id = snowprint & DECOMPOSE_TICKET_BITS;

    (time, logical_id, ticket_id)
}
