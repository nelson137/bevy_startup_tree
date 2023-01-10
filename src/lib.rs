use bevy::{ecs::schedule::SystemDescriptor, prelude::*};

pub use bevy_startup_tree_macros::startup_tree;

const STAGE_LABELS: [&str; 128] = bevy_startup_tree_macros::generage_stage_labels!(128);

pub trait AddStartupTree {
    /// Add a dependency tree of startup systems. See [`startup_tree`] for how to build a tree.
    ///
    /// TODO
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
