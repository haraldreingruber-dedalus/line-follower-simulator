use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy_editor_cam::DefaultEditorCamPlugins;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;
use executor::wasm_bindings::exports::robot::Configuration;

use crate::bot::BotPlugin;
use crate::track::TrackPlugin;
use crate::ui::{CameraSetupPlugin, KeyboardInputTestPlugin};

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

impl AppType {
    pub fn has_physics(&self) -> bool {
        match self {
            AppType::Simulator(_) => true,
            AppType::Test(_) => true,
            AppType::Visualizer => false,
        }
    }

    pub fn has_visualization(&self) -> bool {
        match self {
            AppType::Simulator(_) => false,
            AppType::Test(_) => true,
            AppType::Visualizer => true,
        }
    }

    pub fn get_configuration(&self) -> Option<&Configuration> {
        match self {
            AppType::Simulator(config) => Some(config),
            AppType::Test(config) => Some(config),
            AppType::Visualizer => None,
        }
    }
}

pub struct HeadlessSetupPlugin;

impl Plugin for HeadlessSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ScenePlugin::default(),
        ))
        .init_asset::<Mesh>();
    }
}

pub struct WindowSetupPlugin;

impl Plugin for WindowSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
            .insert_resource(Time::from_hz(120.0))
            .add_plugins(CameraSetupPlugin);
    }
}

pub struct RapierPhysicsSetupPlugin;

impl Plugin for RapierPhysicsSetupPlugin {
    fn build(&self, app: &mut App) {
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
        ));
    }
}

pub fn create_app(app_type: AppType, step_period_us: u32) -> App {
    let mut app = App::new();

    let step_hz = 1_000_000.0 / (step_period_us as f64);
    app.insert_resource(Time::<Fixed>::from_hz(step_hz));

    if app_type.has_visualization() {
        app.add_plugins(WindowSetupPlugin);
    } else {
        app.add_plugins(HeadlessSetupPlugin);
    }

    if app_type.has_physics() {
        app.add_plugins(RapierPhysicsSetupPlugin);
    }

    if let Some(config) = app_type.get_configuration() {
        app.insert_resource(BotConfigWrapper::new(config.clone()));
    }

    app.add_plugins((
        BotPlugin::new(crate::utils::EntityFeatures::Physics),
        TrackPlugin::new(crate::utils::EntityFeatures::Physics),
    ));

    if matches!(app_type, AppType::Test(_)) {
        app.add_plugins(KeyboardInputTestPlugin);
    }

    app
}
