use bevy_startup_tree_macros_core::{Branch, Node, NodeChild, Tree, TreeDepth};
use quote::quote;
use syn::parse2;

mod utils;

use self::utils::{assert_err, assert_ok, path};

#[test]
fn parse_tree_with_one_node() -> syn::Result<()> {
    let tree: Tree = parse2(quote! { sys })?;
    assert_eq!(tree, Tree::from_branch(Node::new(path!(sys)).into()));
    Ok(())
}

#[test]
fn parse_complex_tree() -> syn::Result<()> {
    let expected = Tree::with_branches(vec![
        Branch::from_path(path!(s1a), true),
        Branch::new(
            Node::new(path!(s1b)),
            Some(NodeChild::Tree(Tree::with_branches(vec![
                Branch::new(
                    Node::new(path!(s2a)),
                    Some(NodeChild::Node(Node::new(path!(s3a)))),
                    true,
                ),
                Branch::new(
                    Node::new(path!(s2b)),
                    Some(NodeChild::Tree(Tree::with_branches(vec![
                        Branch::from_path(path!(s3b), true),
                        Branch::from_path(path!(s3c), true),
                    ]))),
                    true,
                ),
            ]))),
            true,
        ),
    ]);

    let actual: Tree = parse2(quote! {
        s1a,
        s1b => {
            s2a => s3a,
            s2b => {
                s3b,
                s3c,
            },
        },
    })?;

    assert_eq!(actual, expected);

    Ok(())
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
            .map(|branch| match &branch.child {
                Some((_, NodeChild::Tree(subtree))) => {
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
            Branch::from_path(path!(s1a), true),
            Branch::new(
                Node::new(path!(s1b)),
                Some(NodeChild::Tree(Tree::new(
                    TreeDepth::default(),
                    vec![
                        Branch::from_path(path!(s2a), true),
                        Branch::new(
                            Node::new(path!(s2b)),
                            Some(NodeChild::Tree(Tree::new(
                                TreeDepth::default(),
                                vec![Branch::new(
                                    path!(s3a),
                                    Some(NodeChild::Node(Node::new(path!(s4a)))),
                                    true,
                                )],
                            ))),
                            true,
                        ),
                    ],
                ))),
                true,
            ),
        ],
    );
    actual.set_depth_root();
    let actual_depths = get_tree_depths(&actual);

    assert_eq!(actual_depths, expected_depths);
}

#[test]
fn parse_branch() -> syn::Result<()> {
    let cases = [
        (quote! { sys }, Ok(Branch::from_path(path!(sys), false))),
        (quote! { sys, }, Ok(Branch::from_path(path!(sys), true))),
        (quote! { sys => }, Err("unexpected end of input, expected identifier")),
        (
            quote! { sys => sys_child },
            Ok(Branch::new(path!(sys), Some(NodeChild::Node(path!(sys_child))), false)),
        ),
        (
            quote! { sys => sys_child, },
            Ok(Branch::new(path!(sys), Some(NodeChild::Node(path!(sys_child))), true)),
        ),
        (
            quote! { sys => { sys_child } },
            Ok(Branch::new(
                path!(sys),
                Some(NodeChild::Tree(Tree::from_path(path!(sys_child), false))),
                false,
            )),
        ),
        (
            quote! { sys => { sys_child }, },
            Ok(Branch::new(
                path!(sys),
                Some(NodeChild::Tree(Tree::from_path(path!(sys_child), false))),
                true,
            )),
        ),
    ];

    for (tokens, expected) in cases {
        let actual = parse2::<Branch>(tokens);
        match &expected {
            Ok(expected_branch) => assert_ok(&actual, expected_branch),
            Err(expected_err) => assert_err(&actual, expected_err),
        }
    }

    Ok(())
}
