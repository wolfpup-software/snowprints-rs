use snowprints::{compose, decompose, Error, Settings, Snowprint};
use std::time::{Duration, UNIX_EPOCH};

const JANUARY_1ST_2024_AS_MS: u64 = 1704096000000;
const JANUARY_1ST_2024_AS_DURATION: Duration = Duration::from_millis(JANUARY_1ST_2024_AS_MS);

#[test]
fn compose_and_decompose() {
    let time = 987654321;
    let logical_id = 7890;
    let ticket_id = 956;

    let snowprint = compose(time, logical_id, ticket_id);
    let (d_time, d_logical_id, d_ticket_id) = decompose(snowprint);

    assert_eq!(time, d_time);
    assert_eq!(logical_id, d_logical_id);
    assert_eq!(ticket_id, d_ticket_id);
}

#[test]
fn compose_and_decompose_from_a_real_date() {
    let logical_id = 7890;
    let ticket_id = 956;

    let snowprint = compose(JANUARY_1ST_2024_AS_MS, logical_id, ticket_id);
    let (d_time, d_logical_id, d_ticket_id) = decompose(snowprint);

    assert_eq!(JANUARY_1ST_2024_AS_MS, d_time);
    assert_eq!(logical_id, d_logical_id);
    assert_eq!(ticket_id, d_ticket_id);
}

#[test]
fn snowprint_struct_builds_and_returns_snowprint() {
    let settings = Settings {
        origin_system_time: UNIX_EPOCH + JANUARY_1ST_2024_AS_DURATION,
        logical_volume_base: 0,
        logical_volume_modulo: 8192,
    };

    let mut builder = match Snowprint::new(settings) {
        Ok(snow) => snow,
        // error by comparing result to incorrect error
        Err(err) => return assert_eq!(Error::ExceededAvailableSequences, err),
    };

    let snowprint = builder.compose();
    match snowprint {
        Ok(sp) => {
            let (_timestamp, logical_volume, sequence) = decompose(sp);

            assert_eq!(logical_volume, 0);
            assert_eq!(sequence, 1);
        }
        // error by comparing result to incorrect error
        Err(err) => assert_eq!(Error::ExceededAvailableLogicalVolumes, err),
    }
}

// test starting logical volumes from zero
// test last possible logical volume sequence
