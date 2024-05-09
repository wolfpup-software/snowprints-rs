use super::*;
use std::time::{Duration, SystemTime};

#[test]
fn test_check_failed_settings() {
    let mod_fail_settings = Settings {
        origin_system_time: SystemTime::now(),
        logical_volume_base: 4096,
        logical_volume_length: 0,
    };
    let snowprinter = Snowprint::new(mod_fail_settings);
    assert_eq!(snowprinter, Err(Error::LogicalVolumeModuloIsZero));

    let exceed_fail_settings = Settings {
        origin_system_time: SystemTime::now(),
        logical_volume_base: 4096,
        logical_volume_length: 8192,
    };
    let snowprinter2 = Snowprint::new(exceed_fail_settings);
    assert_eq!(snowprinter2, Err(Error::ExceededAvailableLogicalVolumes));
}

#[test]
fn test_get_most_recent_duration_ms() {
    let duration = SystemTime::now();

    let duration_ms = get_most_recent_duration_ms(duration, 0);
    assert_eq!(duration_ms, 0);

    let greater_duration = duration + Duration::from_millis(1);
    let greater_duration_ms = get_most_recent_duration_ms(greater_duration, duration_ms);

    assert_eq!(greater_duration_ms, duration_ms);

    let greater_duration_ms = get_most_recent_duration_ms(duration, greater_duration_ms);
    assert_eq!(greater_duration_ms, duration_ms);
}

#[test]
fn test_modify_state_time_changed() {
    let mut state = State {
        prev_duration_ms: 0,
        sequence: 82,
        logical_volume: 0,
        prev_logical_volume: 0,
    };
    let expected_state = State {
        prev_duration_ms: 5,
        sequence: 0,
        logical_volume: 1,
        prev_logical_volume: 0,
    };
    modify_state_time_changed(&mut state, 8192, 5);
    assert_eq!(expected_state, state);

    let expected_state = State {
        prev_duration_ms: 6,
        sequence: 0,
        logical_volume: 2,
        prev_logical_volume: 1,
    };
    modify_state_time_changed(&mut state, 8192, 6);
    assert_eq!(expected_state, state);
}

#[test]
fn test_modify_state_time_did_not_change() {
    // sequence
    let mut state = State {
        prev_duration_ms: 0,
        sequence: 0,
        logical_volume: 0,
        prev_logical_volume: 0,
    };
    let result = modify_state_time_did_not_change(&mut state, 8192);
    assert_eq!(Ok(()), result);

    let expected_state = State {
        prev_duration_ms: 0,
        sequence: 1,
        logical_volume: 0,
        prev_logical_volume: 0,
    };
    assert_eq!(expected_state, state);

    // rollover
    let mut state = State {
        prev_duration_ms: 0,
        sequence: 1023,
        logical_volume: 8191,
        prev_logical_volume: 8191,
    };
    let result = modify_state_time_did_not_change(&mut state, 8192);
    assert_eq!(Ok(()), result);

    let expected_state = State {
        prev_duration_ms: 0,
        sequence: 0,
        logical_volume: 0,
        prev_logical_volume: 8191,
    };
    assert_eq!(expected_state, state);

    // fail
    let mut state = State {
        prev_duration_ms: 0,
        sequence: 1023,
        logical_volume: 8191,
        prev_logical_volume: 0,
    };
    let result = modify_state_time_did_not_change(&mut state, 8192);
    assert_eq!(Err(Error::ExceededAvailableSequences), result);
}

#[test]
fn test_compose_from_settings_and_state() {
    // time did not change
    let settings = Settings {
        origin_system_time: SystemTime::now(),
        logical_volume_base: 4096,
        logical_volume_length: 4096,
    };
    let mut state = State {
        prev_duration_ms: 0,
        sequence: 255,
        logical_volume: 2048,
        prev_logical_volume: 4096,
    };

    let duration_ms = 0;
    let snowprint = compose_from_settings_and_state(&settings, &mut state, duration_ms);
    match snowprint {
        Ok(sp) => {
            let (_timestamp, logical_volume, sequence) = decompose(sp);
            assert_eq!(logical_volume, 6144);
            assert_eq!(sequence, 256);
        }
        // error by comparing result to incorrect error
        Err(err) => assert_eq!(Error::ExceededAvailableLogicalVolumes, err),
    }

    // fail out
    let mut state = State {
        prev_duration_ms: 0,
        sequence: 1023,
        logical_volume: 4095,
        prev_logical_volume: 0,
    };

    let snowprint = compose_from_settings_and_state(&settings, &mut state, duration_ms);
    assert_eq!(Err(Error::ExceededAvailableSequences), snowprint);

    // time changed
    let duration_ms = 1;
    let mut state = State {
        prev_duration_ms: 0,
        sequence: 1023,
        logical_volume: 4095,
        prev_logical_volume: 0,
    };

    let snowprint = compose_from_settings_and_state(&settings, &mut state, duration_ms);
    match snowprint {
        Ok(sp) => {
            let (_timestamp, logical_volume, sequence) = decompose(sp);
            assert_eq!(logical_volume, 4096);
            assert_eq!(sequence, 0);
        }
        // error by comparing result to incorrect error
        Err(err) => assert_eq!(Error::ExceededAvailableLogicalVolumes, err),
    }
}
