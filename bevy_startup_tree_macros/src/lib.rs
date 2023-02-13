use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod tree;

use tree::StartupTree;

#[proc_macro]
pub fn startup_tree(input: TokenStream) -> TokenStream {
    let tree: StartupTree = parse_macro_input!(input);
    quote! {
        #tree
    }
    .into()
}
