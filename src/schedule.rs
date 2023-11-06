use bevy_app::{App, CoreSchedule};
use bevy_ecs::schedule::{IntoSystemSetConfig, Schedules, SystemSet};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, SystemSet)]
#[system_set(base)]
pub enum StartupTreeLayer {
    Set(&'static str),
    Flush(&'static str),
}

#[cfg(test)]
impl StartupTreeLayer {
    pub(crate) fn set_label(self) -> Option<&'static str> {
        match self {
            Self::Set(label) => Some(label),
            Self::Flush(_) => None,
        }
    }
}

pub trait AppExts {
    fn configure_startup_set(&mut self, set: impl IntoSystemSetConfig) -> &mut Self;
}

impl AppExts for App {
    fn configure_startup_set(&mut self, set: impl IntoSystemSetConfig) -> &mut Self {
        self.world
            .resource_mut::<Schedules>()
            .get_mut(&CoreSchedule::Startup)
            .expect("get the startup schedule")
            .configure_set(set);
        self
    }
}
