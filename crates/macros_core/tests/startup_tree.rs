use bevy_startup_tree_macros_core::startup_tree::StartupTree;
use quote::quote;
use syn::parse2;

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
                ::bevy::prelude::IntoScheduleConfigs::into_configs(s1a),
                ::bevy::prelude::IntoScheduleConfigs::into_configs(s1b)
            ],
            ::std::vec![
                ::bevy::prelude::IntoScheduleConfigs::into_configs(s2a),
                ::bevy::prelude::IntoScheduleConfigs::into_configs(s2b)
            ],
            ::std::vec![
                ::bevy::prelude::IntoScheduleConfigs::into_configs(s3a),
                ::bevy::prelude::IntoScheduleConfigs::into_configs(s3b),
                ::bevy::prelude::IntoScheduleConfigs::into_configs(s3c)
            ],
            ::std::vec![::bevy::prelude::IntoScheduleConfigs::into_configs(s4a)],
            ::std::vec![::bevy::prelude::IntoScheduleConfigs::into_configs(s5a)]
        ]
    }
    .to_string();

    let actual = quote! { #tree }.to_string();

    assert_eq!(actual, expected);
}
