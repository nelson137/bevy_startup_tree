#[cfg(debug_assertions)]
use std::fmt;
use std::ops::{Add, AddAssign};

use syn::{
    parse::{Parse, ParseStream},
    punctuated as punct, token, Error, Path, Result,
};

mod node;

pub use self::node::*;

// Tree ////////////////////////////////////////////////////////////////////////

#[derive(PartialEq)]
pub struct Tree<V> {
    pub nodes: punct::Punctuated<Node<V>, token::Comma>,
}

impl<V> Tree<V> {
    pub fn new(nodes: Vec<Node<V>>) -> Self {
        Self { nodes: punct::Punctuated::from_iter(nodes) }
    }

    pub fn from_nodes(nodes: Vec<Node<V>>, trailing_comma: bool) -> Self {
        let mut nodes = punct::Punctuated::from_iter(nodes);
        if trailing_comma {
            nodes.push_punct(Default::default());
        }
        Self { nodes }
    }

    pub fn from_node(node: Node<V>, trailing_comma: bool) -> Self {
        Self::from_nodes(vec![node], trailing_comma)
    }

    pub fn from_value(value: V, trailing_comma: bool) -> Self {
        Self::from_node(Node::Leaf(value), trailing_comma)
    }
}

impl Tree<crate::startup_tree::ExprNode> {
    pub fn from_path(path: Path, trailing_comma: bool) -> Self {
        Self::from_node(path.into(), trailing_comma)
    }
}

impl Tree<crate::system_tree::SystemNode> {
    pub fn from_path(path: Path, trailing_comma: bool) -> Self {
        Self::from_node(path.into(), trailing_comma)
    }
}

impl<V, B: Into<Node<V>>> FromIterator<B> for Tree<V> {
    fn from_iter<T: IntoIterator<Item = B>>(iter: T) -> Self {
        let nodes = iter.into_iter().map(Into::into).collect();
        Self::from_nodes(nodes, false)
    }
}

impl<V> From<Node<V>> for Tree<V> {
    fn from(node: Node<V>) -> Self {
        Self::from_node(node, false)
    }
}

impl<V> From<V> for Tree<V> {
    fn from(node: V) -> Self {
        Self::from_value(node, false)
    }
}

impl From<Path> for Tree<crate::startup_tree::ExprNode> {
    fn from(path: Path) -> Self {
        Self::from_path(path, false)
    }
}

impl From<Path> for Tree<crate::system_tree::SystemNode> {
    fn from(path: Path) -> Self {
        Self::from_path(path, false)
    }
}

impl<V: Parse> Parse for Tree<V> {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(input.span(), "tree may not be empty"));
        }
        Ok(Self { nodes: punct::Punctuated::parse_terminated(input)? })
    }
}

#[cfg(debug_assertions)]
impl<V: fmt::Debug> fmt::Debug for Tree<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Tree").field("nodes", &self.nodes).finish()
    }
}

#[cfg(test)]
mod tree_tests {
    use proc_macro2::TokenStream as TokenStream2;
    use syn::{
        parse::{Parse, ParseStream},
        parse2, Result,
    };

    use crate::test_utils::assert_err;

    type Tree = super::Tree<N>;

    struct N;

    impl Parse for N {
        fn parse(_input: ParseStream) -> Result<Self> {
            Ok(Self)
        }
    }

    #[test]
    fn error_on_empty_tree() {
        let result = parse2::<Tree>(TokenStream2::new());
        assert_err(&result, "tree may not be empty");
    }
}

////////////////////////////////////////////////////////////////////////////////

// Tree Display ////////////////////////////////////////////////////////////////

#[cfg(debug_assertions)]
impl<V> Tree<V> {
    pub fn display(&self) -> TreeDisplay<V> {
        self.display_with_depth(Default::default())
    }

    fn display_with_depth(&self, depth: TreeDepth) -> TreeDisplay<V> {
        TreeDisplay { tree: self, depth }
    }
}

#[cfg(debug_assertions)]
pub struct TreeDisplay<'tree, V> {
    tree: &'tree Tree<V>,
    depth: TreeDepth,
}

#[cfg(debug_assertions)]
impl<V: fmt::Display> fmt::Display for TreeDisplay<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use fmt::Write;

        if self.tree.nodes.is_empty() {
            return f.write_str("{}");
        }

        f.write_str("{\n")?;
        let depth = self.depth + 1;

        for node in &self.tree.nodes {
            fmt::Display::fmt(&depth, f)?;
            fmt::Display::fmt(&NodeDisplay::new(node, depth), f)?;
            f.write_char('\n')?;
        }

        fmt::Display::fmt(&self.depth, f)?;
        f.write_char('}')?;

        Ok(())
    }
}

#[cfg(debug_assertions)]
struct NodeDisplay<'tree, V> {
    node: &'tree Node<V>,
    depth: TreeDepth,
}

impl<'tree, V> NodeDisplay<'tree, V> {
    fn new(node: &'tree Node<V>, depth: TreeDepth) -> Self {
        Self { node, depth }
    }
}

#[cfg(debug_assertions)]
impl<V: fmt::Display> fmt::Display for NodeDisplay<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node {
            Node::Leaf(value) => fmt::Display::fmt(value, f)?,
            Node::Arm(value, _, child) => {
                fmt::Display::fmt(value, f)?;
                f.write_str(" => ")?;
                fmt::Display::fmt(&Self::new(child.as_ref(), self.depth), f)?;
            }
            Node::Tree(value, _, child) => {
                fmt::Display::fmt(value, f)?;
                f.write_str(" => ")?;
                fmt::Display::fmt(&child.display_with_depth(self.depth), f)?;
            }
        }
        Ok(())
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Copy, Default, PartialEq)]
struct TreeDepth(u32);

#[cfg(debug_assertions)]
impl Add<u32> for TreeDepth {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0.add(rhs))
    }
}

#[cfg(debug_assertions)]
impl AddAssign<u32> for TreeDepth {
    fn add_assign(&mut self, rhs: u32) {
        self.0.add_assign(rhs);
    }
}

#[cfg(debug_assertions)]
impl fmt::Debug for TreeDepth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match f.alternate() {
            true => write!(f, "{}", self.0),
            false => f.debug_tuple("TreeDepth").field(&self.0).finish(),
        }
    }
}

#[cfg(debug_assertions)]
impl fmt::Display for TreeDepth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.0 {
            f.write_str("    ")?;
        }
        Ok(())
    }
}

#[cfg(all(test, debug_assertions))]
mod tree_display_tests {
    use std::ops::{Add, AddAssign};

    use rand::random;

    use super::TreeDepth;

    //
    // Tree Depth
    //

    fn safe_random_tree_depth() -> (u32, TreeDepth) {
        let value = random::<u32>();
        // Subtract 1 to guarantee that adding 1 won't overflow
        (value, TreeDepth(value.saturating_sub(1)))
    }

    #[test]
    fn tree_depth_adds_one() {
        let (expected, depth) = safe_random_tree_depth();
        assert_eq!(Add::add(depth, 1).0, expected);
    }

    #[test]
    fn tree_depth_add_assigns_one() {
        let (expected, mut depth) = safe_random_tree_depth();
        AddAssign::add_assign(&mut depth, 1);
        assert_eq!(depth.0, expected);
    }

    //
    // Tree Display
    //

    type Tree = super::Tree<u32>;
    type Node = super::Node<u32>;

    #[test]
    fn print() {
        let expected = "\
{
    101
    102 => {
        201 => 301
        202 => 302 => 401 => {
            501
        }
    }
}";

        let tree = Tree::from_iter([
            Node::leaf(101),
            Node::tree(
                102,
                Tree::from_iter([
                    Node::arm(201, Node::leaf(301)),
                    Node::arm(202, Node::arm(302, Node::tree(401, Tree::from(501)))),
                ]),
            ),
        ]);
        let actual = tree.display().to_string();

        assert_eq!(actual, expected);
    }
}

////////////////////////////////////////////////////////////////////////////////
