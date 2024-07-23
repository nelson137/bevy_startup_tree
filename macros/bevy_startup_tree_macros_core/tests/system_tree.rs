use bevy_startup_tree_macros_core::{
    system_tree::{SystemNode, SystemTree, NODE_RNG},
    tree,
};
use quote::quote;
use syn::parse2;

#[allow(dead_code)]
mod utils;

use self::utils::path;

type Tree = tree::Tree<SystemNode>;
type Node = tree::Node<SystemNode>;

fn reseed_rng() {
    NODE_RNG.with(|rng| rng.reseed(0));
}

#[test]
fn parse_tree_with_one_node() -> syn::Result<()> {
    reseed_rng();
    let tree: Tree = parse2(quote! { sys })?;
    reseed_rng();
    assert_eq!(tree, Tree::from(path!(sys)));
    Ok(())
}

#[test]
fn parse_tree_with_long_branch() -> syn::Result<()> {
    reseed_rng();
    let tree: Tree = parse2(quote! { sys1 => sys2 => sys3 })?;
    reseed_rng();
    let expected = Tree::from(Node::arm(
        path!(sys1).into(),
        Node::arm(path!(sys2).into(), path!(sys3).into()),
    ));
    assert_eq!(tree, expected);
    Ok(())
}

#[test]
fn parse_complex_tree() -> syn::Result<()> {
    reseed_rng();
    let expected = Tree::from_iter([
        Node::from(path!(s1a)),
        Node::tree(
            SystemNode::from(path!(s1b)),
            Tree::from_iter([
                Node::arm(path!(s2a).into(), path!(s3a).into()),
                Node::tree(path!(s2b).into(), Tree::from_iter([path!(s3b), path!(s3c)])),
            ]),
        ),
    ]);

    reseed_rng();
    let actual: Tree = parse2(quote! {
        s1a,
        s1b => {
            s2a => s3a,
            s2b => {
                s3b,
                s3c
            }
        }
    })?;

    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn tokenize_tree() {
    reseed_rng();
    let tree: SystemTree = parse2(quote! {
        s1a,
        s1b => {
            s2a => s3a,
            s2b => {
                s3b,
                s3c => s4a => s5a,
            },
        },
    })
    .expect("failed to arrange for test");

    let expected = quote! {
        #[allow(non_snake_case, clippy::let_unit_value)]
        |world: &mut ::bevy::ecs::world::World| {
            use ::bevy::ecs::system::RunSystemOnce;
            let __sysout__ccd58l__s1a = world.run_system_once_with((), s1a);
            let __sysout__dZ0PpD__s1b = world.run_system_once_with((), s1b);
            let __sysout__6OOTyu__s2a = world.run_system_once_with(__sysout__dZ0PpD__s1b, s2a);
            let __sysout__lLQ3Y5__s3a = world.run_system_once_with(__sysout__6OOTyu__s2a, s3a);
            let __sysout__C0b2AW__s2b = world.run_system_once_with(__sysout__dZ0PpD__s1b, s2b);
            let __sysout__7SXSra__s3b = world.run_system_once_with(__sysout__C0b2AW__s2b, s3b);
            let __sysout__BqyWIc__s3c = world.run_system_once_with(__sysout__C0b2AW__s2b, s3c);
            let __sysout__sWVb0o__s4a = world.run_system_once_with(__sysout__BqyWIc__s3c, s4a);
            let __sysout__GF7MJv__s5a = world.run_system_once_with(__sysout__sWVb0o__s4a, s5a);
        }
    }
    .to_string();

    let actual = quote! { #tree }.to_string();

    assert_eq!(actual, expected);
}
