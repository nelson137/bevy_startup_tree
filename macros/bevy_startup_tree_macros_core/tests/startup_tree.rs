use bevy_startup_tree_macros_core::{
    startup_tree::{Node, StartupTree},
    tree::{self, Branch, TreeDepth},
};
use quote::quote;
use syn::parse2;

mod utils;

use self::utils::{assert_result, path};

type Tree = tree::Tree<Node>;

#[test]
fn parse_tree_with_one_node() -> syn::Result<()> {
    let tree: Tree = parse2(quote! { sys })?;
    assert_eq!(tree, Tree::from(path!(sys)));
    Ok(())
}

#[test]
fn parse_tree_with_long_branch() -> syn::Result<()> {
    let tree: Tree = parse2(quote! { sys1 => sys2 => sys3 })?;
    let expected = Tree::from(Branch::arm(
        path!(sys1).into(),
        Branch::arm(path!(sys2).into(), path!(sys3).into()),
    ));
    assert_eq!(tree, expected);
    Ok(())
}

#[test]
fn parse_complex_tree() -> syn::Result<()> {
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
    let cases = [
        (quote! { sys1 }, Ok(Tree::from(path!(sys1)))),
        (
            quote! { sys1.func(param) },
            Ok(Tree::from_branch(
                Branch::leaf(Node::new(parse2(quote! { sys1.func(param) })?)),
                false,
            )),
        ),
        (quote! { sys2, }, Ok(Tree::from_branch(Branch::from(path!(sys2)), true))),
        (quote! { sys3 => }, Err("unexpected end of input, expected an expression")),
        (
            quote! { sys4 => child },
            Ok(Tree::from_branch(
                Branch::arm(Node::from(path!(sys4)), Branch::from(path!(child))),
                false,
            )),
        ),
        (
            quote! { sys4.func(param) => child },
            Ok(Tree::from_branch(
                Branch::arm(
                    Node::new(parse2(quote! { sys4.func(param) })?),
                    Branch::from(path!(child)),
                ),
                false,
            )),
        ),
        (
            quote! { sys5 => child, },
            Ok(Tree::from_branch(
                Branch::arm(Node::from(path!(sys5)), Branch::from(path!(child))),
                true,
            )),
        ),
        (
            quote! { sys6 => child1 => child2 },
            Ok(Tree::from_branch(
                Branch::arm(
                    Node::from(path!(sys6)),
                    Branch::arm(Node::from(path!(child1)), Branch::from(path!(child2))),
                ),
                false,
            )),
        ),
        (
            quote! { sys7 => child1 => child2, },
            Ok(Tree::from_branch(
                Branch::arm(
                    Node::from(path!(sys7)),
                    Branch::arm(Node::from(path!(child1)), Branch::from(path!(child2))),
                ),
                true,
            )),
        ),
        (
            quote! { sys8 => { child } },
            Ok(Tree::from_branch(
                Branch::tree(Node::from(path!(sys8)), Tree::from_path(path!(child), false)),
                false,
            )),
        ),
        (
            quote! { sys8 => { child.func(param) } },
            Ok(Tree::from_branch(
                Branch::tree(
                    Node::from(path!(sys8)),
                    Tree::from_node(
                        Node::new(parse2(quote! { child.func(param) }).unwrap()),
                        false,
                    ),
                ),
                false,
            )),
        ),
        (
            quote! { sys9 => { child }, },
            Ok(Tree::from_branch(
                Branch::tree(Node::from(path!(sys9)), Tree::from_path(path!(child), false)),
                true,
            )),
        ),
        (
            quote! { sys10 => { child, }, },
            Ok(Tree::from_branch(
                Branch::tree(Node::from(path!(sys10)), Tree::from_path(path!(child), true)),
                true,
            )),
        ),
        (quote! { sys11a sys11b }, Err("expected `,`")),
        (quote! { sys12a, sys12b }, Ok(Tree::from_iter([path!(sys12a), path!(sys12b)]))),
        (
            quote! { sys13a => child, sys13b },
            Ok(Tree::from_iter([
                Branch::arm(Node::from(path!(sys13a)), path!(child).into()),
                Branch::from(path!(sys13b)),
            ])),
        ),
        (quote! { sys14a => child sys14b }, Err("expected `,`")),
    ];

    for (tokens, expected) in cases {
        let actual = parse2(tokens);
        assert_result(&actual, &expected);
    }

    Ok(())
}

#[test]
fn tokenize_tree() {
    let tree: StartupTree = parse2(quote! {
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
        vec![
            ::std::vec![
                ::bevy::prelude::IntoSystemConfigs::into_configs(s1a),
                ::bevy::prelude::IntoSystemConfigs::into_configs(s1b)
            ],
            ::std::vec![
                ::bevy::prelude::IntoSystemConfigs::into_configs(s2a),
                ::bevy::prelude::IntoSystemConfigs::into_configs(s2b)
            ],
            ::std::vec![
                ::bevy::prelude::IntoSystemConfigs::into_configs(s3a),
                ::bevy::prelude::IntoSystemConfigs::into_configs(s3b),
                ::bevy::prelude::IntoSystemConfigs::into_configs(s3c)
            ],
            ::std::vec![::bevy::prelude::IntoSystemConfigs::into_configs(s4a)],
            ::std::vec![::bevy::prelude::IntoSystemConfigs::into_configs(s5a)]
        ]
    }
    .to_string();

    let actual = quote! { #tree }.to_string();

    assert_eq!(actual, expected);
}

#[test]
fn calculate_tree_depth() {
    #[derive(Debug, PartialEq)]
    enum D {
        Value(TreeDepth),
        Tree(TreeDepth, Vec<D>),
    }

    fn get_tree_depths(tree: &Tree) -> D {
        let depth = TreeDepth::default();
        D::Tree(depth, get_tree_depths_inner(tree, depth))
    }

    fn get_tree_depths_inner(tree: &Tree, mut depth: TreeDepth) -> Vec<D> {
        depth += 1;
        tree.branches
            .iter()
            .map(|branch| match branch {
                Branch::Tree(_, _, subtree) => {
                    D::Tree(depth, get_tree_depths_inner(subtree, depth))
                }
                _ => D::Value(depth),
            })
            .collect()
    }

    // let tree = startup_tree! {
    //     s1a,
    //     s1b => {
    //         s2a,
    //         s2b => {
    //             s3a => s4a,
    //         },
    //     },
    // };

    let expected_depths = D::Tree(
        TreeDepth(0),
        vec![
            D::Value(TreeDepth(1)),
            D::Tree(
                TreeDepth(1),
                vec![D::Value(TreeDepth(2)), D::Tree(TreeDepth(2), vec![D::Value(TreeDepth(3))])],
            ),
        ],
    );

    let mut actual = Tree::new(
        TreeDepth::default(),
        vec![
            Branch::from(path!(s1a)),
            Branch::tree(
                Node::from(path!(s1b)),
                Tree::new(
                    TreeDepth::default(),
                    vec![
                        Branch::from(path!(s2a)),
                        Branch::tree(
                            Node::from(path!(s2b)),
                            Tree::new(
                                TreeDepth::default(),
                                vec![Branch::arm(Node::from(path!(s3a)), Branch::from(path!(s4a)))],
                            ),
                        ),
                    ],
                ),
            ),
        ],
    );
    actual.set_depth_root();
    let actual_depths = get_tree_depths(&actual);

    assert_eq!(actual_depths, expected_depths);
}
