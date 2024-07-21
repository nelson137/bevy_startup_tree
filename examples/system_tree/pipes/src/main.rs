//!

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use rand::{prelude::*, Rng as _};

use bevy_startup_tree::system_tree;

fn main() {
    App::new()
        .add_plugins((LogPlugin::default(), TaskPoolPlugin::default()))
        .insert_resource(Rng(SmallRng::seed_from_u64(3)))
        .init_resource::<Value1>()
        .init_resource::<Value2>()
        .add_systems(
            // This doesn't have to be `Startup`! Try changing this to `Update`
            // and watch it generate a new `a` and `b` and calculate their
            // distance each frame. (A custom plugin below detects if there is
            // any system in the `Update` schedule and insert the
            // `ScheduleRunnerPlugin` which makes the app run until stopped,
            // otherwise the app will only execute for one frame.)
            Startup,
            // Generates a closure system like:
            // ```rust
            // #[allow(clippy::let_unit_value)]
            // |world: &mut ::bevy::ecs::world::World| {
            //     use bevy::ecs::system::RunSystemOnce;
            //     let _step0_out = world.run_system_once_with((), step0);
            //     let _step1_out = world.run_system_once_with(_step0_out, step1);
            //     let _step2_out = world.run_system_once_with(_step1_out, step2);
            //     let _step3_out = world.run_system_once_with(_step2_out, step3);
            // }
            // ```
            system_tree! {
                step0 => step1 => step2 => step3
            },
        )
        .add_plugins(DynamicScheduleRunnerPlugin)
        .run();
}

struct DynamicScheduleRunnerPlugin;

impl Plugin for DynamicScheduleRunnerPlugin {
    fn build(&self, app: &mut App) {
        let update_len = app.get_schedule(Update).map(|s| s.systems_len()).unwrap_or_default();
        if update_len > 0 {
            app.add_plugins(ScheduleRunnerPlugin::default());
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Rng(pub SmallRng);

impl Rng {
    fn rand_f32(&mut self) -> f32 {
        self.gen_range(0.0_f32..10.0).trunc()
    }
}

#[derive(Resource, Default, Deref)]
pub struct Value1(f32);

#[derive(Resource, Default, Deref)]
pub struct Value2(f32);

/// Generate two random floats from 0 to 10.
pub fn step0(mut rng: ResMut<Rng>, mut a_res: ResMut<Value1>, mut b_res: ResMut<Value2>) {
    let (a, b) = (rng.rand_f32(), rng.rand_f32());
    a_res.0 = a;
    b_res.0 = b;
    info!("==============================");
    info!(a, b, "[step0] begin");
}

/// Square the values.
pub fn step1(a: Res<Value1>, b: Res<Value2>) -> (f32, f32) {
    let (a, b) = (**a, **b);
    let (a, b) = (a * a, b * b);
    info!(a, b, "[step1] square values");
    (a, b)
}

/// Sum the values.
pub fn step2(In((a, b)): In<(f32, f32)>) -> f32 {
    let sum = a + b;
    info!(sum, "[step2] sum values");
    sum
}

/// Square root the sum.
pub fn step3(In(sum): In<f32>) {
    let sqrt = sum.sqrt();
    info!(sqrt = %format!("{sqrt:.4}"), "[step3] sqrt the sum");
}
