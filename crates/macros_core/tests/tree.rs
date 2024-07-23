use bevy_startup_tree_macros_core::tree::{self, Node};
use bevy_startup_tree_test_utils::assert_result;
use quote::quote;
use syn::parse2;

type Tree = tree::Tree<syn::LitInt>;

macro_rules! lit {
    ($value:literal) => {
        syn::LitInt::new(stringify!($value), proc_macro2::Span::call_site())
    };
}

#[test]
fn parse_tree_with_one_node() -> syn::Result<()> {
    let tree: Tree = parse2(quote! { 1 })?;
    assert_eq!(tree, Tree::from(lit!(1)));
    Ok(())
}

#[test]
fn parse_tree_with_long_branch() -> syn::Result<()> {
    let tree: Tree = parse2(quote! { 1 => 2 => 3 })?;
    let expected = Tree::from(Node::arm(lit!(1), Node::arm(lit!(2), lit!(3).into())));
    assert_eq!(tree, expected);
    Ok(())
}

#[test]
fn parse_complex_tree() -> syn::Result<()> {
    let expected = Tree::from_iter([
        lit!(101).into(),
        Node::tree(
            lit!(102),
            Tree::from_iter([
                Node::arm(lit!(201), lit!(301).into()),
                Node::tree(lit!(202), Tree::from_iter([lit!(302), lit!(303)])),
            ]),
        ),
    ]);

    let actual: Tree = parse2(quote! {
        101,
        102 => {
            201 => 301,
            202 => {
                302,
                303
            }
        }
    })?;

    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn parse_tree_nodes_and_commas() -> syn::Result<()> {
    let cases = [
        (quote! { 1 }, Ok(Tree::from(lit!(1)))),
        (quote! { 2, }, Ok(Tree::from_node(lit!(2).into(), true))),
        (quote! { 3 => }, Err("unexpected end of input, expected integer literal")),
        (quote! { 4 => 0 }, Ok(Tree::from_node(Node::arm(lit!(4), lit!(0).into()), false))),
        (quote! { 5 => 0, }, Ok(Tree::from_node(Node::arm(lit!(5), lit!(0).into()), true))),
        (
            quote! { 6 => 0 => 1 },
            Ok(Tree::from_node(Node::arm(lit!(6), Node::arm(lit!(0), lit!(1).into())), false)),
        ),
        (
            quote! { 7 => 0 => 1, },
            Ok(Tree::from_node(Node::arm(lit!(7), Node::arm(lit!(0), lit!(1).into())), true)),
        ),
        (
            quote! { 8 => { 0 } },
            Ok(Tree::from_node(Node::tree(lit!(8), Tree::from_value(lit!(0), false)), false)),
        ),
        (
            quote! { 9 => { 0 }, },
            Ok(Tree::from_node(Node::tree(lit!(9), Tree::from_value(lit!(0), false)), true)),
        ),
        (
            quote! { 10 => { 0, }, },
            Ok(Tree::from_node(Node::tree(lit!(10), Tree::from_value(lit!(0), true)), true)),
        ),
        (quote! { 11 12 }, Err("expected `,`")),
        (quote! { 13, 14 }, Ok(Tree::from_iter([lit!(13), lit!(14)]))),
        (
            quote! { 15 => 0, 16 },
            Ok(Tree::from_iter([Node::arm(lit!(15), lit!(0).into()), lit!(16).into()])),
        ),
        (quote! { 17 => 0 18 }, Err("expected `,`")),
    ];

    for (tokens, expected) in cases {
        let actual = parse2(tokens);
        assert_result(&actual, &expected);
    }

    Ok(())
}
