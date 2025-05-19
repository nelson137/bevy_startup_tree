use bevy::{
    diagnostic::{FrameCount, FrameCountPlugin},
    log::LogPlugin,
    prelude::*,
};

use bevy_startup_tree::{startup_tree, AddStartupTree};

fn main() {
    App::new()
        .add_plugins((TaskPoolPlugin::default(), LogPlugin::default(), FrameCountPlugin))
        .add_systems(PreStartup, begin)
        .add_startup_tree(startup_tree! {
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
        })
        .add_systems(PostStartup, end)
        .add_systems(Update, |frame: Res<FrameCount>| info!(frame = frame.0))
        .run();
}

#[rustfmt::skip]
mod systems {
    use bevy::log::info;
    pub fn begin()   { info!("[Begin]"); }
    pub fn sys_1_a() { info!("1.a");     }
    pub fn sys_1_b() { info!("1.b");     }
    pub fn sys_1_c() { info!("1.c");     }
    pub fn sys_1_d() { info!("1.d");     }
    pub fn sys_2_a() { info!("2.a");     }
    pub fn sys_2_b() { info!("2.b");     }
    pub fn sys_2_c() { info!("2.c");     }
    pub fn sys_2_d() { info!("2.d");     }
    pub fn sys_3_a() { info!("3.a");     }
    pub fn end()     { info!("[End]");   }
}
use systems::*;
