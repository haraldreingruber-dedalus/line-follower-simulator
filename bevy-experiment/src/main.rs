use bevy::prelude::*;
use bevy_editor_cam::DefaultEditorCamPlugins;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;

mod bot;
mod motors;
mod sensors;
mod track;
mod ui;
mod utils;

use crate::bot::add_bot_setup;
use crate::motors::add_motors;
use crate::sensors::add_sensors;
use crate::track::add_track;
use crate::ui::add_ui_setup;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default().with_custom_initialization(
            RapierContextInitialization::InitializeDefaultRapierContext {
                rapier_configuration: {
                    let mut config = RapierConfiguration::new(0.001);
                    config.gravity = Vec3::NEG_Z * 9.81;
                    config
                },
                integration_parameters: IntegrationParameters::default(),
            },
        ),
        DefaultEditorCamPlugins,
        RapierDebugRenderPlugin::default(),
    ))
    .insert_resource(Time::<Fixed>::from_hz(10000.0));

    add_track(&mut app);
    add_bot_setup(&mut app);
    add_ui_setup(&mut app);
    add_sensors(&mut app);
    add_motors(&mut app);

    app.run();
}

// fn test_runner(mut app: App) -> AppExit {
//     loop {
//         println!("In main loop");
//         app.update();
//         if let Some(exit) = app.should_exit() {
//             return exit;
//         }
//     }
// }

// fn main_headless() {
//     App::new()
//         .add_plugins((
//             // DefaultPlugins,
//             MinimalPlugins,
//             AssetPlugin::default(),
//             ScenePlugin::default(),
//             InputPlugin::default(),
//             RapierPhysicsPlugin::<NoUserData>::default().with_custom_initialization(
//                 RapierContextInitialization::InitializeDefaultRapierContext {
//                     rapier_configuration: {
//                         let mut config = RapierConfiguration::new(0.001);
//                         config.gravity = Vec3::NEG_Z * 9.81;
//                         config
//                     },
//                     integration_parameters: IntegrationParameters::default(),
//                 },
//             ),
//             // DefaultEditorCamPlugins,
//             // RapierDebugRenderPlugin::default(),
//             // FrameTimeDiagnosticsPlugin::default(),
//         ))
//         .init_asset::<Mesh>()
//         // Set fixed timestep
//         .insert_resource(Time::<Fixed>::from_hz(10000.0))
//         // Resource for motors pwm values.
//         .insert_resource(MotorsPwm::new())
//         // Define the track layout and spawn it.
//         .insert_resource(Track::new(vec![
//             TrackSegment::start(),
//             TrackSegment::straight(2.0),
//             TrackSegment::ninety_deg_turn(0.5, Side::Right),
//             TrackSegment::cyrcle_turn(1.0, Angle::from_degrees(120.0), Side::Left),
//             TrackSegment::ninety_deg_turn(1.0, Side::Left),
//             TrackSegment::cyrcle_turn(2.0, Angle::from_degrees(60.0), Side::Right),
//             TrackSegment::end(),
//         ]))
//         // Spawn text instructions for keybinds.
//         .add_systems(
//             RunFixedMainLoop,
//             (handle_motors_input, apply_motors_pwm)
//                 .chain()
//                 .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
//         )
//         .add_systems(
//             RunFixedMainLoop,
//             (compute_sensor_readings, compute_bot_position, time_checker)
//                 .chain()
//                 .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
//         )
//         // Add systems for toggling the diagnostics UI and pausing and stepping the simulation.
//         .add_systems(Startup, (setup_bot, setup_track, setup_ui).chain())
//         .set_runner(test_runner)
//         .run();
// }
