extern crate cgid;

use std::env;

#[test]
fn test_set_header() {
    let mut content_length: usize = 0;
    let input_key = "Header-test";
    let input_value = "value";
    let input = format!("{}: {}", input_key, input_value);

    let expected_key = "HTTP_HEADER_TEST";  // HTTP_ prefix, upper_case, replace - with _
    let expected_value = "value";

    assert!(cgid::set_header(input, &mut content_length).is_ok());
    assert_eq!(env::var(expected_key).unwrap(), expected_value);
}

