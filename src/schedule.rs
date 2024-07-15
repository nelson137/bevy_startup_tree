use std::fmt;

use bevy_ecs::schedule::SystemSet;

#[derive(Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
pub struct StartupTreeLayer(pub &'static str);

impl fmt::Debug for StartupTreeLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(test)]
        if f.alternate() {
            return f.write_str(self.0);
        }
        f.debug_tuple("Set").field(&self.0).finish()
    }
}
