pub fn assert_err<T>(result: &syn::Result<T>, expected_message: &str) {
    match result {
        Err(err) => assert_eq!(err.to_string(), expected_message),
        _ => panic!("expected Ok result to be an error with message: {expected_message}"),
    }
}

#[macro_export]
macro_rules! __path {
    ($tokens:path) => {
        syn::parse2(quote::quote! { $tokens }).expect("failed to parse path")
    };
}
pub use crate::__path as path;
