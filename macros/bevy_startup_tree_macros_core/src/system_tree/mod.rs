use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

use crate::tree::{Branch, Tree};

mod node;

pub use self::node::{Node, RuntimeStmt, NODE_RNG};

pub struct SystemTree(Tree<Node>);

impl Parse for SystemTree {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl SystemTree {
    fn to_tokens_runtime_system_statements(&self) -> TokenStream2 {
        fn tree_to_tokens(
            stream: &mut TokenStream2,
            parent_node: Option<&Node>,
            tree: &Tree<Node>,
        ) {
            for branch in &tree.branches {
                branch_to_tokens(stream, parent_node, branch);
            }
        }

        fn branch_to_tokens(
            stream: &mut TokenStream2,
            parent_node: Option<&Node>,
            branch: &Branch<Node>,
        ) {
            match branch {
                Branch::Leaf(node) => RuntimeStmt { parent_node, node }.to_tokens(stream),
                Branch::Arm(node, _, next_branch) => {
                    RuntimeStmt { parent_node, node }.to_tokens(stream);
                    branch_to_tokens(stream, Some(node), next_branch);
                }
                Branch::Tree(node, _, next_tree) => {
                    RuntimeStmt { parent_node, node }.to_tokens(stream);
                    tree_to_tokens(stream, Some(node), next_tree);
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
