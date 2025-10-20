use bevy::prelude::*;

use execution_data::SensorsData;

pub mod bot_position;
pub mod line_sensors;

use bot_position::compute_bot_position;
use line_sensors::compute_sensor_readings;

fn print_sensors_data(sensors_data: Res<SensorsData>) {
    println!("line sensors: {:?}", sensors_data.line_sensors);
    println!("bot position: {:?}", sensors_data.bot_position);
}

pub fn add_sensors(app: &mut App) {
    app.insert_resource(SensorsData::default()).add_systems(
        RunFixedMainLoop,
        (
            compute_sensor_readings,
            compute_bot_position,
            print_sensors_data,
        )
            .chain()
            .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
    );
}
