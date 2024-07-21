use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

use crate::tree::{Node, Tree};

mod node;

pub use self::node::{RuntimeStmt, SystemNode, NODE_RNG};

pub struct SystemTree(Tree<SystemNode>);

impl Parse for SystemTree {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl SystemTree {
    fn to_tokens_runtime_system_statements(&self) -> TokenStream2 {
        fn tree_to_tokens(
            stream: &mut TokenStream2,
            parent_node: Option<&SystemNode>,
            tree: &Tree<SystemNode>,
        ) {
            for node in &tree.nodes {
                node_to_tokens(stream, parent_node, node);
            }
        }

        fn node_to_tokens(
            stream: &mut TokenStream2,
            parent: Option<&SystemNode>,
            node: &Node<SystemNode>,
        ) {
            match node {
                Node::Leaf(value) => RuntimeStmt { parent, value }.to_tokens(stream),
                Node::Arm(value, _, next) => {
                    RuntimeStmt { parent, value }.to_tokens(stream);
                    node_to_tokens(stream, Some(value), next);
                }
                Node::Tree(value, _, next) => {
                    RuntimeStmt { parent, value }.to_tokens(stream);
                    tree_to_tokens(stream, Some(value), next);
                }
            }
        }

        let mut stream = TokenStream2::default();
        tree_to_tokens(&mut stream, None, &self.0);
        stream
    }
}

impl ToTokens for SystemTree {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let body = self.to_tokens_runtime_system_statements();

        quote! {
            #[allow(non_snake_case, clippy::let_unit_value)]
            |world: &mut ::bevy::ecs::world::World| {
                use ::bevy::ecs::system::RunSystemOnce;
                #body
            }
        }
        .to_tokens(tokens);
    }
}
