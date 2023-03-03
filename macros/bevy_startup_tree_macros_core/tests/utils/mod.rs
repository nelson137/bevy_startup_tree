use std::fmt::Debug;

pub fn assert_ok<T: PartialEq + Debug>(result: &syn::Result<T>, expected_value: &T) {
    match result {
        Ok(actual_value) => assert_eq!(actual_value, expected_value),
        _ => {
            let result = result.as_ref().map_err(|err| err.to_string());
            assert_eq!(result, Ok(expected_value));
        }
    }
}

pub fn assert_err<T: PartialEq + Debug>(result: &syn::Result<T>, expected_message: &str) {
    match result {
        Err(err) => assert_eq!(err.to_string(), expected_message),
        _ => {
            let result = result.as_ref().map_err(|err| err.to_string());
            assert_eq!(result, Err(expected_message.to_string()));
        }
    }
}

#[macro_export]
macro_rules! __path {
    ($tokens:path) => {
        syn::parse2::<syn::Path>(quote::quote! { $tokens }).expect("failed to parse path")
    };
}
pub use crate::__path as path;
