#[macro_export]
macro_rules! __path {
    ($tokens:path) => {
        syn::parse2(quote::quote! { $tokens }).expect("failed to parse path")
    };
}
pub use crate::__path as path;
