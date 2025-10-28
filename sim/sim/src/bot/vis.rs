use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

use bevy::ecs::system::Commands;
use bevy::prelude::*;
use execution_data::{BodyExecutionData, WheelExecutionData};
use executor::wasm_bindings::exports::robot::Configuration;

use crate::utils::Side;

use super::motors::Wheel;
use super::{BotBodyMarker, BotConfigurationResource};

pub struct BotMeshes {
    pub cube: Handle<Mesh>,
    pub cylinder: Handle<Mesh>,
}

pub struct BotMaterials {
    pub black: Handle<StandardMaterial>,
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
    let cube_mesh = meshes.add(Cuboid::from_size(Vec3::ONE));
    let cylinder_mesh = meshes.add(Cylinder::new(0.5, 1.0));

    let black_material = materials.add(Color::srgb(0.0, 0.0, 0.0));

    let assets = BotAssets {
        meshes: BotMeshes {
            cube: cube_mesh.clone(),
            cylinder: cylinder_mesh.clone(),
        },
        materials: BotMaterials {
            black: black_material.clone(),
        },
    };

    commands.insert_resource(assets);
}

trait SetupColorMaterials {
    fn setup_color_materials(
        &self,
        materials: &mut Assets<StandardMaterial>,
    ) -> (Handle<StandardMaterial>, Handle<StandardMaterial>);
}

impl SetupColorMaterials for Configuration {
    fn setup_color_materials(
        &self,
        materials: &mut Assets<StandardMaterial>,
    ) -> (Handle<StandardMaterial>, Handle<StandardMaterial>) {
        let color_main = Color::srgb(
            self.color_main.r as f32 / u8::max_value() as f32,
            self.color_main.g as f32 / u8::max_value() as f32,
            self.color_main.b as f32 / u8::max_value() as f32,
        );
        let color_secondary = Color::srgb(
            self.color_secondary.r as f32 / u8::max_value() as f32,
            self.color_secondary.g as f32 / u8::max_value() as f32,
            self.color_secondary.b as f32 / u8::max_value() as f32,
        );

        (materials.add(color_main), materials.add(color_secondary))
    }
}

pub fn spawn_bot_body(
    commands: &mut Commands,
    parent: Entity,
    configuration: &Configuration,
    assets: &BotAssets,
    materials: &mut Assets<StandardMaterial>,
    data: Option<BodyExecutionData>,
) -> Entity {
    let id = commands.spawn((ChildOf(parent), Transform::default())).id();
    if let Some(data) = data {
        commands.entity(id).insert(data);
    }

    let (color_main_material, color_secondary_material) =
        configuration.setup_color_materials(materials);

    let wheel_diameter = configuration.wheel_diameter / 1000.0;

    const BODY_TO_WHEEL: f32 = 0.005;
    let body_width = configuration.width_axle / 1000.0 - 2.0 * BODY_TO_WHEEL;

    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cube.clone()),
        MeshMaterial3d(color_main_material.clone()),
        Transform::from_scale(Vec3::new(
            body_width,
            wheel_diameter * 2.0,
            wheel_diameter / 2.0,
        )),
    ));

    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(color_main_material.clone()),
        Transform::from_scale(Vec3::new(
            wheel_diameter * 0.7,
            body_width,
            wheel_diameter * 0.7,
        ))
        .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));

    // axle
    let axle_d = 0.003;
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_scale(Vec3::new(axle_d, configuration.width_axle / 1000.0, axle_d))
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));
    id
}

pub fn spawn_bot_wheel(
    commands: &mut Commands,
    parent: Entity,
    configuration: &Configuration,
    assets: &BotAssets,
    materials: &mut Assets<StandardMaterial>,
    side: Side,
    data: Option<WheelExecutionData>,
) {
    let wheel_world = Vec3::new((configuration.width_axle / 2000.0) * -side.sign(), 0.0, 0.0);

    let (_, color_secondary_material) = configuration.setup_color_materials(materials);

    let transform = data
        .as_ref()
        .map(|data| {
            let t = Transform::from_translation(wheel_world);
            println!("wheel transform {} {:?}", data.side, t);
            t
        })
        .unwrap_or_default();

    let id = commands.spawn((ChildOf(parent), transform)).id();

    if let Some(data) = data {
        commands.entity(id).insert(data);
    }

    let wheel_d = configuration.wheel_diameter / 1000.0;
    let wheel_w = 0.02; // wheel_d * 3.0 / 2.0;

    // cylinder mesh
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(color_secondary_material.clone()),
        Transform::from_translation(Vec3::X * -side.sign() * wheel_w / 2.0)
            .with_scale(Vec3::new(wheel_d, wheel_w, wheel_d))
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));

    // ext drawing
    let drawing_out = 0.001;
    let ext_cyl_tranform = Vec3::new(wheel_d / 3.5, drawing_out / 2.0, wheel_d / 2.0);
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_translation(Vec3::new(-side.sign() * wheel_w, wheel_d / 4.0, 0.0))
            .with_scale(ext_cyl_tranform)
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_translation(Vec3::new(
            -side.sign() * wheel_w,
            -(wheel_d / 4.0) * FRAC_PI_3.cos(),
            -(wheel_d / 4.0) * FRAC_PI_3.sin(),
        ))
        .with_scale(ext_cyl_tranform)
        .with_rotation(Quat::from_euler(EulerRot::XYZ, FRAC_PI_3, 0.0, FRAC_PI_2)),
    ));
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_translation(Vec3::new(
            -side.sign() * wheel_w,
            -(wheel_d / 4.0) * FRAC_PI_3.cos(),
            (wheel_d / 4.0) * FRAC_PI_3.sin(),
        ))
        .with_scale(ext_cyl_tranform)
        .with_rotation(Quat::from_euler(EulerRot::XYZ, -FRAC_PI_3, 0.0, FRAC_PI_2)),
    ));
}

pub fn setup_test_bot_visualizer(
    mut commands: Commands,
    assets: Res<BotAssets>,
    configuration: Res<BotConfigurationResource>,
    body_query: Query<Entity, With<BotBodyMarker>>,
    wheels_query: Query<(Entity, &Wheel)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cfg = configuration.cfg();

    let body = body_query.single().unwrap();
    spawn_bot_body(&mut commands, body, &cfg, &assets, &mut materials, None);

    for (wheel_id, wheel) in wheels_query.iter() {
        spawn_bot_wheel(
            &mut commands,
            wheel_id,
            &cfg,
            &assets,
            &mut materials,
            wheel.side,
            None,
        );
    }
}
