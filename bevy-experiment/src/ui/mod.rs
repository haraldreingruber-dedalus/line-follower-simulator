use bevy_editor_cam::prelude::{EditorCam, OrbitConstraint};

use bevy::{prelude::*, render::view::RenderLayers};
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext,
    egui::{self, Color32},
};

use crate::motors::MotorsPwm;

fn handle_motors_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut pwm: ResMut<MotorsPwm>) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let forward = if up {
        1.0
    } else if down {
        -1.0
    } else {
        0.0
    };
    let side = if left {
        -1.0
    } else if right {
        1.0
    } else {
        0.0
    };

    const MAX_PWM: f32 = 1.0;
    const USE_PWM: f32 = 1.0;

    pwm.left_pwm = (forward * USE_PWM + side * USE_PWM).clamp(-MAX_PWM, MAX_PWM);
    pwm.right_pwm = (forward * USE_PWM - side * USE_PWM).clamp(-MAX_PWM, MAX_PWM);
}

fn setup_ui(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        EditorCam {
            orbit_constraint: OrbitConstraint::Fixed {
                up: Vec3::Z,
                can_pass_tdc: false,
            },
            ..Default::default()
        },
        Transform::from_translation(Vec3::X * 0.5).looking_at(
            Vec3::X,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.1,
            },
        ),
    ));
}

pub fn add_ui_setup(app: &mut App) {
    app.add_systems(Startup, setup_ui)
        .add_systems(Update, handle_motors_input)
        // egui support
        .add_plugins(EguiPlugin::default())
        // gui setup
        .add_systems(Startup, setup_gui)
        // gui implementation
        .add_systems(EguiPrimaryContextPass, gui_example)
        // Background color
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));
}

fn gui_example(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;

    ctx.style_mut(|style| style.visuals.panel_fill = Color32::from_rgba_unmultiplied(0, 0, 0, 0));

    egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(false)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.label("Bottom fixed panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    egui::SidePanel::left("left_panel")
        .resizable(false)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.label("Left fixed panel");
        });

    egui::SidePanel::right("right_panel")
        .resizable(false)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.label("Right fixed panel");
        });

    Ok(())
}

fn setup_gui(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) {
    // Disable the automatic creation of a primary context to set it up manually for the camera we need.
    egui_global_settings.auto_create_primary_context = false;

    // Egui camera.
    commands.spawn((
        // The `PrimaryEguiContext` component requires everything needed to render a primary context.
        PrimaryEguiContext,
        Camera2d,
        // Setting RenderLayers to none makes sure we won't render anything apart from the UI.
        RenderLayers::none(),
        Camera {
            order: 1,
            ..default()
        },
    ));
}
