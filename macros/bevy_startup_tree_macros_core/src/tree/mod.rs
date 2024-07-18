use std::ops::{Add, AddAssign};

use syn::{
    parse::{Parse, ParseStream},
    punctuated as punct, token, Error, Path, Result,
};

mod branch;

pub use self::branch::*;

// Tree ////////////////////////////////////////////////////////////////////////

#[derive(PartialEq)]
pub struct Tree<N> {
    pub depth: TreeDepth,
    pub branches: punct::Punctuated<Branch<N>, token::Comma>,
}

impl<N> Tree<N> {
    pub fn new(depth: TreeDepth, branches: Vec<Branch<N>>) -> Self {
        Self { depth, branches: punct::Punctuated::from_iter(branches) }
    }

    pub fn from_branches(branches: Vec<Branch<N>>, trailing_comma: bool) -> Self {
        let mut branches = punct::Punctuated::from_iter(branches);
        if trailing_comma {
            branches.push_punct(Default::default());
        }
        Self { depth: TreeDepth::default(), branches }
    }

    pub fn from_branch(branch: Branch<N>, trailing_comma: bool) -> Self {
        Self::from_branches(vec![branch], trailing_comma)
    }

    pub fn from_node(node: N, trailing_comma: bool) -> Self {
        Self::from_branch(Branch::Leaf(node), trailing_comma)
    }

    fn _calculate_depths_impl(this: &mut Self, depth: TreeDepth) {
        this.depth = depth;
        for branch in &mut this.branches {
            if let Some(b_child_tree) = branch.sub_tree_mut() {
                Self::_calculate_depths_impl(b_child_tree, depth + 1);
            }
        }
    }

    pub fn set_depth_root(&mut self) {
        Self::_calculate_depths_impl(self, TreeDepth::default());
    }
}

impl Tree<crate::startup_tree::Node> {
    pub fn from_path(path: Path, trailing_comma: bool) -> Self {
        Self::from_branch(path.into(), trailing_comma)
    }
}

impl Tree<crate::system_tree::Node> {
    pub fn from_path(path: Path, trailing_comma: bool) -> Self {
        Self::from_branch(path.into(), trailing_comma)
    }
}

impl<N, B: Into<Branch<N>>> FromIterator<B> for Tree<N> {
    fn from_iter<T: IntoIterator<Item = B>>(iter: T) -> Self {
        let branches = iter.into_iter().map(Into::into).collect();
        Self::from_branches(branches, false)
    }
}

impl<N> From<Branch<N>> for Tree<N> {
    fn from(branch: Branch<N>) -> Self {
        Self::from_branch(branch, false)
    }
}

impl<N> From<N> for Tree<N> {
    fn from(node: N) -> Self {
        Self::from_node(node, false)
    }
}

impl From<Path> for Tree<crate::startup_tree::Node> {
    fn from(path: Path) -> Self {
        Self::from_path(path, false)
    }
}

impl From<Path> for Tree<crate::system_tree::Node> {
    fn from(path: Path) -> Self {
        Self::from_path(path, false)
    }
}

impl<N: Parse> Parse for Tree<N> {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(input.span(), "tree may not be empty"));
        }
        Ok(Self {
            depth: TreeDepth::default(),
            branches: punct::Punctuated::parse_terminated(input)?,
        })
    }
}

#[cfg(debug_assertions)]
impl<N: std::fmt::Debug> std::fmt::Debug for Tree<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("depth", &self.depth)
            .field("branches", &self.branches)
            .finish()
    }
}

#[cfg(debug_assertions)]
impl<N: std::fmt::Display> std::fmt::Display for Tree<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Write;
        if self.branches.is_empty() {
            f.write_str("{}")
        } else {
            f.write_str("{\n")?;
            for branch in &self.branches {
                std::fmt::Display::fmt(&(self.depth + 1), f)?;
                std::fmt::Display::fmt(branch, f)?;
                f.write_char('\n')?;
            }
            std::fmt::Display::fmt(&self.depth, f)?;
            f.write_char('}')
        }
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

// Tree Depth //////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default, PartialEq)]
pub struct TreeDepth(pub u32);

impl Add<u32> for TreeDepth {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0.add(rhs))
    }
}

impl AddAssign<u32> for TreeDepth {
    fn add_assign(&mut self, rhs: u32) {
        self.0.add_assign(rhs);
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for TreeDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match f.alternate() {
            true => write!(f, "{}", self.0),
            false => f.debug_tuple("TreeDepth").field(&self.0).finish(),
        }
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for TreeDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for _ in 0..self.0 {
            f.write_str("    ")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tree_depth_tests {
    use std::ops::{Add, AddAssign};

    use rand::random;

    use super::TreeDepth;

    fn safe_random_tree_depth() -> (u32, TreeDepth) {
        let value = random::<u32>();
        // Subtract 1 to guarantee that adding 1 won't overflow
        (value, TreeDepth(value.saturating_sub(1)))
    }

    #[test]
    fn tree_depth_adds_one() {
        let (value, depth) = safe_random_tree_depth();
        assert_eq!(Add::add(depth, 1).0, value);
    }

    #[test]
    fn tree_depth_add_assigns_one() {
        let (value, mut depth) = safe_random_tree_depth();
        AddAssign::add_assign(&mut depth, 1);
        assert_eq!(depth.0, value);
    }
}

////////////////////////////////////////////////////////////////////////////////
