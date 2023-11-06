use bevy::{core::FrameCount, log::LogPlugin, prelude::*};

use bevy_startup_tree::{startup_tree, AddStartupTree};

fn main() {
    App::new()
        .add_plugin(TaskPoolPlugin::default())
        .add_plugin(LogPlugin::default())
        .add_plugin(FrameCountPlugin)
        .add_startup_system(begin)
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
        .add_startup_system(end.in_base_set(StartupSet::PostStartup))
        .add_system(|frame: Res<FrameCount>| info!(frame = frame.0))
        .run();
}

fn begin() {
    info!("[Begin]");
}

fn sys_1_a() {
    info!("1.a");
}

fn sys_1_b() {
    info!("1.b");
}

fn sys_1_c() {
    info!("1.c");
}

fn sys_1_d() {
    info!("1.d");
}

fn sys_2_a() {
    info!("2.a");
}

fn sys_2_b() {
    info!("2.b");
}

fn sys_2_c() {
    info!("2.c");
}

fn sys_2_d() {
    info!("2.d");
}

fn sys_3_a() {
    info!("3.a");
}

fn end() {
    info!("[End]");
}
