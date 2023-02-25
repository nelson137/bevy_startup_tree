use std::fmt::Write;

use bevy_app::{App, StartupStage};
use bevy_ecs::schedule::{SystemDescriptor, SystemStage};
use rand::distributions::{Alphanumeric, DistString};

mod rng;

use self::rng::get_rng;

/// Generate a tree of startup systems that can be used by [`AddStartupTree`].
///
/// TODO
pub use bevy_startup_tree_macros::startup_tree;

/// An extension trait for [`bevy::app::App`](bevy::app::App).
pub trait AddStartupTree {
    /// Add a dependency tree of startup systems. See [`startup_tree`] for how to build a tree.
    ///
    /// Each level of the tree is grouped into a [`SystemStage`] with the parallel executor, meaning
    /// there is no guarantee of the order in which they will run. The stages are also made to run
    /// in order.
    ///
    /// ## Example
    ///
    /// The following is an example bevy app that uses a startup tree with 11 systems that
    /// demonstrate the order of execution. The app is configured to only go through startup and
    /// tick once.
    ///
    /// The startup tree systems are arranged into 3 levels -- those named `sys_1_*`, `sys_2_*`, and
    /// `sys_3_*` -- that log their name. Since levels run *in order* and systems within a level run
    /// *unordered* the logs of level 1 will be together, then those of level 2, then 3.
    ///
    /// There are also the `begin` and `end` systems which show when the tree runs during startup.
    /// The tree is added after [`StartupStage::Startup`] so the startup stages run in the following
    /// order:
    ///
    /// - `StartupStage::PreStartup`
    /// - `StartupStage::Startup`
    /// - tree level 1
    /// - tree level 2
    /// - tree level 3
    /// - `StartupStage::PostStartup`
    ///
    /// ### Code
    ///
    /// ```rust ignore
    /// use bevy::{log::LogPlugin, prelude::*};
    /// use bevy_startup_tree::{startup_tree, AddStartupTree};
    ///
    /// fn main() {
    ///     App::new()
    ///         .add_plugin(CorePlugin::default())
    ///         .add_plugin(LogPlugin::default())
    ///         .add_startup_system(begin)
    ///         .add_startup_tree(startup_tree! {
    ///             sys_1_a => {
    ///                 sys_2_a,
    ///                 sys_2_b,
    ///             },
    ///             sys_1_b => {
    ///                 sys_2_c,
    ///                 sys_2_d => sys_3_a,
    ///             },
    ///             sys_1_c,
    ///             sys_1_d,
    ///         })
    ///         .add_startup_system_to_stage(StartupStage::PostStartup, end)
    ///         .run();
    /// }
    ///
    /// fn begin() { info!("[Begin]"); }
    /// fn sys_1_a() { info!("1.a"); }
    /// fn sys_1_b() { info!("1.b"); }
    /// fn sys_1_c() { info!("1.c"); }
    /// fn sys_1_d() { info!("1.d"); }
    /// fn sys_2_a() { info!("2.a"); }
    /// fn sys_2_b() { info!("2.b"); }
    /// fn sys_2_c() { info!("2.c"); }
    /// fn sys_2_d() { info!("2.d"); }
    /// fn sys_3_a() { info!("3.a"); }
    /// fn end() { info!("[End]"); }
    /// ```
    ///
    /// ### Output
    ///
    /// ```ignore
    /// 2023-01-08T19:38:41.664766Z  INFO example_app: [Begin]
    /// 2023-01-08T19:38:41.664906Z  INFO example_app: 1.b
    /// 2023-01-08T19:38:41.664937Z  INFO example_app: 1.c
    /// 2023-01-08T19:38:41.664959Z  INFO example_app: 1.a
    /// 2023-01-08T19:38:41.664967Z  INFO example_app: 1.d
    /// 2023-01-08T19:38:41.665104Z  INFO example_app: 2.c
    /// 2023-01-08T19:38:41.665109Z  INFO example_app: 2.d
    /// 2023-01-08T19:38:41.665133Z  INFO example_app: 2.a
    /// 2023-01-08T19:38:41.665141Z  INFO example_app: 2.b
    /// 2023-01-08T19:38:41.665204Z  INFO example_app: 3.a
    /// 2023-01-08T19:38:41.665264Z  INFO example_app: [End]
    /// ```
    fn add_startup_tree<T, U>(&mut self, startup_tree: T) -> &mut Self
    where
        T: IntoIterator<Item = U>,
        U: IntoIterator<Item = SystemDescriptor>;
}

impl AddStartupTree for App {
    fn add_startup_tree<T, U>(&mut self, startup_tree: T) -> &mut Self
    where
        T: IntoIterator<Item = U>,
        U: IntoIterator<Item = SystemDescriptor>,
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
        use std::sync::Mutex;

        use bevy::{
            app::{App, StartupStage},
            core::CorePlugin,
        };
        use lazy_static::lazy_static;

        use crate::{rng::reset_rng, startup_tree, AddStartupTree};

        lazy_static! {
            static ref TEST_EVENTS: Mutex<Vec<TestEvent>> = Mutex::new(Vec::with_capacity(16));
        }

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
                $( fn $name() { TEST_EVENTS.lock().unwrap().push($event); } )+
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

            app.run();

            assert_eq!(
                *TEST_EVENTS.lock().unwrap(),
                vec![
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
