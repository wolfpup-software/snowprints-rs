use snowprints::{compose_snowprint, decompose_snowprint, Settings, Snowprint};
use std::time::Duration;

const JANUARY_1ST_2024_AS_MS: u64 = 1704096000000;
const JANUARY_1ST_2024_AS_DURATION: Duration = Duration::from_millis(JANUARY_1ST_2024_AS_MS);

#[test]
fn compose_and_decompose_snowprint() {
    let time = 987654321;
    let logical_id = 7890;
    let ticket_id = 956;

    let snowprint = compose_snowprint(time, logical_id, ticket_id);
    let (d_time, d_logical_id, d_ticket_id) = decompose_snowprint(snowprint);

    assert_eq!(time, d_time);
    assert_eq!(logical_id, d_logical_id);
    assert_eq!(ticket_id, d_ticket_id);
}

#[test]
fn compose_and_decompose_snowprint_from_a_real_date() {
    let logical_id = 7890;
    let ticket_id = 956;

    let snowprint = compose_snowprint(JANUARY_1ST_2024_AS_MS, logical_id, ticket_id);
    let (d_time, d_logical_id, d_ticket_id) = decompose_snowprint(snowprint);

    assert_eq!(JANUARY_1ST_2024_AS_MS, d_time);
    assert_eq!(logical_id, d_logical_id);
    assert_eq!(ticket_id, d_ticket_id);
}

#[test]
fn snowprint_struct_builds_and_returns_snowprint() {
    let settings = Settings {
        origin_duration: JANUARY_1ST_2024_AS_DURATION,
        logical_volume_base: 0,
        logical_volume_modulo: 8192,
    };

    let mut builder = match Snowprint::new(settings) {
        Ok(snow) => snow,
        Err(err) => return assert_eq!("fail", format!("{:?}", err)),
    };

    let snowprint = builder.get_snowprint();

    println!("{:?}", snowprint)
}
