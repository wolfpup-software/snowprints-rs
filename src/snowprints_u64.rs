#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};


// u64 bit implementation
const LOGICAL_VOLUME_BIT_LEN: u64 = 13;
const SEQUENCE_BIT_LEN: u64 = 10;
const LOGICAL_VOLUME_BIT_MASK: u64 = ((1 << LOGICAL_VOLUME_BIT_LEN) - 1) << SEQUENCE_BIT_LEN;
const MAX_LOGICAL_VOLUMES: u64 = u32::pow(2, LOGICAL_VOLUME_BIT_LEN as u32) as u64;
const MAX_SEQUENCES: u64 = u32::pow(2, SEQUENCE_BIT_LEN as u32) as u64;
const SEQUENCE_BIT_MASK: u64 = (1 << SEQUENCE_BIT_LEN) - 1;

// #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
// #[derive(Debug, Clone, Eq, PartialEq)]
// pub struct Params {
//     pub logical_volume_base: u64,
//     pub logical_volume_length: u64,
//     pub origin_time_ms: u64,
// }

// #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
// #[derive(Debug, Clone, Eq, PartialEq)]
// struct State {
//     pub duration_ms: u64,
//     pub logical_volume: u64,
//     pub prev_logical_volume: u64,
//     pub sequence: u64,
// }

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Snowprints64 {
    origin_time_duration: SystemTime,
    params: Params,
    state: State,
}

impl Snowprints64 {
    pub fn from(params: Params) -> Result<Snowprints64, Errors> {
        if let Err(err) = check_params(&params) {
            return Err(err);
        }

        let origin_time_duration = UNIX_EPOCH + Duration::from_millis(params.origin_time_ms);

        let duration_ms = match SystemTime::now().duration_since(origin_time_duration) {
            Ok(duration) => duration.as_millis() as u64,
            _ => return Err(Errors::FailedToParseOriginSystemTime),
        };

        Ok(Snowprints64 {
            params: params,
            origin_time_duration: origin_time_duration,
            state: State {
                duration_ms: duration_ms,
                sequence: 0,
                logical_volume: 0,
                prev_logical_volume: 0,
            },
        })
    }

    pub fn create_id(&mut self) -> Result<u64, Errors> {
        let duration_ms =
            get_most_recent_duration_ms(self.origin_time_duration, self.state.duration_ms);

        match duration_ms > self.state.duration_ms {
            true => tick_logical_volume(&mut self.state, &self.params, duration_ms),
            _ => {
                if let Err(err) = tick_sequence(&mut self.state, &self.params) {
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
        get_most_recent_duration_ms(self.origin_time_duration, self.state.duration_ms)
    }

    pub fn get_bit_shifted_timestamp(&self, offset_ms: u64) -> u64 {
        let mut duration_ms =
            get_most_recent_duration_ms(self.origin_time_duration, self.state.duration_ms);

        duration_ms = match offset_ms < duration_ms {
            true => duration_ms - offset_ms,
            _ => 0,
        };

        compose(duration_ms, 0, 0)
    }
}

fn check_params(params: &Params) -> Result<(), Errors> {
    if params.logical_volume_length == 0 {
        return Err(Errors::LogicalVolumeModuloIsZero);
    }
    if MAX_LOGICAL_VOLUMES < (params.logical_volume_base + params.logical_volume_length) {
        return Err(Errors::ExceededAvailableLogicalVolumes);
    }

    Ok(())
}

fn get_most_recent_duration_ms(origin_time_duration: SystemTime, duration_ms: u64) -> u64 {
    if let Ok(duration) = SystemTime::now().duration_since(origin_time_duration) {
        let dur_ms = duration.as_millis() as u64;
        if duration_ms < dur_ms {
            return dur_ms;
        }
    }

    duration_ms
}

fn tick_logical_volume(state: &mut State, params: &Params, duration_ms: u64) {
    state.duration_ms = duration_ms;
    state.sequence = 0;
    state.prev_logical_volume = state.logical_volume;
    state.logical_volume = (state.logical_volume + 1) % params.logical_volume_length;
}

fn tick_sequence(state: &mut State, params: &Params) -> Result<(), Errors> {
    state.sequence += 1;
    if state.sequence < MAX_SEQUENCES {
        return Ok(());
    }

    state.sequence = 0;
    let next_logical_volume = (state.logical_volume + 1) % params.logical_volume_length;
    if state.prev_logical_volume != next_logical_volume {
        state.logical_volume = next_logical_volume;
        return Ok(());
    }

    Err(Errors::ExceededAvailableSequences)
}

pub fn compose(ms_timestamp: u64, logical_volume: u64, ticket_id: u64) -> u64 {
    ms_timestamp << (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN)
        | logical_volume << SEQUENCE_BIT_LEN
        | ticket_id
}

pub fn decompose(snowprint: u64) -> (u64, u64, u64) {
    let time = snowprint >> (LOGICAL_VOLUME_BIT_LEN + SEQUENCE_BIT_LEN);
    let logical_volume = (snowprint & LOGICAL_VOLUME_BIT_MASK) >> SEQUENCE_BIT_LEN;
    let ticket_id = snowprint & SEQUENCE_BIT_MASK;

    (time, logical_volume, ticket_id)
}
