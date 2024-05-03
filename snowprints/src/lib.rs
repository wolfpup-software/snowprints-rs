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
    settings: SnowprintSettings,
    state: SnowprintState,
}

// The point is to distribute ids across logical volume shards evenly
//     - reset sequence every MS to 0 to remain sortable
//     - increase logical volume sequence by 1 every MS
//     - return err if available logical volume ids have been used

// This assumes sequences + logical volume ids occur in the same ms

pub struct SnowprintSettings {
    pub origin_timestamp_ms: SystemTime,
    pub logical_volume_modulo: u64,
    pub logical_volume_base: u64,
}

pub struct SnowprintState {
    pub last_duration_ms: u64,
    pub sequence_id: u64,
    pub logical_volume_id: u64,
    pub last_logical_volume_id: u64,
}

impl Snowprint {
    pub fn new(settings: SnowprintSettings) -> Snowprint {
        Snowprint {
            settings: settings,
            state: SnowprintState {
                last_duration_ms: 0,
                sequence_id: 0,
                logical_volume_id: 0,
                last_logical_volume_id: 0,
            },
        }
    }

    pub fn get_snowprint(&mut self) -> Result<u64, Error> {
        let duration_ms = match SystemTime::now().duration_since(self.settings.origin_timestamp_ms)
        {
            // check time didn't go backward
            Ok(duration) => {
                let temp_duration = duration.as_millis() as u64;
                match temp_duration > self.state.last_duration_ms {
                    true => temp_duration,
                    _ => self.state.last_duration_ms,
                }
            }
            // time went backwards so use the most recent step
            _ => self.state.last_duration_ms,
        };

        compose_snowprint_from_settings_and_state(&mut self.state, &self.settings, duration_ms)
    }
}

fn compose_snowprint_from_settings_and_state(
    state: &mut SnowprintState,
    settings: &SnowprintSettings,
    duration_ms: u64,
) -> Result<u64, Error> {
    // time changed
    if state.last_duration_ms != duration_ms {
        // reset sequence
        // record last logical volume
        // increase logical volume and rotate
        state.sequence_id = 0;
        state.last_duration_ms = duration_ms;
        state.last_logical_volume_id = state.logical_volume_id;
        state.logical_volume_id = (state.logical_volume_id + 1) % settings.logical_volume_modulo;
    } else {
        // time did not change!
        if state.sequence_id + 1 > MAX_SEQUENCES - 1 {
            let next_logical_volume_id =
                (state.logical_volume_id + 1) % settings.logical_volume_modulo;
            // cycled through all sequences on all available logical shards
            if next_logical_volume_id == state.last_logical_volume_id {
                return Err(Error::NoAvailableSequences);
            }
            state.logical_volume_id = next_logical_volume_id;
            state.sequence_id = 0;
        }
    }

    Ok(compose_snowprint(
        duration_ms as u64,
        settings.logical_volume_base + state.logical_volume_id,
        state.sequence_id,
    ))
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
