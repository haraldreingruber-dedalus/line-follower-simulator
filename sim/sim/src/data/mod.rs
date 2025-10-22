use bevy::prelude::*;

use crate::{bot::motors::Wheel, bot::sensors::bot_position::BotPositionDetector};
use execution_data::{ExecutionData, ExecutionStep};

fn store_data(
    bot_query: Query<&Transform, With<BotPositionDetector>>,
    wheels_query: Query<(&Wheel, &Transform)>,
    mut exec_data: ResMut<ExecutionData>,
    fixed_time: Res<Time<Fixed>>,
) {
    let time_us = fixed_time.elapsed_wrapped().subsec_micros();

    let body_transform = *bot_query.single().unwrap();
    let mut left_wheel_transform = Transform::default();
    let mut right_wheel_transform = Transform::default();

    for (wheel, transform) in wheels_query {
        match wheel.side {
            crate::utils::Side::Left => left_wheel_transform = *transform,
            crate::utils::Side::Right => right_wheel_transform = *transform,
        }
    }

    exec_data.steps.push(ExecutionStep::new(
        time_us,
        body_transform,
        left_wheel_transform,
        right_wheel_transform,
    ));
}

pub fn add_data(app: &mut App) {
    app.insert_resource(ExecutionData::default()).add_systems(
        RunFixedMainLoop,
        (store_data)
            .chain()
            .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
    );
}
