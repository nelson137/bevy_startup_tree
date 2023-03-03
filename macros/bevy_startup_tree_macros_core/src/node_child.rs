use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::Brace,
    Result,
};

use crate::{Branch, Tree};

#[derive(PartialEq)]
pub enum NodeChild {
    Branch(Box<Branch>),
    Tree(Tree),
}

impl NodeChild {
    pub fn branch(branch: Branch) -> Self {
        Self::Branch(Box::new(branch))
    }
}

impl Parse for NodeChild {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.peek(Brace) {
            let brace_contents;
            braced!(brace_contents in input);
            NodeChild::Tree(brace_contents.call(Tree::parse)?)
        } else {
            NodeChild::Branch(input.parse()?)
        })
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for NodeChild {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Branch(branch) => f.debug_tuple("Branch").field(branch).finish(),
            Self::Tree(tree) => f.debug_tuple("Tree").field(tree).finish(),
        }
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for NodeChild {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Branch(branch) => std::fmt::Display::fmt(&branch, f),
            Self::Tree(tree) => std::fmt::Display::fmt(&tree, f),
        }
    }
}
