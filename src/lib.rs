use bevy_app::{App, StartupStage};
use bevy_ecs::schedule::{SystemDescriptor, SystemStage};

/// Generate a tree of startup systems that can be used by [`AddStartupTree`].
///
/// TODO
pub use bevy_startup_tree_macros::startup_tree;

const STAGE_LABELS: [&str; 128] = bevy_startup_tree_macros::generage_stage_labels!(128);

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
    /// ```rust
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
    /// ```
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
        for (i, level) in startup_tree.into_iter().enumerate() {
            let label = STAGE_LABELS[i];

            let mut stage = SystemStage::parallel();
            for system in level {
                stage.add_system(system);
            }

            if i == 0 {
                self.add_startup_stage_after(StartupStage::Startup, label, stage);
            } else {
                self.add_startup_stage_after(STAGE_LABELS[i - 1], label, stage);
            }
        }

        self
    }
}
