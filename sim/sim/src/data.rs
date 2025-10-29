use bevy::prelude::*;

use crate::{
    app_builder::BotUpdate,
    bot::sensors::{bot_position::BotPositionDetector, imu::compute_imu_data},
};
use execution_data::{ExecutionData, MotorAngles};

fn store_data(
    bot_query: Query<&Transform, With<BotPositionDetector>>,
    motor_angles: Res<MotorAngles>,
    mut exec_data: ResMut<ExecutionData>,
) {
    let body_transform = *bot_query.single().unwrap();
    exec_data.body_data.steps.push(body_transform);
    exec_data.left_wheel_data.steps.push(motor_angles.left);
    exec_data.right_wheel_data.steps.push(motor_angles.right);
}

pub struct StoreExecDataPlugin {
    step_period_us: u32,
    force_initially_started: bool,
}

impl StoreExecDataPlugin {
    pub fn new(step_period_us: u32, force_initially_started: bool) -> Self {
        Self {
            step_period_us,
            force_initially_started,
        }
    }
}

impl Plugin for StoreExecDataPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExecutionData::empty(
            self.step_period_us,
            self.force_initially_started,
        ))
        .add_systems(BotUpdate, store_data.after(compute_imu_data));
    }
}
