use bevy_startup_tree_macros_core::{StartupTree, SystemTree};
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn startup_tree(input: TokenStream) -> TokenStream {
    let tree: StartupTree = parse_macro_input!(input);
    quote! {
        #tree
    }
    .into()
}

#[proc_macro]
pub fn system_tree(input: TokenStream) -> TokenStream {
    let tree: SystemTree = parse_macro_input!(input);
    quote! {
        #tree
    }
    .into()
}
