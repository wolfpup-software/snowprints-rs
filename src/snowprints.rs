// An i64 bit implementation
// SQL sortable without bit-shifting
use crate::flyweight;
use crate::flyweight::{Errors, Params, State};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LOGICAL_VOLUME_BIT_LEN: u64 = 12;
const SEQUENCE_BIT_LEN: u64 = 10;
const LOGICAL_VOLUME_BIT_MASK: i64 = ((1 << LOGICAL_VOLUME_BIT_LEN) - 1) << SEQUENCE_BIT_LEN;
const MAX_LOGICAL_VOLUMES: u64 = u32::pow(2, LOGICAL_VOLUME_BIT_LEN as u32) as u64;
const MAX_SEQUENCES: u64 = i32::pow(2, SEQUENCE_BIT_LEN as u32) as u64;
const SEQUENCE_BIT_MASK: i64 = (1 << SEQUENCE_BIT_LEN) - 1;
const NEGATIVE_BIT_MASK: i64 = !(1 << 63);

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Snowprints {
    origin_time_duration: SystemTime,
    params: Params,
    state: State,
}

impl Snowprints {
    pub fn from(params: Params) -> Result<Snowprints, Errors> {
        if let Err(err) = flyweight::check_params(&params) {
            return Err(err);
        }

        let origin_time_duration = UNIX_EPOCH + Duration::from_millis(params.origin_time_ms);

        let duration_ms = match SystemTime::now().duration_since(origin_time_duration) {
            Ok(duration) => duration.as_millis() as u64,
            _ => return Err(Errors::FailedToParseOriginSystemTime),
        };

        Ok(Snowprints {
            params: params,
            origin_time_duration: origin_time_duration,
            state: State {
                duration_ms,
                sequence: 0,
                logical_volume: 0,
                prev_logical_volume: 0,
            },
        })
    }

    pub fn create_id(&mut self) -> Result<i64, Errors> {
        let duration_ms = flyweight::get_most_recent_duration_ms(
            self.origin_time_duration,
            self.state.duration_ms,
        );

        match duration_ms > self.state.duration_ms {
            true => flyweight::tick_logical_volume(&mut self.state, &self.params, duration_ms),
            _ => {
                if let Err(err) = flyweight::tick_sequence(&mut self.state, &self.params) {
                    return Err(err);
                };
            }
        }

        Ok(compose(
            duration_ms,
            self.params.logical_volume_base + self.state.logical_volume,
            self.state.sequence,
        ))
    }

    pub fn get_timestamp(&self) -> u64 {
        flyweight::get_most_recent_duration_ms(self.origin_time_duration, self.state.duration_ms)
    }

    pub fn get_bit_shifted_timestamp(&self, offset_ms: u64) -> i64 {
        let mut duration_ms = flyweight::get_most_recent_duration_ms(
            self.origin_time_duration,
            self.state.duration_ms,
        );

        duration_ms = match offset_ms < duration_ms {
            true => duration_ms - offset_ms,
            _ => 0,
        };

        compose(duration_ms, 0, 0)
    }
}

pub fn compose(ms_timestamp: u64, logical_volume: u64, ticket_id: u64) -> i64 {
    let shifted = ms_timestamp << (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN);
    println!("try time {:?} {:?}", ms_timestamp, shifted);
    println!(
        "try time {:?} {:?}",
        shifted,
        (shifted as i64 & NEGATIVE_BIT_MASK) >> (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN)
    );

    // 111010110111100110100010110001
    // 1110101101111001101000101100010000000000000000000000

    // println!("{:?}", ms_timestamp << (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN);

    (ms_timestamp << (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN)
        | logical_volume << SEQUENCE_BIT_LEN
        | ticket_id) as i64
}

pub fn decompose(snowprint: i64) -> (u64, u64, u64) {
    println!(
        "decompose {:?} {:?}",
        snowprint,
        (snowprint & NEGATIVE_BIT_MASK) >> (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN)
    );
    println!(
        "decompose {:?} {:?}",
        snowprint,
        (snowprint) >> (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN)
    );

    let time = (snowprint & NEGATIVE_BIT_MASK) >> (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN);
    let logical_volume = (snowprint & LOGICAL_VOLUME_BIT_MASK) >> SEQUENCE_BIT_LEN;
    let ticket_id = snowprint & SEQUENCE_BIT_MASK;

    println!("decompose {:?} {:?} {:?}", time, logical_volume, ticket_id);

    (time as u64, logical_volume as u64, ticket_id as u64)
}
