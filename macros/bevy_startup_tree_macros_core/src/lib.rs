use std::ops::{Add, AddAssign};

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Bracket},
    Error, ExprPath, Macro, MacroDelimiter, Path, PathSegment, Result, Token,
};

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
    fn push_node<'tree>(levels: &mut Vec<Vec<&'tree Node>>, node: &'tree Node, depth: usize) {
        if depth >= levels.len() {
            levels.push(vec![node]);
        } else {
            levels[depth].push(node);
        }
    }

    for branch in &subtree.branches {
        push_node(tree_levels, &branch.node, depth);
        if let Some((_, child)) = &branch.child {
            match child {
                NodeChild::Node(node) => push_node(tree_levels, node, depth + 1),
                NodeChild::Tree(tree) => tree_to_levels_impl(tree_levels, tree, depth + 1),
            }
        }
    }
}

struct Tree {
    depth: TreeDepth,
    branches: Vec<Branch>,
}

impl Tree {
    fn _calculate_depths_impl(this: &mut Self, depth: TreeDepth) {
        this.depth = depth;
        for branch in &mut this.branches {
            if let Some((_, NodeChild::Tree(b_child_tree))) = branch.child.as_mut() {
                Self::_calculate_depths_impl(b_child_tree, depth + 1);
            }
        }
    }

    pub fn set_depth_root(&mut self) {
        Self::_calculate_depths_impl(self, TreeDepth::default());
    }
}

impl Parse for Tree {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(input.span(), "subtree may not be empty"));
        }

        let mut branches = Vec::new();
        while !input.is_empty() {
            branches.push(input.parse()?);
        }

        Ok(Self { depth: TreeDepth::default(), branches })
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

#[derive(Clone, Copy, Default)]
struct TreeDepth(u32);

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
        f.debug_tuple("TreeDepth").field(&self.0).finish()
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

struct Branch {
    node: Node,
    child: Option<(Token![=>], NodeChild)>,
    #[allow(dead_code)]
    comma_token: Option<Token![,]>,
}

impl Parse for Branch {
    fn parse(input: ParseStream) -> Result<Self> {
        let node = input.parse()?;

        let child = if input.peek(Token![=>]) {
            let fat_arrow_token = input.parse()?;
            let node_child = input.parse()?;
            Some((fat_arrow_token, node_child))
        } else {
            None
        };

        let comma_token = input.parse().ok();

        Ok(Self { node, child, comma_token })
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[derive(Debug)]
        struct FatArrow;
        #[derive(Debug)]
        struct Comma;
        f.debug_struct("Branch")
            .field("node", &self.node)
            .field("child", &self.child.as_ref().map(|(_, child)| (FatArrow, child)))
            .field("comma_token", &self.comma_token.and(Some(Comma)))
            .finish()
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Write;

        std::fmt::Display::fmt(&self.node, f)?;

        if let Some((_, child)) = &self.child {
            f.write_str(" => ")?;
            std::fmt::Display::fmt(child, f)?;
        }

        if self.comma_token.is_some() {
            f.write_char(',')?;
        }

        Ok(())
    }
}

struct Node(ExprPath);

impl Node {
    fn as_into_descriptor_call(&self) -> TokenStream2 {
        let receiver = &self.0;
        quote! {
            ::bevy::prelude::IntoSystemDescriptor::into_descriptor(#receiver)
        }
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let path = &self.0;
        let path = quote! { #path };
        f.debug_tuple("Node").field(&path).finish()
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let path = &self.0;
        let path = quote! { #path };
        f.write_str(&path.to_string())
    }
}

enum NodeChild {
    Node(Node),
    Tree(Tree),
}

impl Parse for NodeChild {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.peek(Brace) {
            let brace_contents;
            braced!(brace_contents in input);
            NodeChild::Tree(brace_contents.call(Tree::parse)?)
        } else {
            NodeChild::Node(input.parse()?)
        })
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for NodeChild {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Node(node) => f.debug_tuple("Node").field(node).finish(),
            Self::Tree(tree) => f.debug_tuple("Tree").field(tree).finish(),
        }
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for NodeChild {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Node(node) => std::fmt::Display::fmt(&node, f),
            Self::Tree(tree) => std::fmt::Display::fmt(&tree, f),
        }
    }
}
