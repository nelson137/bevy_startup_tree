use syn::{
    braced,
    parse::{Parse, ParseStream},
    token,
    token::Brace,
    Path, Result, Token,
};

use super::Tree;

#[derive(PartialEq)]
pub enum Node<V> {
    Leaf(V),
    Arm(V, token::FatArrow, Box<Node<V>>),
    Tree(V, token::FatArrow, Tree<V>),
}

impl<V> Node<V> {
    pub fn leaf(value: V) -> Self {
        Self::Leaf(value)
    }

    pub fn arm(value: V, child: Node<V>) -> Self {
        Self::Arm(value, Default::default(), Box::new(child))
    }

    pub fn tree(value: V, child: Tree<V>) -> Self {
        Self::Tree(value, Default::default(), child)
    }

    pub fn value(&self) -> &V {
        match self {
            Self::Leaf(value) | Self::Arm(value, _, _) | Self::Tree(value, _, _) => value,
        }
    }

    pub fn sub_tree_mut(&mut self) -> Option<&mut Tree<V>> {
        match self {
            Self::Tree(_, _, sub_tree) => Some(sub_tree),
            _ => None,
        }
    }
}

impl From<Path> for Node<crate::startup_tree::ExprNode> {
    fn from(path: Path) -> Self {
        Self::leaf(crate::startup_tree::ExprNode::from(path))
    }
}

impl From<Path> for Node<crate::system_tree::SystemNode> {
    fn from(path: Path) -> Self {
        Self::leaf(crate::system_tree::SystemNode::from(path))
    }
}

impl<V: Parse> Parse for Node<V> {
    fn parse(input: ParseStream) -> Result<Self> {
        let value = input.parse()?;

        Ok(if input.peek(Token![=>]) {
            let fat_arrow_token = input.parse()?;
            if input.peek(Brace) {
                let brace_contents;
                braced!(brace_contents in input);
                Self::Tree(value, fat_arrow_token, brace_contents.call(Tree::parse)?)
            } else {
                Self::Arm(value, fat_arrow_token, input.parse()?)
            }
        } else {
            Self::Leaf(value)
        })
    }
}

#[cfg(debug_assertions)]
impl<V: std::fmt::Debug> std::fmt::Debug for Node<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[derive(Debug)]
        struct FatArrow;
        match self {
            Node::Leaf(node) => f.debug_tuple("Node::Leaf").field(node).finish(),
            Node::Arm(node, _, child) => {
                f.debug_tuple("Node::Arm").field(node).field(&FatArrow).field(child).finish()
            }
            Node::Tree(node, _, child) => {
                f.debug_tuple("Node::Tree").field(node).field(&FatArrow).field(child).finish()
            }
        }
    }
}

#[cfg(debug_assertions)]
impl<V: std::fmt::Display> std::fmt::Display for Node<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.value(), f)?;

        match self {
            Node::Leaf(_) => {}
            Node::Arm(_, _, child) => {
                f.write_str(" => ")?;
                std::fmt::Display::fmt(child, f)?;
            }
            Node::Tree(_, _, child) => {
                f.write_str(" => ")?;
                std::fmt::Display::fmt(&child.display(), f)?;
            }
        }

        Ok(())
    }
}
