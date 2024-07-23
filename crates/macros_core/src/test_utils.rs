pub fn assert_err<T>(result: &syn::Result<T>, expected_message: &str) {
    match result {
        Err(err) => assert_eq!(err.to_string(), expected_message),
        _ => panic!("expected Ok result to be an error with message: {expected_message}"),
    }
}
