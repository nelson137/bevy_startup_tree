use syn::{
    braced,
    parse::{Parse, ParseStream},
    token,
    token::Brace,
    Path, Result, Token,
};

use super::Tree;

#[derive(PartialEq)]
pub enum Branch<N> {
    Leaf(N),
    Arm(N, token::FatArrow, Box<Branch<N>>),
    Tree(N, token::FatArrow, Tree<N>),
}

impl<N> Branch<N> {
    pub fn leaf(node: N) -> Self {
        Self::Leaf(node)
    }

    pub fn arm(node: N, child: Branch<N>) -> Self {
        Self::Arm(node, Default::default(), Box::new(child))
    }

    pub fn tree(node: N, child: Tree<N>) -> Self {
        Self::Tree(node, Default::default(), child)
    }

    pub fn node(&self) -> &N {
        match self {
            Self::Leaf(node) | Self::Arm(node, _, _) | Self::Tree(node, _, _) => node,
        }
    }

    pub fn sub_tree_mut(&mut self) -> Option<&mut Tree<N>> {
        match self {
            Self::Tree(_, _, sub_tree) => Some(sub_tree),
            _ => None,
        }
    }
}

impl From<Path> for Branch<crate::startup_tree::Node> {
    fn from(path: Path) -> Self {
        Self::leaf(crate::startup_tree::Node::from(path))
    }
}

impl From<Path> for Branch<crate::system_tree::Node> {
    fn from(path: Path) -> Self {
        Self::leaf(crate::system_tree::Node::from(path))
    }
}

impl<N: Parse> Parse for Branch<N> {
    fn parse(input: ParseStream) -> Result<Self> {
        let node = input.parse()?;

        Ok(if input.peek(Token![=>]) {
            let fat_arrow_token = input.parse()?;
            if input.peek(Brace) {
                let brace_contents;
                braced!(brace_contents in input);
                Self::Tree(node, fat_arrow_token, brace_contents.call(Tree::parse)?)
            } else {
                Self::Arm(node, fat_arrow_token, Box::new(input.parse()?))
            }
        } else {
            Self::Leaf(node)
        })
    }
}

#[cfg(debug_assertions)]
impl<N: std::fmt::Debug> std::fmt::Debug for Branch<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[derive(Debug)]
        struct FatArrow;
        match self {
            Branch::Leaf(node) => f.debug_tuple("Branch::Leaf").field(node).finish(),
            Branch::Arm(node, _, child) => {
                f.debug_tuple("Branch::Arm").field(node).field(&FatArrow).field(child).finish()
            }
            Branch::Tree(node, _, child) => {
                f.debug_tuple("Branch::Tree").field(node).field(&FatArrow).field(child).finish()
            }
        }
    }
}

#[cfg(debug_assertions)]
impl<N: std::fmt::Display> std::fmt::Display for Branch<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.node(), f)?;

        match self {
            Branch::Leaf(_) => {}
            Branch::Arm(_, _, child) => {
                f.write_str(" => ")?;
                std::fmt::Display::fmt(child, f)?;
            }
            Branch::Tree(_, _, child) => {
                f.write_str(" => ")?;
                std::fmt::Display::fmt(child, f)?;
            }
        }

        Ok(())
    }
}
