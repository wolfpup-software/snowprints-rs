use snowprints::{compose_snowprint, decompose_snowprint};

#[test]
fn compose_and_decompose_snowprint() {
    let time = 987654321;
    let logical_id = 7890;
    let ticket_id = 956;

    let snowprint = compose_snowprint(time, logical_id, ticket_id);

    let (decomposed_time, decomposed_logical_id, decomposed_ticket_id) =
        decompose_snowprint(snowprint);
    println!("{}", snowprint);
    println!(
        "{}\n{}\n{}\n",
        decomposed_time, decomposed_logical_id, decomposed_ticket_id
    );
    assert_eq!(time, decomposed_time);
    assert_eq!(logical_id, decomposed_logical_id);
    assert_eq!(ticket_id, decomposed_ticket_id);
}
