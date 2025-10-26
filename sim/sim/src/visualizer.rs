use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    pbr::StandardMaterial,
    render::mesh::Mesh,
    transform::components::Transform,
};
use execution_data::ExecutionData;
use executor::wasm_bindings::exports::robot::Configuration;

use crate::{
    bot::vis::{BotAssets, spawn_bot_body, spawn_bot_wheel},
    track::{Track, setup_track},
    ui::RunnerGuiState,
    utils::EntityFeatures,
};

#[derive(Component)]
pub struct BotVisualization {
    config: Configuration,
    bot_number: usize,
}

const VIS_LAYER_Z_STEP: f32 = 0.5;

pub fn spawn_bot_visualization(
    mut commands: Commands,
    track: &Track,
    data: ExecutionData,
    configuration: Configuration,
    bot_number: usize,
    bot_assets: &BotAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let vis_transform = Transform::from_xyz(0.0, 0.0, bot_number as f32 * VIS_LAYER_Z_STEP);
    let vis = commands
        .spawn((
            BotVisualization {
                config: configuration.clone(),
                bot_number,
            },
            vis_transform,
        ))
        .id();
    setup_track(
        &mut commands,
        vis,
        EntityFeatures::Visualization,
        track,
        meshes,
        materials,
    );
    let bot = spawn_bot_body(
        &mut commands,
        vis,
        &configuration,
        bot_assets,
        Some(data.body_data),
    );
    spawn_bot_wheel(
        &mut commands,
        bot,
        &configuration,
        bot_assets,
        Some(data.left_wheel_data),
    );
    spawn_bot_wheel(
        &mut commands,
        bot,
        &configuration,
        bot_assets,
        Some(data.right_wheel_data),
    );
}

pub fn sync_visualizer_time(
    vis_query: Query<(Entity, &BotVisualization)>,
    runner_gui_state: Res<RunnerGuiState>,
) {
    todo!()
}
