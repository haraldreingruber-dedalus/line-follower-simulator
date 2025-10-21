use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy_editor_cam::DefaultEditorCamPlugins;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;
use executor::wasm_bindings::exports::robot::Configuration;

use crate::bot::add_bot_setup;
use crate::data::add_data;
use crate::motors::add_motors;
use crate::sensors::add_sensors;
use crate::track::add_track;
use crate::ui::add_ui_setup;

#[derive(Resource)]
pub struct BotConfigWrapper {
    pub config: Configuration,
}

impl BotConfigWrapper {
    fn new(config: Configuration) -> Self {
        Self { config }
    }
}

pub enum AppType {
    Simulator(Configuration),
    Test(Configuration),
    Visualizer,
}

pub fn create_app(app_type: AppType, step_period_us: u32) -> App {
    let step_hz = 1_000_000.0 / (step_period_us as f64);
    let mut app = App::new();

    app.add_plugins((
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
    ))
    .insert_resource(Time::<Fixed>::from_hz(step_hz));

    match app_type {
        AppType::Simulator(configuration) => {
            app.add_plugins((
                MinimalPlugins,
                AssetPlugin::default(),
                ScenePlugin::default(),
            ))
            .insert_resource(BotConfigWrapper::new(configuration))
            .init_asset::<Mesh>();

            add_track(&mut app);
            add_bot_setup(&mut app);
            add_motors(&mut app);
            add_sensors(&mut app);

            add_data(&mut app);
        }
        AppType::Test(configuration) => {
            app.add_plugins((
                DefaultPlugins,
                DefaultEditorCamPlugins,
                RapierDebugRenderPlugin::default(),
            ))
            .insert_resource(BotConfigWrapper::new(configuration))
            .insert_resource(Time::from_hz(120.0));

            add_track(&mut app);
            add_bot_setup(&mut app);
            add_motors(&mut app);
            add_sensors(&mut app);

            add_ui_setup(&mut app);
        }
        AppType::Visualizer => {}
    };

    app
}
