use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

// https://instagram-engineering.com/sharding-ids-at-instagram-1cf5a71e5a5c

const SEQUENCE_BIT_LEN: u64 = 10;
const SEQUENCE_BIT_MASK: u64 = (1 << SEQUENCE_BIT_LEN) - 1;
const MAX_SEQUENCES: u64 = 2 ^ SEQUENCE_BIT_LEN;
const LOGICAL_VOLUME_BIT_LEN: u64 = 13;
const LOGICAL_VOLUME_BIT_MASK: u64 = ((1 << LOGICAL_VOLUME_BIT_LEN) - 1) << SEQUENCE_BIT_LEN;
const MAX_LOGICAL_VOLUMES: u64 = 2 ^ LOGICAL_VOLUME_BIT_LEN;
const TIME_BIT_LEN: u64 = 41;
// number of milliseconds since UTC epoch time
const JANUARY_1ST_2024_AS_DURATION: Duration = Duration::from_millis(1704067200541);

pub enum Error {
    NoAvailableSequences,
}

pub struct Snowprint {
    last_duration: u128,
    logical_volume_id: u64,
    logical_volume_base: u64,
    logical_volume_count: u64,
    sequence_id: u64,
}

// The point is to distribute ids across logical volume shards evenly
//     - reset sequence every MS to 0 to remain sortable
//     - increase logical volume sequence by 1 every MS
//     - return err if available logical volume ids have been used

// This assumes sequences + logical volume ids occur in the same ms

pub struct SnowprintSettings {
    pub origin_timestamp: SystemTime,
    pub logical_volume_count: u64,
    pub logical_volume_base: u64,
}

pub struct SnowprintState {
    pub last_duration_ms: u64,
    pub sequence_id: u64,
    pub logical_volume_id: u64,
    pub last_logical_volume_id: u64,
}

impl Snowprint {
    pub fn new() -> Snowprint {
        Snowprint {
            settings: SnowprintSettings {},
            state: SnowprintState {},
        }
    }

    pub fn get_snowprint(&mut self) -> Result<u64, Error> {
        let origin_timestamp: SystemTime = UNIX_EPOCH + JANUARY_1ST_2024_AS_DURATION;
        let duration_ms = match SystemTime::now().duration_since(origin_timestamp) {
            // check time didn't go backward
            Ok(duration) => {
                let temp_duration = duration.as_millis();
                match temp_duration > self.last_duration {
                    true => temp_duration,
                    _ => self.last_duration,
                }
            }
            // time went backwards so use the most recent step
            _ => self.last_duration,
        };

        // time changed
        if self.last_duration != duration_ms {
            // reset sequence
            // record last logical volume
            // increase logical volume and rotate
            self.sequence_id = 0;
            self.last_duration = duration_ms;
            self.last_logical_volume_id = self.logical_volume_id;
            self.logical_volume_id += 1;
            self.logical_volume_id %= self.logical_volume_count;
        } else {
            // time did not change!
            self.sequence_id += 1;
            if self.sequence_id > MAX_SEQUENCES - 1 {
                self.logical_volume_id += 1;
                self.logical_volume_id %= self.logical_volume_count;
                if self.last_logical_volume_id == self.logical_volume_id {
                    return Err(Error::NoAvailableSequences);
                }
                self.sequence_id = 0;
            }
        }

        Ok(compose_snowprint(
            duration_ms as u64,
            self.logical_volume_base + self.logical_volume_id,
            self.sequence_id,
        ))
    }
}

// at it's core this is a snowprint
pub fn compose_snowprint(ms_timestamp: u64, logical_id: u64, ticket_id: u64) -> u64 {
    ms_timestamp << (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN)
        | logical_id << SEQUENCE_BIT_LEN
        | ticket_id
}

pub fn decompose_snowprint(snowprint: u64) -> (u64, u64, u64) {
    let time = snowprint >> (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN);
    let logical_id = (snowprint & LOGICAL_VOLUME_BIT_MASK) >> SEQUENCE_BIT_LEN;
    let ticket_id = snowprint & SEQUENCE_BIT_MASK;

    (time, logical_id, ticket_id)
}
