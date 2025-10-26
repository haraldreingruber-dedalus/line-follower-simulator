use bevy::ecs::system::Commands;
use bevy::prelude::*;
use execution_data::{BodyExecutionData, WheelExecutionData};
use executor::wasm_bindings::exports::robot::Configuration;

use super::motors::Wheel;
use super::{BotBodyMarker, BotConfigurationResource};

pub struct BotMeshes {
    pub body: Handle<Mesh>,
    pub wheel: Handle<Mesh>,
}

pub struct BotMaterials {
    pub body: Handle<StandardMaterial>,
    pub wheel: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct BotAssets {
    pub meshes: BotMeshes,
    pub materials: BotMaterials,
}

pub fn setup_bot_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body_mesh = meshes.add(Cuboid::from_size(Vec3::new(0.1, 0.2, 0.02)));
    let body_material = materials.add(Color::srgb(0.8, 0.2, 0.2));

    let wheel_mesh = meshes.add(Cuboid::from_size(Vec3::new(0.01, 0.03, 0.03)));
    let wheel_material = materials.add(Color::srgb(0.2, 0.8, 0.2));

    let assets = BotAssets {
        meshes: BotMeshes {
            body: body_mesh.clone(),
            wheel: wheel_mesh.clone(),
        },
        materials: BotMaterials {
            body: body_material.clone(),
            wheel: wheel_material.clone(),
        },
    };

    commands.insert_resource(assets);
}

pub fn spawn_bot_body(
    commands: &mut Commands,
    parent: Entity,
    configuration: &Configuration,
    assets: &BotAssets,
    data: Option<BodyExecutionData>,
) -> Entity {
    let id = commands
        .spawn((
            ChildOf(parent),
            Mesh3d(assets.meshes.body.clone()),
            MeshMaterial3d(assets.materials.body.clone()),
        ))
        .id();
    if let Some(data) = data {
        commands.entity(id).insert(data);
    }
    id
}

pub fn spawn_bot_wheel(
    commands: &mut Commands,
    parent: Entity,
    configuration: &Configuration,
    assets: &BotAssets,
    data: Option<WheelExecutionData>,
) {
    let id = commands
        .spawn((
            ChildOf(parent),
            Mesh3d(assets.meshes.wheel.clone()),
            MeshMaterial3d(assets.materials.wheel.clone()),
        ))
        .id();
    if let Some(data) = data {
        commands.entity(id).insert(data);
    }
}

pub fn setup_test_bot_visualizer(
    mut commands: Commands,
    assets: Res<BotAssets>,
    configuration: Res<BotConfigurationResource>,
    body_query: Query<Entity, With<BotBodyMarker>>,
    wheels_query: Query<(Entity, &Wheel)>,
) {
    let cfg = configuration.cfg();

    let body = body_query.single().unwrap();
    spawn_bot_body(&mut commands, body, &cfg, &assets, None);

    for (wheel_id, wheel) in wheels_query.iter() {
        spawn_bot_wheel(&mut commands, wheel_id, &cfg, &assets, None);
    }
}
