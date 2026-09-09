use snowprints::{Errors, Params, Snowprints, compose, decompose};

const JANUARY_1ST_2024_AS_MS: u64 = 1704096000000;

#[test]
fn compose_and_decompose() {
    let time = 987654321;
    let logical_volume = 4095;
    let sequence = 956;

    let snowprint = compose(time, logical_volume, sequence);
    let (d_time, d_logical_volume, d_sequence) = decompose(snowprint);

    assert_eq!(time, d_time, "time does not match");
    assert_eq!(
        logical_volume, d_logical_volume,
        "logical volume does not match"
    );
    assert_eq!(sequence, d_sequence, "sequence volume does not match");
}

#[test]
fn compose_and_decompose_from_a_real_date() {
    let logical_volume = 4095;
    let sequence = 956;

    let snowprint = compose(JANUARY_1ST_2024_AS_MS, logical_volume, sequence);
    let (d_time, d_logical_volume, d_sequence) = decompose(snowprint);

    assert_eq!(JANUARY_1ST_2024_AS_MS, d_time);
    assert_eq!(logical_volume, d_logical_volume);
    assert_eq!(sequence, d_sequence);
}

#[test]
fn snowprint_struct_builds_and_returns_snowprint() {
    let params = Params {
        origin_time_ms: JANUARY_1ST_2024_AS_MS,
        logical_volume_base: 0,
        logical_volume_length: 4095,
    };

    let mut snowprints = match Snowprints::from(params) {
        Ok(snow) => snow,
        // error by comparing result to incorrect error
        Err(err) => return assert_eq!(Errors::ExceededAvailableSequences, err),
    };

    let snowprint = snowprints.create_id();
    match snowprint {
        Ok(sp) => {
            let (_timestamp, logical_volume, sequence) = decompose(sp);

            assert_eq!(logical_volume, 0);
            assert_eq!(sequence, 1);
        }
        // error by comparing result to incorrect error
        Err(err) => assert_eq!(Errors::ExceededAvailableLogicalVolumes, err),
    }
}

#[test]
fn test_check_failed_settings() {
    let mod_fail_settings = Params {
        origin_time_ms: 0,
        logical_volume_base: 2048,
        logical_volume_length: 0,
    };
    let snowprinter = Snowprints::from(mod_fail_settings);
    assert_eq!(snowprinter, Err(Errors::LogicalVolumeModuloIsZero));

    let exceed_fail_settings = Params {
        origin_time_ms: 0,
        logical_volume_base: 2048,
        logical_volume_length: 2049,
    };
    let snowprinter2 = Snowprints::from(exceed_fail_settings);
    assert_eq!(snowprinter2, Err(Errors::ExceededAvailableLogicalVolumes));
}
