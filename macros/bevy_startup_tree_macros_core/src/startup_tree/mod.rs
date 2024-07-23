use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    punctuated as punct, Result, Token,
};

use crate::tree::{Node, Tree};

mod node;

pub use self::node::ExprNode;

pub struct StartupTree(Tree<ExprNode>);

impl Parse for StartupTree {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
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
                    level.into_iter().map(ExprNode::as_into_descriptor_call),
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

fn tree_to_levels(tree: &Tree<ExprNode>) -> Vec<Vec<&ExprNode>> {
    let mut tree_levels: Vec<Vec<&ExprNode>> = Vec::new();
    tree_to_levels_impl(&mut tree_levels, tree, 0);
    tree_levels
}

fn tree_to_levels_impl<'tree>(
    levels: &mut Vec<Vec<&'tree ExprNode>>,
    subtree: &'tree Tree<ExprNode>,
    depth: usize,
) {
    fn push_value<'tree>(
        levels: &mut Vec<Vec<&'tree ExprNode>>,
        value: &'tree ExprNode,
        depth: usize,
    ) {
        if depth >= levels.len() {
            levels.push(vec![value]);
        } else {
            levels[depth].push(value);
        }
    }

    fn push_node<'tree>(
        levels: &mut Vec<Vec<&'tree ExprNode>>,
        node: &'tree Node<ExprNode>,
        depth: usize,
    ) {
        match node {
            Node::Arm(value, _, child) => {
                push_value(levels, value, depth);
                push_node(levels, child, depth + 1);
            }
            Node::Tree(value, _, next) => {
                push_value(levels, value, depth);
                tree_to_levels_impl(levels, next, depth + 1);
            }
            Node::Leaf(value) => push_value(levels, value, depth),
        }
    }

    for node in &subtree.nodes {
        push_node(levels, node, depth);
    }
}
