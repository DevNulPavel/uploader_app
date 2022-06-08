use serde::Deserialize;
use serde_json_string_parse_helper::{JsonParseError, ParseJson};

#[derive(Deserialize, Debug)]
struct TestStruct {
    key: String,
}

#[test]
fn success_string_parsing() {
    #[rustfmt::skip]
    let text = String::from(r#"{
        "key": "value"
    }"#);

    let parse_result: TestStruct = text.parse_json_with_data_err().expect("Parsing failed");
    assert_eq!(parse_result.key, "value");
}

#[test]
fn failed_string_parsing() {
    #[rustfmt::skip]
    let text = String::from(r#"{
        "key" ___ "value"
    }"#);

    let parse_error: JsonParseError<String> = text
        .clone()
        .parse_json_with_data_err::<TestStruct>()
        .expect_err("Parsing must fail");

    assert_eq!(parse_error.original_data, text);
}

#[test]
fn success_str_parsing() {
    #[rustfmt::skip]
    let text = r#"{
        "key": "value"
    }"#;

    let parse_result: TestStruct = text.parse_json_with_data_err().expect("Parsing failed");
    assert_eq!(parse_result.key, "value");
}

#[test]
fn failed_str_parsing() {
    #[rustfmt::skip]
    let text = r#"{
        "key" ___ "value"
    }"#;

    let parse_error: JsonParseError<&str> = text
        .parse_json_with_data_err::<TestStruct>()
        .expect_err("Parsing must fail");

    assert_eq!(parse_error.original_data, text);
}