use generate_stage_labels::StageLabelGenerator;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod generate_stage_labels;
mod tree;

use tree::StartupTree;

/// Generate a tree of startup systems that can be used by
/// [`AddStartupTree`](bevy_startup_tree::AddStartupTree).
///
/// TODO
#[proc_macro]
pub fn startup_tree(input: TokenStream) -> TokenStream {
    let tree: StartupTree = parse_macro_input!(input);
    quote! {
        #tree
    }
    .into()
}

#[proc_macro]
pub fn generage_stage_labels(input: TokenStream) -> TokenStream {
    let generator: StageLabelGenerator = parse_macro_input!(input);
    quote! {
        #generator
    }
    .into()
}
