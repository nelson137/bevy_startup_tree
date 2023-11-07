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
//! `~0.9` | `>=0.1.2`
//! `>=0.10` | N/A
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
//! The sets for each sub-array run in order, between the [`StartupSets`][`StartupSet`]
//! `StartupFlush` and `PostStartup`. Additionally, each set has its own flush set that runs after
//! it, containing only the [`apply_system_buffers`] system. The startup phase of an app with the
//! above tree would be:
//!
//! - `StartupSet::PreStartup`
//! - `StartupSet::PreStartupFlush`
//! - `StartupSet::Startup`
//! - `StartupSet::StartupFlush`
//! - Depth 0 tree set
//! - Depth 0 tree flush set
//! - Depth 1 tree set
//! - Depth 1 tree flush set
//! - `StartupSet::PostStartup`
//! - `StartupSet::PostStartupFlush`
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
//!         .add_plugin(TaskPoolPlugin::default())
//!         .add_plugin(LogPlugin::default())
//!         .add_startup_system(begin.in_base_set(StartupSet::Startup))
//!         .add_startup_tree(startup_tree! {
//!             sys_1_a,
//!             sys_1_b => sys_2_a,
//!             sys_1_c => {
//!                 sys_2_b,
//!                 sys_2_c => sys_3_a,
//!             },
//!         })
//!         .add_startup_system(end.in_base_set(StartupSet::PostStartup))
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
//! Note that all of the logs for a depth (those with the same number) are grouped together. This is
//! because all of the systems at some depth in the tree are in the same set. The sets run in order,
//! but the systems within them do not.
//!
//! The `begin` and `end` systems demonstrates when the tree runs during startup. To run a system
//! before the tree, insert it into the `StartupSet::Startup` base set. To run a system after the
//! tree, insert it into the `StartupSet::PostStartup` base set.
//!
//! [`App`]: https://docs.rs/bevy/~0.10/bevy/app/struct.App.html
//! [`apply_system_buffers`]: https://docs.rs/bevy/~0.10/bevy/ecs/schedule/fn.apply_system_buffers.html
//! [`StartupSet`]: https://docs.rs/bevy/~0.10/bevy/app/enum.StartupSet.html
//! [`SystemSet`]: https://docs.rs/bevy/~0.10/bevy/ecs/schedule/trait.SystemSet.html

use std::fmt::Write;

use bevy_app::{App, StartupSet};
use bevy_ecs::schedule::{
    apply_system_buffers, IntoSystemConfig, IntoSystemSetConfig, SystemConfig,
};
use rand::distributions::{Alphanumeric, DistString};

mod rng;
mod schedule;

use self::rng::get_rng;
use self::schedule::{AppExts, StartupTreeLayer};

/// Generate a tree of startup systems that can be consumed by [`AddStartupTree::add_startup_tree`].
///
/// See the [module docs](crate) for more information.
pub use bevy_startup_tree_macros::startup_tree;

const NAMESPACE_LEN: usize = 6;

/// An extension trait for [`bevy::app::App`][`App`].
///
/// [`App`]: https://docs.rs/bevy/*/bevy/app/struct.App.html
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
    /// [`App`]: https://docs.rs/bevy/*/bevy/app/struct.App.html
    fn add_startup_tree<I2, I>(&mut self, startup_tree: I2) -> &mut Self
    where
        I2: IntoIterator<Item = I>,
        I: IntoIterator<Item = SystemConfig>;
}

impl AddStartupTree for App {
    fn add_startup_tree<I2, I>(&mut self, startup_tree: I2) -> &mut Self
    where
        I2: IntoIterator<Item = I>,
        I: IntoIterator<Item = SystemConfig>,
    {
        let mut rng = get_rng();
        let namespace = Alphanumeric.sample_string(&mut rng, NAMESPACE_LEN);
        let label_base = format!("__startup_tree_{namespace}");

        let mut last_layer_set: Option<StartupTreeLayer> = None;

        for (i, level) in startup_tree.into_iter().enumerate() {
            let mut label = label_base.clone();
            write!(label, "_layer_{i}").unwrap();
            let label: &str = label.leak();

            let layer_set = StartupTreeLayer::Set(label);

            let layer_config = layer_set.before(StartupSet::PostStartup);
            self.configure_startup_set(if let Some(last_layer_set) = last_layer_set {
                layer_config.after(last_layer_set)
            } else {
                layer_config.after(StartupSet::StartupFlush)
            });

            for system in level {
                self.add_startup_system(system.in_base_set(layer_set));
            }

            let flush_set = StartupTreeLayer::Flush(label);
            self.configure_startup_set(flush_set.after(layer_set).before(StartupSet::PostStartup));
            self.add_startup_system(apply_system_buffers.in_base_set(flush_set));

            last_layer_set = Some(flush_set);
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::prelude::{App, CoreSchedule, Schedules};

    use crate::{rng::reset_rng, startup_tree, AddStartupTree, StartupTreeLayer};

    fn get_app_startup_tree_labels(app: &App) -> impl Iterator<Item = &'static str> + '_ {
        let schedules = app.world.resource::<Schedules>();
        let startup_schedule = schedules.get(&CoreSchedule::Startup).expect("get startup schedule");
        let startup_graph = startup_schedule.graph();

        startup_graph.hierarchy().graph().nodes().filter_map(|id| {
            startup_graph
                .get_set_at(id)
                .and_then(|set| set.as_any().downcast_ref::<StartupTreeLayer>())
                .copied()
                .and_then(StartupTreeLayer::set_label)
        })
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
            "__startup_tree_zujxzB_layer_0",
            "__startup_tree_zujxzB_layer_1",
            "__startup_tree_zujxzB_layer_2",
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
            "__startup_tree_zujxzB_layer_0",
            "__startup_tree_zujxzB_layer_1",
            "__startup_tree_zujxzB_layer_2",
            "__startup_tree_zujxzB_layer_3",
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

        let expected_labels =
            HashSet::from(["__startup_tree_zujxzB_layer_0", "__startup_tree_ql3QHx_layer_0"]);
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
            app.add_plugin(TaskPoolPlugin::default());
            app.insert_non_send_resource(TestEventData(Vec::with_capacity(11)));
            app.add_startup_system(begin.in_base_set(StartupSet::PreStartup));
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
            app.add_startup_system(end.in_base_set(StartupSet::PostStartup));

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
