use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Expr, ExprPath, Path, Result,
};

#[derive(PartialEq)]
pub struct Node(Expr);

impl Node {
    pub fn new(expr: Expr) -> Self {
        Self(expr)
    }

    pub fn as_into_descriptor_call(&self) -> TokenStream2 {
        let receiver = &self.0;
        quote! {
            ::bevy::prelude::IntoSystemConfigs::into_configs(#receiver)
        }
    }
}

impl From<Path> for Node {
    fn from(path: Path) -> Self {
        Node::new(Expr::Path(ExprPath { attrs: Vec::new(), qself: None, path }))
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

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::{startup_tree::Node, test_utils::path};

    #[test]
    fn node_correctly_creates_the_into_descriptor_call() {
        let node = Node::new(path!(sys));
        let expected_call =
            quote! { ::bevy::prelude::IntoSystemConfigs::into_configs(sys) }.to_string();
        let actual_call = node.as_into_descriptor_call().to_string();
        assert_eq!(actual_call, expected_call);
    }
}
