use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Expr, ExprPath, Path, Result,
};

#[derive(PartialEq)]
pub struct ExprNode(Expr);

impl ExprNode {
    pub fn new(expr: Expr) -> Self {
        Self(expr)
    }

    pub fn as_into_descriptor_call(&self) -> TokenStream2 {
        let receiver = &self.0;
        quote! {
            ::bevy::prelude::IntoScheduleConfigs::into_configs(#receiver)
        }
    }
}

impl From<Path> for ExprNode {
    fn from(path: Path) -> Self {
        ExprNode::new(Expr::Path(ExprPath { attrs: Vec::new(), qself: None, path }))
    }
}

impl Parse for ExprNode {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for ExprNode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for ExprNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let path = &self.0;
        let path = quote! { #path };
        f.debug_tuple("Node").field(&path).finish()
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for ExprNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let path = &self.0;
        let path = quote! { #path };
        f.write_str(&path.to_string())
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use quote::quote;

    use super::*;

    #[test]
    fn node_correctly_creates_the_into_descriptor_call() {
        let node = ExprNode::from(syn::Path::from(syn::Ident::new("sys", Span::call_site())));
        let expected_call =
            quote! { ::bevy::prelude::IntoScheduleConfigs::into_configs(sys) }.to_string();
        let actual_call = node.as_into_descriptor_call().to_string();
        assert_eq!(actual_call, expected_call);
    }
}
