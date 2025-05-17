use std::{
    cell::{RefCell, RefMut},
    hash::Hash,
};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use rand::{
    distributions::Alphanumeric,
    rngs::{SmallRng, ThreadRng},
    Rng, SeedableRng,
};
use syn::{
    parse::{Parse, ParseStream},
    Ident, Path, Result,
};

// Node Pseudo-Random Number Generator /////////////////////////////////////////

thread_local! {
    pub static NODE_RNG: NodeRng =
        NodeRng(RefCell::new(SmallRng::from_rng(ThreadRng::default()).unwrap()));
}

pub struct NodeRng(RefCell<SmallRng>);

impl NodeRng {
    pub fn get(&self) -> RefMut<SmallRng> {
        self.0.borrow_mut()
    }

    pub fn reseed(&self, seed: u64) {
        self.0.replace(SmallRng::seed_from_u64(seed));
    }
}

#[cfg(test)]
mod node_rng_tests {
    use rand::Rng;

    use super::NODE_RNG;

    fn reseed() {
        NODE_RNG.with(|rng| rng.reseed(0));
    }

    #[test]
    fn reseed_works() {
        reseed();
        let a = NODE_RNG.with(|rng| rng.get().gen::<u32>());
        reseed();
        let b = NODE_RNG.with(|rng| rng.get().gen::<u32>());
        assert_eq!(a, b);
    }
}

////////////////////////////////////////////////////////////////////////////////

// Node ////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq)]
pub struct SystemNode {
    system_path: Path,
    system_output_ident: Ident,
}

impl Hash for SystemNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.system_output_ident.hash(state);
    }
}

impl PartialOrd for SystemNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.system_output_ident.cmp(&other.system_output_ident))
    }
}

impl Ord for SystemNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.system_output_ident.cmp(&other.system_output_ident)
    }
}

impl From<Path> for SystemNode {
    fn from(path: Path) -> Self {
        SystemNode::new(path)
    }
}

impl SystemNode {
    pub fn new(system_path: Path) -> Self {
        let mut output = "__sysout__".to_string();
        NODE_RNG.with(|rng| {
            let rng = &mut *rng.get();
            let slug_iter = rng.sample_iter(Alphanumeric).take(6).map(char::from);
            output.extend(slug_iter);
        });
        output.push('_');
        for x in system_path.segments.iter() {
            output.push('_');
            output.push_str(&x.ident.to_string());
        }
        let system_output_ident = Ident::new(output.leak(), Span::call_site());
        Self { system_path, system_output_ident }
    }
}

impl Parse for SystemNode {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self::new(input.parse()?))
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for SystemNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output_ident = &self.system_output_ident;
        let output_ident = quote! { #output_ident };

        let system = &self.system_path;
        let system = quote! { #system };

        f.debug_tuple("Node").field(&output_ident).field(&system).finish()
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for SystemNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let output_ident = &self.system_output_ident;
        let output_ident = quote! { #output_ident };
        f.write_str(&output_ident.to_string())?;

        let system = &self.system_path;
        let system = quote! { #system };
        f.write_str(&system.to_string())?;

        std::fmt::Result::Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

// Runtime Statement ///////////////////////////////////////////////////////////

pub struct RuntimeStmt<'a> {
    pub parent: Option<&'a SystemNode>,
    pub value: &'a SystemNode,
}

impl ToTokens for RuntimeStmt<'_> {
    /// `let __sysout__abc123__step0 = world.run_system_once_with((), step0)?;`
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let output_ident = &self.value.system_output_ident;
        let parent_output_ident = match &self.parent {
            Some(parent_node) => parent_node.system_output_ident.to_token_stream(),
            None => quote! { () },
        };
        let system = &self.value.system_path;
        quote! { let #output_ident = world.run_system_once_with(#parent_output_ident, #system)?; }
            .to_tokens(tokens);
    }
}

////////////////////////////////////////////////////////////////////////////////
