use bevy_startup_tree_macros_core::{
    system_tree::{Node, SystemTree, NODE_RNG},
    tree::{self, Branch},
};
use quote::quote;
use syn::parse2;

mod utils;

use self::utils::{assert_result, path};

type Tree = tree::Tree<Node>;

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
    let expected = Tree::from(Branch::arm(
        path!(sys1).into(),
        Branch::arm(path!(sys2).into(), path!(sys3).into()),
    ));
    assert_eq!(tree, expected);
    Ok(())
}

#[test]
fn parse_complex_tree() -> syn::Result<()> {
    reseed_rng();
    let expected = Tree::from_iter([
        Branch::from(path!(s1a)),
        Branch::tree(
            Node::from(path!(s1b)),
            Tree::from_iter([
                Branch::arm(Node::from(path!(s2a)), Branch::from(path!(s3a))),
                Branch::tree(Node::from(path!(s2b)), Tree::from_iter([path!(s3b), path!(s3c)])),
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
fn parse_tree_branches_and_commas() -> syn::Result<()> {
    reseed_rng();
    // Expected cases that are `Err` have an array with a dummy node so that the
    // RNG lines up with the actual cases.
    let expected_cases: &[(&[Node], Result<Tree, &str>)] = &[
        // #1
        (&[], Ok(Tree::from(path!(sys1)))),
        // #2
        (&[], Ok(Tree::from_branch(Branch::from(path!(sys2)), true))),
        // #3
        (&[Node::from(path!(p))], Err("unexpected end of input, expected identifier")),
        // #4
        (
            &[],
            Ok(Tree::from_branch(
                Branch::arm(Node::from(path!(sys4)), Branch::from(path!(child))),
                false,
            )),
        ),
        // #5
        (
            &[],
            Ok(Tree::from_branch(
                Branch::arm(Node::from(path!(sys5)), Branch::from(path!(child))),
                true,
            )),
        ),
        // #6
        (
            &[],
            Ok(Tree::from_branch(
                Branch::arm(
                    Node::from(path!(sys6)),
                    Branch::arm(Node::from(path!(child1)), Branch::from(path!(child2))),
                ),
                false,
            )),
        ),
        // #7
        (
            &[],
            Ok(Tree::from_branch(
                Branch::arm(
                    Node::from(path!(sys7)),
                    Branch::arm(Node::from(path!(child1)), Branch::from(path!(child2))),
                ),
                true,
            )),
        ),
        // #8
        (
            &[],
            Ok(Tree::from_branch(
                Branch::tree(Node::from(path!(sys8)), Tree::from_path(path!(child), false)),
                false,
            )),
        ),
        // #9
        (
            &[],
            Ok(Tree::from_branch(
                Branch::tree(Node::from(path!(sys9)), Tree::from_path(path!(child), false)),
                true,
            )),
        ),
        // #10
        (
            &[],
            Ok(Tree::from_branch(
                Branch::tree(Node::from(path!(sys10)), Tree::from_path(path!(child), true)),
                true,
            )),
        ),
        // #11
        (&[Node::from(path!(p))], Err("expected `,`")),
        // #12
        (&[], Ok(Tree::from_iter([path!(sys12a), path!(sys12b)]))),
        // #13
        (
            &[],
            Ok(Tree::from_iter([
                Branch::arm(Node::from(path!(sys13a)), path!(child).into()),
                Branch::from(path!(sys13b)),
            ])),
        ),
        // #14
        (&[Node::from(path!(p))], Err("expected `,`")),
    ];

    reseed_rng();
    let actual_cases = [
        // #1
        quote! { sys1 },
        // #2
        quote! { sys2, },
        // #3
        quote! { sys3 => },
        // #4
        quote! { sys4 => child },
        // #5
        quote! { sys5 => child, },
        // #6
        quote! { sys6 => child1 => child2 },
        // #7
        quote! { sys7 => child1 => child2, },
        // #8
        quote! { sys8 => { child } },
        // #9
        quote! { sys9 => { child }, },
        // #10
        quote! { sys10 => { child, }, },
        // #11
        quote! { sys11a sys11b },
        // #12
        quote! { sys12a, sys12b },
        // #13
        quote! { sys13a => child, sys13b },
        // #14
        quote! { sys14a => child sys14b },
    ];

    for (tokens, (_, expected)) in actual_cases.into_iter().zip(expected_cases) {
        let actual = parse2(tokens);
        assert_result(&actual, expected);
    }

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
