use std::time::SystemTime;

const LOGICAL_VOLUME_BIT_LEN: u64 = 12;
const SEQUENCE_BIT_LEN: u64 = 10;
const LOGICAL_VOLUME_BIT_MASK: i64 = ((1 << LOGICAL_VOLUME_BIT_LEN) - 1) << SEQUENCE_BIT_LEN;
const MAX_LOGICAL_VOLUMES: u64 = u32::pow(2, LOGICAL_VOLUME_BIT_LEN as u32) as u64;
const MAX_SEQUENCES: u64 = i32::pow(2, SEQUENCE_BIT_LEN as u32) as u64;
const SEQUENCE_BIT_MASK: i64 = (1 << SEQUENCE_BIT_LEN) - 1;

const NEGATIVE_BIT_MASK: i64 = !(1 << 63);

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Params {
    pub logical_volume_base: u64,
    pub logical_volume_length: u64,
    pub origin_time_ms: u64,
}

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct State {
    pub duration_ms: u64,
    pub logical_volume: u64,
    pub prev_logical_volume: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Errors {
    ExceededAvailableLogicalVolumes,
    ExceededAvailableSequences,
    FailedToParseOriginSystemTime,
    LogicalVolumeModuloIsZero,
}

pub fn check_params(params: &Params) -> Result<(), Errors> {
    if params.logical_volume_length == 0 {
        return Err(Errors::LogicalVolumeModuloIsZero);
    }
    if MAX_LOGICAL_VOLUMES < (params.logical_volume_base + params.logical_volume_length) {
        return Err(Errors::ExceededAvailableLogicalVolumes);
    }

    Ok(())
}

pub fn get_most_recent_duration_ms(origin_time_duration: SystemTime, duration_ms: u64) -> u64 {
    if let Ok(duration) = SystemTime::now().duration_since(origin_time_duration) {
        let dur_ms = duration.as_millis() as u64;
        if duration_ms < dur_ms {
            return dur_ms;
        }
    }

    duration_ms
}

pub fn tick_logical_volume(state: &mut State, params: &Params, duration_ms: u64) {
    state.duration_ms = duration_ms;
    state.sequence = 0;
    state.prev_logical_volume = state.logical_volume;
    state.logical_volume = (state.logical_volume + 1) % params.logical_volume_length;
}

pub fn tick_sequence(state: &mut State, params: &Params) -> Result<(), Errors> {
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::time::{Duration, SystemTime};

//     #[test]
//     fn test_check_failed_settings() {
//         let mod_fail_settings = Params {
//             origin_time_ms: SystemTime::now().as_millis() as u64,
//             logical_volume_base: 4096,
//             logical_volume_length: 0,
//         };
//         let snowprinter = Snowprint::new(mod_fail_settings);
//         assert_eq!(snowprinter, Err(Errors::LogicalVolumeModuloIsZero));

//         let exceed_fail_settings = Params {
//             origin_time_ms: SystemTime::now().as_millis() as u64,
//             logical_volume_base: 4096,
//             logical_volume_length: 8192,
//         };
//         let snowprinter2 = Snowprint::new(exceed_fail_settings);
//         assert_eq!(snowprinter2, Err(Errors::ExceededAvailableLogicalVolumes));
//     }

//     #[test]
//     fn test_get_most_recent_duration_ms() {
//         let duration = SystemTime::now().as_millis() as u64;

//         let duration_ms = get_most_recent_duration_ms(duration, 0);
//         assert_eq!(duration_ms, 0);

//         let greater_duration = duration + Duration::from_millis(1);
//         let greater_duration_ms = get_most_recent_duration_ms(greater_duration, duration_ms);

//         assert_eq!(greater_duration_ms, duration_ms);

//         let greater_duration_ms = get_most_recent_duration_ms(duration, greater_duration_ms);
//         assert_eq!(greater_duration_ms, duration_ms);
//     }

//     #[test]
//     fn test_modify_state_time_changed() {
//         let mut state = State {
//             duration_ms: 0,
//             sequence: 82,
//             logical_volume: 0,
//             prev_logical_volume: 0,
//         };
//         let expected_state = State {
//             duration_ms: 5,
//             sequence: 0,
//             logical_volume: 1,
//             prev_logical_volume: 0,
//         };
//         modify_state_time_changed(&mut state, 8192, 5);
//         assert_eq!(expected_state, state);

//         let expected_state = State {
//             duration_ms: 6,
//             sequence: 0,
//             logical_volume: 2,
//             prev_logical_volume: 1,
//         };
//         modify_state_time_changed(&mut state, 8192, 6);
//         assert_eq!(expected_state, state);
//     }

//     #[test]
//     fn test_modify_state_time_did_not_change() {
//         // sequence
//         let mut state = State {
//             duration_ms: 0,
//             sequence: 0,
//             logical_volume: 0,
//             prev_logical_volume: 0,
//         };
//         let result = modify_state_time_did_not_change(&mut state, 8192);
//         assert_eq!(Ok(()), result);

//         let expected_state = State {
//             duration_ms: 0,
//             sequence: 1,
//             logical_volume: 0,
//             prev_logical_volume: 0,
//         };
//         assert_eq!(expected_state, state);

//         // rollover
//         let mut state = State {
//             duration_ms: 0,
//             sequence: 1023,
//             logical_volume: 8191,
//             prev_logical_volume: 8191,
//         };
//         let result = modify_state_time_did_not_change(&mut state, 8192);
//         assert_eq!(Ok(()), result);

//         let expected_state = State {
//             duration_ms: 0,
//             sequence: 0,
//             logical_volume: 0,
//             prev_logical_volume: 8191,
//         };
//         assert_eq!(expected_state, state);

//         // fail
//         let mut state = State {
//             duration_ms: 0,
//             sequence: 1023,
//             logical_volume: 8191,
//             prev_logical_volume: 0,
//         };
//         let result = modify_state_time_did_not_change(&mut state, 8192);
//         assert_eq!(Err(Errors::ExceededAvailableSequences), result);
//     }

//     #[test]
//     fn test_compose_from_settings_and_state() {
//         // time did not change
//         let settings = Params {
//             origin_time_ms: SystemTime::now().as_millis() as u64,
//             logical_volume_base: 4096,
//             logical_volume_length: 4096,
//         };
//         let mut state = State {
//             duration_ms: 0,
//             sequence: 255,
//             logical_volume: 2048,
//             prev_logical_volume: 4096,
//         };

//         let duration_ms = 0;
//         let snowprint = compose_from_settings_and_state(&settings, &mut state, duration_ms);
//         match snowprint {
//             Ok(sp) => {
//                 let (_timestamp, logical_volume, sequence) = decompose(sp);
//                 assert_eq!(logical_volume, 6144);
//                 assert_eq!(sequence, 256);
//             }
//             // error by comparing result to incorrect error
//             Err(err) => assert_eq!(Errors::ExceededAvailableLogicalVolumes, err),
//         }

//         // fail out
//         let mut state = State {
//             duration_ms: 0,
//             sequence: 1023,
//             logical_volume: 4095,
//             prev_logical_volume: 0,
//         };

//         let snowprint = compose_from_settings_and_state(&settings, &mut state, duration_ms);
//         assert_eq!(Err(Errors::ExceededAvailableSequences), snowprint);

//         // time changed
//         let duration_ms = 1;
//         let mut state = State {
//             duration_ms: 0,
//             sequence: 1023,
//             logical_volume: 4095,
//             prev_logical_volume: 0,
//         };

//         let snowprint = compose_from_settings_and_state(&settings, &mut state, duration_ms);
//         match snowprint {
//             Ok(sp) => {
//                 let (_timestamp, logical_volume, sequence) = decompose(sp);
//                 assert_eq!(logical_volume, 4096);
//                 assert_eq!(sequence, 0);
//             }
//             // error by comparing result to incorrect error
//             Err(err) => assert_eq!(Errors::ExceededAvailableLogicalVolumes, err),
//         }
//     }
// }
