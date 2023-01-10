use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    LitInt, LitStr, Result, Token,
};

pub struct StageLabelGenerator(u32);

impl Parse for StageLabelGenerator {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse::<LitInt>()?.base10_parse()?))
    }
}

impl ToTokens for StageLabelGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let span = Span::call_site();
        let labels: Vec<_> = (0..self.0).map(|i| format!("__startup_tree_stage_{i}")).collect();
        let label_lits = labels.iter().map(|l| LitStr::new(l, span));
        let elements = Punctuated::<_, Token![,]>::from_iter(label_lits);
        quote! {
            [ #elements ]
        }
        .to_tokens(tokens);
    }
}
