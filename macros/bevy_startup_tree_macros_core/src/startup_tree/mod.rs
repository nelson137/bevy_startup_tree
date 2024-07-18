use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    punctuated as punct, Result, Token,
};

use crate::tree::{Branch, Tree};

mod node;

pub use self::node::Node;

pub struct StartupTree(Tree<Node>);

impl Parse for StartupTree {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut tree: Tree<Node> = input.parse()?;
        tree.set_depth_root();
        Ok(Self(tree))
    }
}

impl ToTokens for StartupTree {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let tree_levels = tree_to_levels(&self.0);
        let span = Span::call_site();

        let vec_elements = tree_levels.into_iter().map(|level| syn::Macro {
            path: syn::Path {
                leading_colon: Some(Token![::](span)),
                segments: punct::Punctuated::from_iter([
                    syn::PathSegment::from(syn::Ident::new("std", span)),
                    syn::PathSegment::from(syn::Ident::new("vec", span)),
                ]),
            },
            bang_token: Token![!](span),
            delimiter: syn::MacroDelimiter::Bracket(syn::token::Bracket(span)),
            tokens: {
                let mut elements = TokenStream2::new();
                elements.append_separated(
                    level.into_iter().map(Node::as_into_descriptor_call),
                    Token![,](span),
                );
                elements
            },
        });
        let vec_elements = punct::Punctuated::<_, Token![,]>::from_iter(vec_elements);

        quote! {
            vec![ #vec_elements ]
        }
        .to_tokens(tokens);
    }
}

fn tree_to_levels(tree: &Tree<Node>) -> Vec<Vec<&Node>> {
    let mut tree_levels: Vec<Vec<&Node>> = Vec::new();
    tree_to_levels_impl(&mut tree_levels, tree, 0);
    tree_levels
}

fn tree_to_levels_impl<'tree>(
    tree_levels: &mut Vec<Vec<&'tree Node>>,
    subtree: &'tree Tree<Node>,
    depth: usize,
) {
    fn push_branch<'tree>(
        levels: &mut Vec<Vec<&'tree Node>>,
        branch: &'tree Branch<Node>,
        depth: usize,
    ) {
        if depth >= levels.len() {
            levels.push(vec![branch.node()]);
        } else {
            levels[depth].push(branch.node());
        }

        match branch {
            Branch::Arm(_, _, b) => push_branch(levels, b, depth + 1),
            Branch::Tree(_, _, t) => tree_to_levels_impl(levels, t, depth + 1),
            Branch::Leaf(_) => {}
        }
    }

    for branch in &subtree.branches {
        push_branch(tree_levels, branch, depth);
    }
}
