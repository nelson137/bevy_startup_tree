use std::ops::{Add, AddAssign};

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Bracket,
    Error, Macro, MacroDelimiter, Path, PathSegment, Result, Token,
};

use crate::{Branch, Node};

pub struct StartupTree(Tree);

impl Parse for StartupTree {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut tree: Tree = input.parse()?;
        tree.set_depth_root();
        Ok(Self(tree))
    }
}

impl ToTokens for StartupTree {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let tree_levels = tree_to_levels(&self.0);
        let span = Span::call_site();

        let vec_elements = tree_levels.into_iter().map(|level| Macro {
            path: Path {
                leading_colon: Some(Token![::](span)),
                segments: Punctuated::from_iter([
                    PathSegment::from(Ident::new("std", span)),
                    PathSegment::from(Ident::new("vec", span)),
                ]),
            },
            bang_token: Token![!](span),
            delimiter: MacroDelimiter::Bracket(Bracket(span)),
            tokens: {
                let mut elements = TokenStream2::new();
                elements.append_separated(
                    level.into_iter().map(Node::as_into_descriptor_call),
                    Token![,](span),
                );
                elements
            },
        });
        let vec_elements = Punctuated::<_, Token![,]>::from_iter(vec_elements);

        quote! {
            vec![ #vec_elements ]
        }
        .to_tokens(tokens);
    }
}

fn tree_to_levels(tree: &Tree) -> Vec<Vec<&Node>> {
    let mut tree_levels: Vec<Vec<&Node>> = Vec::new();
    tree_to_levels_impl(&mut tree_levels, tree, 0);
    tree_levels
}

fn tree_to_levels_impl<'tree>(
    tree_levels: &mut Vec<Vec<&'tree Node>>,
    subtree: &'tree Tree,
    depth: usize,
) {
    fn push_branch<'tree>(levels: &mut Vec<Vec<&'tree Node>>, branch: &'tree Branch, depth: usize) {
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

#[derive(PartialEq)]
pub struct Tree {
    pub depth: TreeDepth,
    pub branches: Punctuated<Branch, Token![,]>,
}

impl Tree {
    pub fn new(depth: TreeDepth, branches: Vec<Branch>) -> Self {
        Self { depth, branches: Punctuated::from_iter(branches) }
    }

    pub fn from_branches(branches: Vec<Branch>, trailing_comma: bool) -> Self {
        let mut branches = Punctuated::from_iter(branches);
        if trailing_comma {
            branches.push_punct(Default::default());
        }
        Self { depth: TreeDepth::default(), branches }
    }

    pub fn from_branch(branch: Branch, trailing_comma: bool) -> Self {
        Self::from_branches(vec![branch], trailing_comma)
    }

    pub fn from_node(node: Node, trailing_comma: bool) -> Self {
        Self::from_branch(Branch::Leaf(node), trailing_comma)
    }

    pub fn from_path(path: Path, trailing_comma: bool) -> Self {
        Self::from_branch(path.into(), trailing_comma)
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

impl<B: Into<Branch>> FromIterator<B> for Tree {
    fn from_iter<T: IntoIterator<Item = B>>(iter: T) -> Self {
        let branches = iter.into_iter().map(Into::into).collect();
        Self::from_branches(branches, false)
    }
}

impl From<Branch> for Tree {
    fn from(branch: Branch) -> Self {
        Self::from_branch(branch, false)
    }
}

impl From<Node> for Tree {
    fn from(node: Node) -> Self {
        Self::from_node(node, false)
    }
}

impl From<Path> for Tree {
    fn from(path: Path) -> Self {
        Self::from_path(path, false)
    }
}

impl Parse for Tree {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(input.span(), "tree may not be empty"));
        }
        Ok(Self { depth: TreeDepth::default(), branches: Punctuated::parse_terminated(input)? })
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("depth", &self.depth)
            .field("branches", &self.branches)
            .finish()
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for Tree {
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
mod tests {
    use std::ops::{Add, AddAssign};

    use proc_macro2::TokenStream as TokenStream2;
    use rand::random;
    use syn::parse2;

    use crate::{test_utils::assert_err, Tree, TreeDepth};

    #[test]
    fn error_on_empty_tree() {
        let result = parse2::<Tree>(TokenStream2::new());
        assert_err(&result, "tree may not be empty");
    }

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
