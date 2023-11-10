use syn::{
    parse::{Parse, ParseStream},
    Path, Result, Token,
};

use crate::{Node, NodeChild};

#[derive(PartialEq)]
pub struct Branch {
    pub node: Node,
    pub child: Option<(Token![=>], NodeChild)>,
}

impl Branch {
    pub fn new(node: Node, child: Option<NodeChild>) -> Self {
        Self { node, child: child.map(|c| (Default::default(), c)) }
    }

    pub fn from_node(node: Node) -> Self {
        Self::new(node, None)
    }

    pub fn from_path(path: Path) -> Self {
        Self::from_node(Node::new(path))
    }
}

impl From<Node> for Branch {
    fn from(node: Node) -> Self {
        Self::from_node(node)
    }
}

impl From<Path> for Branch {
    fn from(path: Path) -> Self {
        Self::from_path(path)
    }
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

        Ok(Self { node, child })
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[derive(Debug)]
        struct FatArrow;
        f.debug_struct("Branch")
            .field("node", &self.node)
            .field("child", &self.child.as_ref().map(|(_, child)| (FatArrow, child)))
            .finish()
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.node, f)?;

        if let Some((_, child)) = &self.child {
            f.write_str(" => ")?;
            std::fmt::Display::fmt(child, f)?;
        }

        Ok(())
    }
}
