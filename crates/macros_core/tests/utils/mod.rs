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
    ($ident:ident $(:: $rest:ident)*) => {
        ::syn::Path {
            leading_colon: ::core::option::Option::None,
            segments: ::syn::punctuated::Punctuated::from_iter([
                ::syn::PathSegment::from(
                    ::syn::Ident::new(stringify!($ident), ::proc_macro2::Span::call_site()),
                ),
                $(::syn::PathSegment::from(
                    ::syn::Ident::new(stringify!($rest), ::proc_macro2::Span::call_site()),
                ),)*
            ]),
        }
    };
    (:: $ident:ident $(:: $rest:ident)*) => {
        ::syn::Path {
            leading_colon: ::core::option::Option::Some(::core::default::Default::default()),
            segments: ::syn::punctuated::Punctuated::from_iter([
                ::syn::PathSegment::from(
                    ::syn::Ident::new(stringify!($ident), ::proc_macro2::Span::call_site()),
                ),
                $(::syn::PathSegment::from(
                    ::syn::Ident::new(stringify!($rest), ::proc_macro2::Span::call_site()),
                ),)*
            ]),
        }
    };
}
pub use crate::__path as path;
