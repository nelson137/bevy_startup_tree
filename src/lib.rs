//! Insert dependency trees of startup systems into [Bevy `App`s][bevy App].
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
//! ## Behavior
//!
//! The systems that make up a startup tree, or nodes, are grouped by depth. The `startup_tree`
//! macro generates a 2-D array where each row with index `i` contains the nodes at depth `i` in the
//! tree. This 2-D array is consumed by `add_startup_tree` where each depth sub-array is combined
//! into a [parallel `SystemStage`][`SystemStage::parallel`].
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
//! `add_startup_tree` inserts the tree after [`StartupStage::Startup`] so the stages of the startup
//! phase run in the following order:
//!
//! - `StartupStage::PreStartup`
//! - `StartupStage::Startup`
//! - Tree stages...
//! - `StartupStage::PostStartup`
//!
//! ## Example
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
//!         .add_plugin(CorePlugin::default())
//!         .add_plugin(LogPlugin::default())
//!         .add_startup_system(begin)
//!         .add_startup_tree(startup_tree! {
//!             sys_1_a,
//!             sys_1_b => sys_2_a,
//!             sys_1_c => {
//!                 sys_2_b,
//!                 sys_2_c => sys_3_a,
//!             },
//!         })
//!         .add_startup_system_to_stage(StartupStage::PostStartup, end)
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
//! ### Output
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
//! because all of the systems at some depth in the tree are in the same stage. However, the logs
//! within a stage run in no particular order because the stage is
//! [parallel][`SystemStage::parallel`].
//!
//! The `begin` and `end` systems show when the tree runs during the startup phase. The tree's
//! stages are inserted after `StartupStage::Startup` so any system added to
//! `StartupStage::PreStartup` or `StartupStage::Startup` run before the tree and any system added
//! to `StartupStage::PostStartup` run after the tree.
//!
//! ## Bevy Compatibility
//!
//! `bevy` | `bevy_startup_tree`
//! :--- | :---
//! `~0.9` | `>=0.1.2`
//! `>=0.10` | N/A
//!
//! [bevy App]: https://docs.rs/bevy/*/bevy/app/struct.App.html
//! [`SystemStage`]: https://docs.rs/bevy/~0.9/bevy/ecs/schedule/struct.SystemStage.html
//! [`StartupStage::Startup`]: https://docs.rs/bevy/~0.9/bevy/app/enum.StartupStage.html
//! [`SystemStage::parallel`]: https://docs.rs/bevy/~0.9/bevy/ecs/schedule/struct.SystemStage.html#method.parallel

use std::fmt::Write;

use bevy_app::{App, StartupStage};
use bevy_ecs::schedule::{SystemDescriptor, SystemStage};
use rand::distributions::{Alphanumeric, DistString};

mod rng;

use self::rng::get_rng;

/// Generate a tree of startup systems that can be consumed by [`AddStartupTree::add_startup_tree`].
///
/// See the [module docs](crate) for more information.
pub use bevy_startup_tree_macros::startup_tree;

/// An extension trait for [`bevy::app::App`][bevy App].
///
/// [bevy App]: https://docs.rs/bevy/*/bevy/app/struct.App.html
pub trait AddStartupTree {
    /// Add a dependency tree of startup systems to the [Bevy `App`][bevy App] `&mut self`.
    ///
    /// The input is an iterator over a 2-D array describing a tree where each row (inner iterator
    /// `I`) with index `i` contains the nodes at depth `i` in the tree. Nodes at the same depth are
    /// run in parallel and thus the order in which they will run is not guaranteed. It is strongly
    /// recommended that the [`startup_tree` macro](startup_tree) is used to generate the tree.
    ///
    /// See the [module docs](crate) for more information.
    ///
    /// [`SystemStage::parallel`]: https://docs.rs/bevy/~0.9/bevy/ecs/schedule/struct.SystemStage.html#method.parallel
    /// [bevy App]: https://docs.rs/bevy/*/bevy/app/struct.App.html
    fn add_startup_tree<I2, I>(&mut self, startup_tree: I2) -> &mut Self
    where
        I2: IntoIterator<Item = I>,
        I: IntoIterator<Item = SystemDescriptor>;
}

impl AddStartupTree for App {
    fn add_startup_tree<I2, I>(&mut self, startup_tree: I2) -> &mut Self
    where
        I2: IntoIterator<Item = I>,
        I: IntoIterator<Item = SystemDescriptor>,
    {
        let mut rng = get_rng();
        let namespace = Alphanumeric.sample_string(&mut rng, 6);
        let label_base = format!("__startup_tree_stage_{namespace}_");

        let mut last_label: &'static str = "";

        for (i, level) in startup_tree.into_iter().enumerate() {
            let mut label = label_base.clone();
            write!(label, "{i}").unwrap();
            let label: &'static str = Box::leak(label.into_boxed_str());

            let mut stage = SystemStage::parallel();
            for system in level {
                stage.add_system(system);
            }

            if i == 0 {
                self.add_startup_stage_after(StartupStage::Startup, label, stage);
            } else {
                self.add_startup_stage_after(last_label, label, stage);
            }

            last_label = label;
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::{
        app::{App, StartupSchedule},
        ecs::schedule::{Schedule, StageLabel},
    };

    use crate::{rng::reset_rng, startup_tree, AddStartupTree};

    fn get_app_startup_tree_labels(app: &App) -> impl Iterator<Item = &'static str> + '_ {
        let startup_schedule = app.schedule.get_stage::<Schedule>(StartupSchedule).unwrap();
        let n_labels = startup_schedule.iter_stages().count();
        // By default, the startup schedule contains the stages `PreStartup`, `Startup`, and
        // `PostStartup`. Startup tree stages are inserted after `Startup` meaning the first 2
        // labels can be skipped. Then, we can take the amount of startup tree labels `n - 3` where
        // `n` is the total number of labels.
        let n_startup_tree_stages = n_labels - 3;
        startup_schedule
            .iter_stages()
            .skip(2)
            .take(n_startup_tree_stages)
            .map(|(id, _)| id.as_str())
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
            "__startup_tree_stage_zujxzB_0",
            "__startup_tree_stage_zujxzB_1",
            "__startup_tree_stage_zujxzB_2",
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
            "__startup_tree_stage_zujxzB_0",
            "__startup_tree_stage_zujxzB_1",
            "__startup_tree_stage_zujxzB_2",
            "__startup_tree_stage_zujxzB_3",
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
            HashSet::from(["__startup_tree_stage_zujxzB_0", "__startup_tree_stage_ql3QHx_0"]);
        let actual_labels = HashSet::from_iter(get_app_startup_tree_labels(&app));
        assert_eq!(actual_labels, expected_labels);
    }

    mod e2e {
        use bevy::{
            app::{App, StartupStage},
            core::CorePlugin,
            ecs::system::{NonSendMut, Resource},
        };

        use crate::{rng::reset_rng, startup_tree, AddStartupTree};

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
            reset_rng();

            let mut app = App::new();
            app.add_plugin(CorePlugin::default());
            app.insert_non_send_resource(TestEventData(Vec::with_capacity(11)));
            app.add_startup_system(begin);
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
            app.add_startup_system_to_stage(StartupStage::PostStartup, end);

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
