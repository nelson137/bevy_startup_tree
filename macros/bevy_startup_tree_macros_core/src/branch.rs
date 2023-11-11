use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::Brace,
    Path, Result, Token,
};

use crate::{Node, Tree};

#[derive(PartialEq)]
pub enum Branch {
    Leaf(Node),
    Arm(Node, Token![=>], Box<Branch>),
    Tree(Node, Token![=>], Tree),
}

impl Branch {
    pub fn leaf(node: Node) -> Self {
        Self::Leaf(node)
    }

    pub fn arm(node: Node, child: Branch) -> Self {
        Self::Arm(node, Default::default(), Box::new(child))
    }

    pub fn tree(node: Node, child: Tree) -> Self {
        Self::Tree(node, Default::default(), child)
    }

    pub fn node(&self) -> &Node {
        match self {
            Self::Leaf(node) | Self::Arm(node, _, _) | Self::Tree(node, _, _) => node,
        }
    }

    pub fn sub_tree_mut(&mut self) -> Option<&mut Tree> {
        match self {
            Self::Tree(_, _, sub_tree) => Some(sub_tree),
            _ => None,
        }
    }
}

impl From<Path> for Branch {
    fn from(path: Path) -> Self {
        Self::leaf(Node::from(path))
    }
}

impl Parse for Branch {
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
impl std::fmt::Debug for Branch {
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
impl std::fmt::Display for Branch {
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
