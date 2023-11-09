use std::fmt;

use bevy_app::{App, Startup};
use bevy_ecs::schedule::{IntoSystemSetConfigs, Schedules, SystemSet};

#[derive(Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
pub enum StartupTreeLayer {
    Set(&'static str),
    Flush(&'static str),
}

impl fmt::Debug for StartupTreeLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(test)]
        if f.alternate() {
            return match self {
                Self::Set(label) | Self::Flush(label) => f.write_str(label),
            };
        }
        match self {
            Self::Set(label) => f.debug_tuple("Set").field(label).finish(),
            Self::Flush(label) => f.debug_tuple("Flush").field(label).finish(),
        }
    }
}

pub trait AppExts {
    fn configure_startup_set(&mut self, set: impl IntoSystemSetConfigs) -> &mut Self;
}

impl AppExts for App {
    fn configure_startup_set(&mut self, set: impl IntoSystemSetConfigs) -> &mut Self {
        self.world
            .resource_mut::<Schedules>()
            .get_mut(Startup)
            .expect("get the startup schedule")
            .configure_sets(set);
        self
    }
}
