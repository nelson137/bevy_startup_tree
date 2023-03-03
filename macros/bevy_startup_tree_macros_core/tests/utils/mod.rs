use std::fmt::Debug;

pub fn assert_result<T: PartialEq + Debug>(actual: &syn::Result<T>, expected: &Result<T, &str>) {
    fn normalize<T, E: ToString>(r: &Result<T, E>) -> Result<&T, String> {
        r.as_ref().map_err(|err| err.to_string())
    }
    match (actual, expected) {
        (Ok(actual_value), Ok(expected_value)) => assert_eq!(actual_value, expected_value),
        (Err(actual_msg), &Err(expected_msg)) => assert_eq!(actual_msg.to_string(), expected_msg),
        _ => assert_eq!(normalize(actual), normalize(expected)),
    }
}

#[macro_export]
macro_rules! __path {
    ($tokens:path) => {
        syn::parse2::<syn::Path>(quote::quote! { $tokens }).expect("failed to parse path")
    };
}
pub use crate::__path as path;
