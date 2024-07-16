//! Insert dependency trees of startup systems into [Bevy `App`s][`App`].
//!
//! Define dependency trees of startup systems for a Bevy `App` with the [`startup_tree`] macro.
//! Insert trees into an `App` with the [`AddStartupTree::add_startup_tree`] extension method. It is
//! strongly recommended that the macro is used to generate the data structure that is consumed by
//! `add_startup_tree`.
//!
//! This is useful in scenarios where the startup logic is complex and would benefit from being
//! broken up into multiple systems. Some of this startup logic can be run in parallel; others may
//! require that certain systems run in a particular order. For example, a system that spawns a
//! complex `bevy_ui` can get very large, deeply nested, and difficult to maintain. Such a system
//! can be divided into multiple that work together to create the complex entity hierarchy. Systems
//! that spawn children entities must run after the one that spawns the parent; this is where
//! `bevy_startup_tree` becomes useful.
//!
//! # Bevy Compatibility
//!
//! `bevy` | `bevy_startup_tree`
//! :--- | :---
//! `~0.13` | `>=0.5`
//! `~0.12` | `>=0.4`
//! `~0.11` | `>=0.3`
//! `~0.10` | `>=0.2`
//! `~0.9` | `~0.1`
//! `<0.9` | Not supported
//!
//! # Behavior
//!
//! The systems that make up a startup tree, or nodes, are grouped by depth. The `startup_tree`
//! macro generates a 2-D array where each row with index `i` contains the nodes at depth `i` in the
//! tree. This 2-D array is consumed by `add_startup_tree` where each depth sub-array is combined
//! into a [`SystemSet`].
//!
//! ```rust no_run
//! # use bevy_startup_tree::startup_tree;
//! # fn sys_1_a() {}
//! # fn sys_1_b() {}
//! # fn sys_2() {}
//! # std::mem::drop(
//! startup_tree! {
//!     sys_1_a,
//!     sys_1_b => sys_2
//! }
//! # );
//! ```
//!
//! This macro invocation would generate the following 2-D array:
//!
//! <pre>
//! [ [sys_1_a, sys_1_b], [sys_2] ]
//! </pre>
//!
//! Note that there are two sub-arrays: one for the nodes at depth 0 and one for depth 1.
//!
//! The sets for each sub-array run in order during the [`Startup` schedule][`Startup`]. Thus, the
//! system sets inserted into the `Startup` schedule for the above tree would be:
//!
//! - Depth 0 tree set
//! - Depth 0 tree flush set
//! - Depth 1 tree set
//! - Depth 1 tree flush set
//!
//! # Example
//!
//! The following is an example Bevy `App` with a startup tree. Note that the app will go through
//! the startup phase, run a single frame cycle, and then exit.
//!
//! ```rust no_run
//! use bevy::{log::LogPlugin, prelude::*};
//! use bevy_startup_tree::{startup_tree, AddStartupTree};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((TaskPoolPlugin::default(), LogPlugin::default()))
//!         .add_systems(PreStartup, begin)
//!         .add_startup_tree(startup_tree! {
//!             sys_1_a,
//!             sys_1_b => sys_2_a,
//!             sys_1_c => {
//!                 sys_2_b,
//!                 sys_2_c => sys_3_a,
//!             },
//!         })
//!         .add_systems(PostStartup, end)
//!         .run();
//! }
//!
//! fn begin() { info!("[Begin]"); }
//! fn sys_1_a() { info!("1.a"); }
//! fn sys_1_b() { info!("1.b"); }
//! fn sys_1_c() { info!("1.c"); }
//! fn sys_2_a() { info!("2.a"); }
//! fn sys_2_b() { info!("2.b"); }
//! fn sys_2_c() { info!("2.c"); }
//! fn sys_3_a() { info!("3.a"); }
//! fn end() { info!("[End]"); }
//! ```
//!
//! ## Output
//!
//! <pre>
//! 2023-01-08T19:38:41.664766Z  INFO example_app: [Begin]
//! 2023-01-08T19:38:41.664906Z  INFO example_app: 1.b
//! 2023-01-08T19:38:41.664937Z  INFO example_app: 1.c
//! 2023-01-08T19:38:41.664959Z  INFO example_app: 1.a
//! 2023-01-08T19:38:41.665104Z  INFO example_app: 2.c
//! 2023-01-08T19:38:41.665133Z  INFO example_app: 2.a
//! 2023-01-08T19:38:41.665141Z  INFO example_app: 2.b
//! 2023-01-08T19:38:41.665204Z  INFO example_app: 3.a
//! 2023-01-08T19:38:41.665264Z  INFO example_app: [End]
//! </pre>
//!
//! Note that all of the logs for a depth (those with the same number) are grouped together because
//! these systems belong to the same set. The sets run in order, causing the numbers to be sorted.
//! However, the systems within a set do not, causing the letters for a given number to be
//! unordered.
//!
//! The `begin` and `end` systems demonstrates when the tree runs during startup. To run a system
//! before the tree, insert it into the [`PreStartup` schedule][`PreStartup`]. To run a system after
//! the tree, insert it into the [`PostStartup` schedule][`PostStartup`].
//!
//! [`App`]: https://docs.rs/bevy/~0.13/bevy/app/struct.App.html
//! [`PostStartup`]: https://docs.rs/bevy/~0.13/bevy/app/struct.PostStartup.html
//! [`PreStartup`]: https://docs.rs/bevy/~0.13/bevy/app/struct.PreStartup.html
//! [`Startup`]: https://docs.rs/bevy/~0.13/bevy/app/struct.Startup.html
//! [`SystemSet`]: https://docs.rs/bevy/~0.13/bevy/ecs/schedule/trait.SystemSet.html

use std::fmt::Write;

use bevy_app::{App, Startup};
use bevy_ecs::schedule::{IntoSystemConfigs, IntoSystemSetConfigs, SystemConfigs};
use rand::distributions::{Alphanumeric, DistString};

mod rng;
mod schedule;

use self::rng::get_rng;
use self::schedule::StartupTreeLayer;

/// Generate a tree of startup systems that can be consumed by [`AddStartupTree::add_startup_tree`].
///
/// See the [module docs](crate) for more information.
pub use bevy_startup_tree_macros::startup_tree;

const NAMESPACE_LEN: usize = 6;

/// An extension trait for [`bevy::app::App`][`App`].
///
/// [`App`]: https://docs.rs/bevy/~0.13/bevy/app/struct.App.html
pub trait AddStartupTree {
    /// Add a dependency tree of startup systems to the [`App`].
    ///
    /// The input is an iterator over a 2-D array describing a tree where each row (inner iterator
    /// `I`) with index `i` contains the nodes at depth `i` in the tree. There is *no guarantee*
    /// that systems at the same depth with run in any specific order. It is strongly recommended
    /// that the [`startup_tree` macro](startup_tree) is used to generate the tree.
    ///
    /// See the [module docs](crate) for more information.
    ///
    /// [`App`]: https://docs.rs/bevy/~0.13/bevy/app/struct.App.html
    fn add_startup_tree<I2, I>(&mut self, startup_tree: I2) -> &mut Self
    where
        I2: IntoIterator<Item = I>,
        I: IntoIterator<Item = SystemConfigs>;
}

impl AddStartupTree for App {
    fn add_startup_tree<I2, I>(&mut self, startup_tree: I2) -> &mut Self
    where
        I2: IntoIterator<Item = I>,
        I: IntoIterator<Item = SystemConfigs>,
    {
        let mut rng = get_rng();
        let namespace = Alphanumeric.sample_string(&mut rng, NAMESPACE_LEN);
        let label_base = format!("__startup_tree_{namespace}");

        startup_tree.into_iter().enumerate().fold(None, |last_layer_set, (i, level)| {
            let mut label = label_base.clone();
            write!(label, "_layer_{i}").unwrap();
            let label: &str = label.leak();

            let layer_set = StartupTreeLayer(label);

            let layer_config = if let Some(last_layer_set) = last_layer_set {
                layer_set.after(last_layer_set)
            } else {
                layer_set.into_configs()
            };
            self.configure_sets(Startup, layer_config);

            for system in level {
                self.add_systems(Startup, system.in_set(layer_set));
            }

            Some(layer_set)
        });

        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::prelude::{App, Schedules, Startup};

    use crate::{rng::reset_rng, startup_tree, AddStartupTree};

    fn get_app_startup_tree_labels(app: &App) -> impl Iterator<Item = String> + '_ {
        let schedules = app.world.resource::<Schedules>();
        let startup_schedule = schedules.get(Startup).expect("get startup schedule");
        let startup_graph = startup_schedule.graph();

        // use bevy::utils::{intern::Internable, label::DynHash};
        // use bevy_ecs::schedule::{InternedSystemSet, SystemSet};
        // use std::any::TypeId;
        // eprintln!("===");
        // eprintln!("interned_id = {:?}", TypeId::of::<InternedSystemSet>());
        // eprintln!("dyn_id = {:?}", TypeId::of::<dyn SystemSet>());
        // eprintln!("dyn_ref_id = {:?}", TypeId::of::<&dyn SystemSet>());
        // eprintln!("box_layer_id = {:?}", TypeId::of::<Box<StartupTreeLayer>>());
        // eprintln!("box_dyn_id   = {:?}", TypeId::of::<Box<dyn SystemSet>>());
        // eprintln!("===");

        startup_graph
            .hierarchy()
            .graph()
            .nodes()
            .filter_map(|id| startup_graph.get_set_at(id))
            .map(|set| format!("{set:#?}"))
            .filter(|label| label.starts_with("__startup_tree"))
    }

    fn system() {}

    #[test]
    fn adds_sequential_labels() {
        reset_rng();

        let mut app = App::new();

        app.add_startup_tree(startup_tree! {
            system => {
                system => system
            }
        });

        let expected_labels = HashSet::from([
            "__startup_tree_zujxzB_layer_0".into(),
            "__startup_tree_zujxzB_layer_1".into(),
            "__startup_tree_zujxzB_layer_2".into(),
        ]);
        let actual_labels = HashSet::from_iter(get_app_startup_tree_labels(&app));
        assert_eq!(actual_labels, expected_labels);
    }

    #[test]
    fn adds_correct_labels_for_complex_tree() {
        reset_rng();

        let mut app = App::new();

        app.add_startup_tree(startup_tree! {
            system,
            system => {
                system => system,
                system => {
                    system,
                    system => system,
                }
            },
            system,
        });

        let expected_labels = HashSet::from([
            "__startup_tree_zujxzB_layer_0".into(),
            "__startup_tree_zujxzB_layer_1".into(),
            "__startup_tree_zujxzB_layer_2".into(),
            "__startup_tree_zujxzB_layer_3".into(),
        ]);
        let actual_labels = HashSet::from_iter(get_app_startup_tree_labels(&app));
        assert_eq!(actual_labels, expected_labels);
    }

    #[test]
    fn multiple_trees_dont_reuse_labels() {
        reset_rng();

        let mut app = App::new();

        app.add_startup_tree(startup_tree! { system });
        app.add_startup_tree(startup_tree! { system });

        let expected_labels = HashSet::from([
            "__startup_tree_zujxzB_layer_0".into(),
            "__startup_tree_ql3QHx_layer_0".into(),
        ]);
        let actual_labels = HashSet::from_iter(get_app_startup_tree_labels(&app));
        assert_eq!(actual_labels, expected_labels);
    }

    mod e2e {
        use bevy::prelude::*;

        use crate::{rng::reseed_rng, startup_tree, AddStartupTree};

        #[derive(Resource, Debug)]
        struct TestEventData(Vec<TestEvent>);

        #[derive(Debug, PartialEq, Eq)]
        enum TestEvent {
            Begin,
            One,
            Two,
            Three,
            End,
        }

        macro_rules! test_systems {
            ($($name:ident => $event:path);+ $(;)?) => {
                $( fn $name(mut data: NonSendMut<TestEventData>) { data.0.push($event); } )+
            };
        }

        test_systems! {
            begin => TestEvent::Begin;
            sys_1_a => TestEvent::One;
            sys_1_b => TestEvent::One;
            sys_1_c => TestEvent::One;
            sys_1_d => TestEvent::One;
            sys_2_a => TestEvent::Two;
            sys_2_b => TestEvent::Two;
            sys_2_c => TestEvent::Two;
            sys_2_d => TestEvent::Two;
            sys_3_a => TestEvent::Three;
            end => TestEvent::End;
        }

        #[test]
        fn end_to_end_test() {
            reseed_rng();

            let mut app = App::new();
            app.add_plugins(TaskPoolPlugin::default());
            app.insert_non_send_resource(TestEventData(Vec::with_capacity(11)));
            app.add_systems(PreStartup, begin);
            app.add_startup_tree(startup_tree! {
                sys_1_a => {
                    sys_2_a,
                    sys_2_b,
                },
                sys_1_b => {
                    sys_2_c,
                    sys_2_d => sys_3_a,
                },
                sys_1_c,
                sys_1_d,
            });
            app.add_systems(PostStartup, end);

            app.update();

            assert_eq!(
                app.world.non_send_resource::<TestEventData>().0,
                &[
                    TestEvent::Begin,
                    TestEvent::One,
                    TestEvent::One,
                    TestEvent::One,
                    TestEvent::One,
                    TestEvent::Two,
                    TestEvent::Two,
                    TestEvent::Two,
                    TestEvent::Two,
                    TestEvent::Three,
                    TestEvent::End
                ]
            );
        }
    }
}
