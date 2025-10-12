use std::f32::consts::{FRAC_PI_2, PI};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::text::cosmic_text::Angle;
use bevy_editor_cam::DefaultEditorCamPlugins;
use bevy_editor_cam::prelude::{EditorCam, OrbitConstraint};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::IntegrationParameters;

const TRACK_HALF_WIDTH: f32 = 0.1;
const TRACK_HALF_HEIGHT: f32 = 0.001;
const TRACK_TIPS_LENGTH: f32 = 0.5;
const TRACK_Z_OFFSET: f32 = -TRACK_HALF_HEIGHT * 2.0;
const TRACK_CIRCLE_SEGMENTS_PER_PI: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn sign(&self) -> f32 {
        match self {
            Side::Left => 1.0,
            Side::Right => -1.0,
        }
    }
}

/// Helper to rotate a Vec2 by angle in radians
/// # Arguments
/// * `v`     - The vector to rotate
/// * `angle` - The angle in radians
fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// Generates a curved "track turn" collider (an arc section)
///
/// # Arguments
/// * `radius`  - Inner radius of the arc
/// * `width`   - Thickness (distance between inner and outer edges)
/// * `angle`   - Total arc angle in radians (e.g., PI/2 for 90° turn)
/// * `height`  - Collider height/thickness
/// * `segments` - Number of convex segments for smoothness
pub fn arc_collider(radius: f32, width: f32, angle: f32, side: Side, height: f32) -> Collider {
    // Approximate the curved arc by composing `segments` small box colliders
    // placed along the arc. Each box is oriented so its long side follows
    // the tangent of the arc. The arc is generated so that angle=0 points
    // in the +Y direction and increases toward +X (so it matches the
    // TrackSegment transform conventions used elsewhere).

    let angle = angle.abs() * side.sign();
    let segments: usize =
        ((TRACK_CIRCLE_SEGMENTS_PER_PI as f32 * angle.abs() / PI).round() as usize).max(1);
    let delta = angle / segments as f32;
    let offset = match side {
        Side::Left => 0.0,
        Side::Right => PI,
    };

    // Collider::compound for bevy_rapier expects parts as (Vec3, Quat, Collider)
    let mut parts: Vec<(Vec3, Quat, Collider)> = Vec::with_capacity(segments);

    for i in 0..segments {
        // angular bounds for this piece
        let theta0 = (i as f32) * delta + offset;
        let theta1 = theta0 + delta;

        let r_in = radius - width / 2.0;
        let r_out = radius + width / 2.0;
        let hz = height * 0.5;

        // build 8 vertices for the prism: inner/out x theta0/theta1 x z-/+
        let mut pts: Vec<Vec3> = Vec::with_capacity(8);

        let p =
            |r: f32, theta: f32, z: f32| -> Vec3 { Vec3::new(r * theta.cos(), r * theta.sin(), z) };

        // inner theta0, z-
        pts.push(p(r_in, theta0, -hz));
        // inner theta0, z+
        pts.push(p(r_in, theta0, hz));
        // inner theta1, z-
        pts.push(p(r_in, theta1, -hz));
        // inner theta1, z+
        pts.push(p(r_in, theta1, hz));

        // outer theta0, z-
        pts.push(p(r_out, theta0, -hz));
        // outer theta0, z+
        pts.push(p(r_out, theta0, hz));
        // outer theta1, z-
        pts.push(p(r_out, theta1, -hz));
        // outer theta1, z+
        pts.push(p(r_out, theta1, hz));

        // place the convex hull at origin; positions are absolute in world-space
        // but Collider::compound wants local translations per part. We'll compute
        // the center of these points and use a local transform so vertices are
        // relative to that center.
        let mut center = Vec3::ZERO;
        for v in &pts {
            center += v;
        }
        center /= pts.len() as f32;

        let rel_pts_vec3: Vec<Vec3> = pts
            .into_iter()
            .map(|v| Vec3::new(v[0] - center.x, v[1] - center.y, v[2] - center.z))
            .collect();

        // Collider::convex_hull commonly accepts a slice of Vec3 and returns
        // an Option<Collider>. Use that if available, otherwise fall back to
        // a cuboid approximation.
        let convex = if let Some(c) = Collider::convex_hull(&rel_pts_vec3) {
            c
        } else {
            Collider::cuboid((r_out - r_in) * 0.5, (radius * delta) * 0.5, hz)
        };

        // The compound part takes translation and rotation; we keep identity
        // rotation because vertices already oriented in world XY plane, and
        // translate by the computed center.
        parts.push((center, Quat::IDENTITY, convex));
    }

    Collider::compound(parts)
}

#[derive(Debug, Clone, Copy)]
struct SegmentTransform {
    position: Vec2,
    direction: Angle,
}

impl SegmentTransform {
    pub fn new(position: Vec2, direction: Angle) -> Self {
        Self {
            position,
            direction,
        }
    }

    pub fn translate_in_direction(&self, translation: Vec2) -> Self {
        Self {
            position: self.position + rotate_vec2(translation, self.direction.to_radians()),
            direction: self.direction,
        }
    }

    pub fn rotate(&self, rotation: Angle) -> Self {
        Self {
            position: self.position,
            direction: Angle::from_radians(self.direction.to_radians() + rotation.to_radians()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StraightSegment {
    length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NinetyDegTurnSegment {
    line_half_length: f32,
    side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CyrcleTurnSegment {
    radius: f32,
    angle: Angle,
    side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
enum TrackSegment {
    Start,
    End,
    Straight(StraightSegment),
    NinetyDegTurn(NinetyDegTurnSegment),
    CyrcleTurn(CyrcleTurnSegment),
}

impl TrackSegment {
    pub fn start() -> Self {
        Self::Start
    }

    pub fn end() -> Self {
        Self::End
    }

    pub fn straight(length: f32) -> Self {
        Self::Straight(StraightSegment { length })
    }

    pub fn ninety_deg_turn(line_half_length: f32, side: Side) -> Self {
        Self::NinetyDegTurn(NinetyDegTurnSegment {
            line_half_length: line_half_length,
            side,
        })
    }

    pub fn cyrcle_turn(radius: f32, angle: Angle, side: Side) -> Self {
        Self::CyrcleTurn(CyrcleTurnSegment {
            radius,
            angle,
            side,
        })
    }

    pub fn collider(&self) -> Collider {
        match *self {
            TrackSegment::Start | TrackSegment::End => {
                Collider::cuboid(TRACK_HALF_WIDTH, TRACK_TIPS_LENGTH / 2.0, TRACK_HALF_HEIGHT)
            }
            TrackSegment::Straight(data) => {
                Collider::cuboid(TRACK_HALF_WIDTH, data.length / 2.0, TRACK_HALF_HEIGHT)
            }
            TrackSegment::NinetyDegTurn(data) => {
                let hl: f32 = (data.line_half_length + TRACK_HALF_WIDTH) / 2.0;
                let ht = (data.line_half_length - TRACK_HALF_WIDTH) / 2.0;
                // Collider::cuboid(hl, hl, TRACK_HALF_HEIGHT);
                Collider::compound(vec![
                    (
                        Vec3::ZERO,
                        Quat::IDENTITY,
                        Collider::cuboid(TRACK_HALF_WIDTH, hl, TRACK_HALF_HEIGHT),
                    ),
                    (
                        Vec3::new(ht * -data.side.sign(), ht, 0.0),
                        Quat::from_rotation_z(FRAC_PI_2),
                        Collider::cuboid(TRACK_HALF_WIDTH, hl, TRACK_HALF_HEIGHT),
                    ),
                ])
            }
            TrackSegment::CyrcleTurn(data) => arc_collider(
                data.radius,
                TRACK_HALF_WIDTH * 2.0,
                data.angle.to_radians(),
                data.side,
                TRACK_HALF_HEIGHT * 2.0,
            ),
        }
    }

    pub fn transform(&self, origin: SegmentTransform) -> Transform {
        let transform_origin = match *self {
            TrackSegment::Start | TrackSegment::End => {
                origin.translate_in_direction(Vec2::Y * TRACK_TIPS_LENGTH / 2.0)
            }
            TrackSegment::Straight(data) => {
                origin.translate_in_direction(Vec2::Y * data.length / 2.0)
            }
            TrackSegment::NinetyDegTurn(data) => origin
                .translate_in_direction(Vec2::Y * (data.line_half_length + TRACK_HALF_WIDTH) / 2.0),
            TrackSegment::CyrcleTurn(data) => {
                origin.translate_in_direction(Vec2::NEG_X * data.radius * data.side.sign())
            }
        };
        Transform::from_translation(transform_origin.position.extend(TRACK_Z_OFFSET)).with_rotation(
            Quat::from_rotation_z(transform_origin.direction.to_radians()),
        )
    }

    pub fn compute_next_origin(&self, origin: SegmentTransform) -> SegmentTransform {
        match *self {
            TrackSegment::Start | TrackSegment::End => {
                origin.translate_in_direction(Vec2::Y * TRACK_TIPS_LENGTH)
            }
            TrackSegment::Straight(data) => origin.translate_in_direction(Vec2::Y * data.length),
            TrackSegment::NinetyDegTurn(data) => origin
                .translate_in_direction(Vec2::new(
                    -data.line_half_length * data.side.sign(),
                    data.line_half_length,
                ))
                .rotate(Angle::from_degrees(90.0 * data.side.sign())),
            TrackSegment::CyrcleTurn(data) => origin
                .translate_in_direction(Vec2::new(
                    data.radius * (data.angle.to_radians().cos() - 1.0) * data.side.sign(),
                    data.radius * data.angle.to_radians().sin(),
                ))
                .rotate(Angle::from_radians(
                    data.angle.to_radians() * data.side.sign(),
                )),
        }
    }
}

#[derive(Resource)]
struct Track {
    origin: SegmentTransform,
    segments: Vec<TrackSegment>,
}

impl Track {
    pub fn new(segments: Vec<TrackSegment>) -> Self {
        Self {
            origin: SegmentTransform::new(Vec2::NEG_Y * TRACK_TIPS_LENGTH / 2.0, Angle::ZERO),
            segments,
        }
    }

    pub fn spawn_bundles(&self, mut commands: Commands) {
        let mut segment_origin = self.origin;

        for segment in &self.segments {
            commands.spawn((
                segment.collider(),
                *segment,
                segment.transform(segment_origin),
                RigidBody::Fixed,
                Friction {
                    coefficient: 0.5,
                    combine_rule: CoefficientCombineRule::Average,
                },
            ));
            segment_origin = segment.compute_next_origin(segment_origin);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WheelSide {
    Left,
    Right,
}

impl WheelSide {
    pub fn sign(&self) -> f32 {
        match self {
            WheelSide::Left => 1.0,
            WheelSide::Right => -1.0,
        }
    }
}

#[derive(Resource)]
struct MotorsTorque {
    left_torque: f32,
    right_torque: f32,
}

impl MotorsTorque {
    pub fn new() -> Self {
        Self {
            left_torque: 0.0,
            right_torque: 0.0,
        }
    }

    pub fn torque(&self, side: WheelSide) -> f32 {
        match side {
            WheelSide::Left => self.left_torque,
            WheelSide::Right => self.right_torque,
        }
    }
}

#[derive(Component)]
struct Motors {
    left_axle: Vec3,
    right_axle: Vec3,
}

#[derive(Component)]
struct Wheel {
    axle: Vec3,
    side: WheelSide,
}

fn handle_motors_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut torque: ResMut<MotorsTorque>,
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let forward = if up {
        -1.0
    } else if down {
        1.0
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

    const FORWARD_TORQUE: f32 = 10.1;
    const SIDE_TORQUE: f32 = 10.1;

    torque.left_torque = forward * FORWARD_TORQUE + side * SIDE_TORQUE;
    torque.right_torque = forward * FORWARD_TORQUE - side * SIDE_TORQUE;
}

fn set_wheel_torque(
    torque: Res<MotorsTorque>,
    mut query: Query<(&Wheel, &Transform, &mut ExternalForce)>,
) {
    for (wheel, transform, mut ext_impulse) in &mut query {
        let torque = torque.torque(wheel.side) * wheel.side.sign();
        let wheel_axle = transform.rotation * wheel.axle;
        ext_impulse.torque = wheel_axle * torque;
    }
}

fn set_motors_torque(
    torque: Res<MotorsTorque>,
    mut query: Query<(&Motors, &Transform, &mut ExternalForce)>,
) {
    for (motors, transform, mut ext_torque) in &mut query {
        let left_torque = torque.left_torque * WheelSide::Left.sign() * -1.0;
        let left_axle = transform.rotation * motors.left_axle;
        let right_torque = torque.right_torque * WheelSide::Right.sign() * -1.0;
        let right_axle = transform.rotation * motors.right_axle;
        ext_torque.torque = (left_axle * left_torque) + (right_axle * right_torque);
    }
}

fn _ray_cast_example(
    read_rapier_context: ReadRapierContext,
    body_query: Query<(&Motors, &GlobalTransform)>,
    gt_query: Query<&GlobalTransform>,
) {
    for (_, body_tf) in body_query {
        let origin = body_tf.translation();
        let dir = body_tf.rotation().mul_vec3(Vec3::X);
        let max_toi = 10.0;

        let rapier_context = read_rapier_context.single().unwrap();

        if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            origin,
            dir,
            max_toi,
            true,
            QueryFilter::default(),
        ) {
            let point: Vec3 = intersection.point.into();

            let gt = gt_query.get(entity).unwrap();

            println!(
                "Ray from {:?} hit {:?} at {} gt {:?}",
                origin, entity, point, gt
            );
        } else {
            println!("Ray from {:?} hit nothing", origin);
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default().with_custom_initialization(
                RapierContextInitialization::InitializeDefaultRapierContext {
                    rapier_configuration: {
                        let mut config = RapierConfiguration::new(1f32);
                        config.gravity = Vec3::NEG_Z * 9.81;
                        config
                    },
                    integration_parameters: IntegrationParameters {
                        length_unit: 1f32,
                        ..default()
                    },
                },
            ),
            DefaultEditorCamPlugins,
            RapierDebugRenderPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        // Add gravity to the physics simulation.
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        // Resource for motors torque values.
        .insert_resource(MotorsTorque::new())
        // Define the track layout and spawn it.
        .insert_resource(Track::new(vec![
            TrackSegment::start(),
            TrackSegment::straight(2.0),
            TrackSegment::ninety_deg_turn(0.5, Side::Right),
            TrackSegment::cyrcle_turn(1.0, Angle::from_degrees(120.0), Side::Left),
            TrackSegment::ninety_deg_turn(1.0, Side::Left),
            TrackSegment::cyrcle_turn(2.0, Angle::from_degrees(60.0), Side::Right),
            TrackSegment::end(),
        ]))
        // Spawn text instructions for keybinds.
        .add_systems(
            RunFixedMainLoop,
            (handle_motors_input, set_wheel_torque, set_motors_torque)
                .chain()
                .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
        )
        // .add_systems(
        //     RunFixedMainLoop,
        //     ray_cast_example.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
        // )
        // Add systems for toggling the diagnostics UI and pausing and stepping the simulation.
        .add_systems(Startup, (setup_bot, setup_track, setup_ui).chain())
        .run();
}

// Robot configuration structure:

// pub struct Configuration {
//     /// Robot name
//     pub name: _rt::String,
//     /// Main color
//     pub color_main: Color,
//     /// Secondary color
//     pub color_secondary: Color,
//     /// Axle width from wheel to wheel (in mm, 100 to 200)
//     pub width_axle: f32,
//     /// Length from wheel axles to front (in mm, 100 to 300)
//     pub length_front: f32,
//     /// Length from wheel axles to back (in mm, 10 to 50)
//     pub length_back: f32,
//     /// Clearing from robot to ground at the robot back (in mm, from 1 to wheels radius)
//     pub clearing_back: f32,
//     /// Diameter of robot wheels (in mm, from 20 to 40)
//     pub wheel_diameter: f32,
//     /// Transmission gear ratio numerator (from 1 to 100)
//     pub gear_ratio_num: u32,
//     /// Transmission gear ratio denumerator (from 1 to 100)
//     pub gear_ratio_den: u32,
//     /// Spacing of line sensors (in mm, from 1 to 15)
//     pub front_sensors_spacing: f32,
//     /// Height of line sensors from the ground (in mm, from 1 to wheels radius)
//     pub front_sensors_height: f32,
// }

const BOT_WHEEL_WIDTH: f32 = 0.5;
const BOT_BODY_LENGHT_MIN: f32 = 0.04;
const BOT_BODY_LENGHT_PERCENT_OF_TOTAL: f32 = 0.6;
const BOT_BODY_WIDTH: f32 = 0.09;
const BOT_BODY_HEIGHT: f32 = 0.02;
const BOT_OFFSET_Y_FRONT_LENGHT_PERCENT: f32 = 0.1;

const BOT_BUMPER_DIAMETER: f32 = BOT_BODY_HEIGHT / 2.0;
const BOT_BUMPER_WIDTH: f32 = BOT_BODY_WIDTH / 2.0;

fn setup_bot(mut commands: Commands) {
    // Axle width from wheel to wheel (in mm, 100 to 200)
    let width_axle: f32 = 100.0 / 1000.0;
    // Length from wheel axles to front (in mm, 100 to 300)
    let length_front: f32 = 100.0 / 1000.0;
    // Length from wheel axles to back (in mm, 10 to 50)
    let length_back: f32 = 20.0 / 1000.0;
    // Clearing from robot to ground at the robot back (in mm, from 1 to wheels radius)
    let clearing_back: f32 = 10.0 / 1000.0;
    // Diameter of robot wheels (in mm, from 20 to 40)
    let wheel_diameter: f32 = 20.0 / 1000.0;
    // Transmission gear ratio numerator (from 1 to 100)
    let gear_ratio_num: u32 = 1;
    // Transmission gear ratio denumerator (from 1 to 100)
    let gear_ratio_den: u32 = 1;
    // Spacing of line sensors (in mm, from 1 to 15)
    let front_sensors_spacing: f32 = 15.0 / 1000.0;
    // Height of line sensors from the ground (in mm, from 1 to wheels radius)
    let front_sensors_height: f32 = 2.0 / 1000.0;

    let body_offset_z: f32 = clearing_back;

    // Static body with motors
    let body = commands
        .spawn((
            Collider::cuboid(
                BOT_BODY_WIDTH * 0.5,
                (BOT_BODY_LENGHT_MIN
                    + BOT_BODY_LENGHT_PERCENT_OF_TOTAL * (length_front + length_back))
                    * 0.5,
                BOT_BODY_HEIGHT * 0.5,
            ),
            RigidBody::Dynamic,
            Friction {
                coefficient: 0.1,
                combine_rule: CoefficientCombineRule::Min,
            },
            ColliderMassProperties::Density(1.0),
            Transform::from_xyz(
                0.0,
                BOT_OFFSET_Y_FRONT_LENGHT_PERCENT * length_front,
                (body_offset_z + BOT_BODY_HEIGHT) * 0.5,
            ),
            GlobalTransform::default(),
            Motors {
                left_axle: Vec3::X,
                right_axle: Vec3::NEG_X,
            },
            ExternalForce::default(),
            Velocity::zero(),
        ))
        .id();

    commands.spawn((
        Collider::compound(vec![(
            Vec3::ZERO,
            Quat::from_rotation_z(FRAC_PI_2),
            Collider::cylinder(BOT_BUMPER_WIDTH / 2.0, BOT_BUMPER_DIAMETER / 2.0),
        )]),
        RigidBody::Dynamic,
        Friction {
            coefficient: 0.1,
            combine_rule: CoefficientCombineRule::Min,
        },
        Transform::from_xyz(0.0, length_front, BOT_BUMPER_DIAMETER / 2.0),
        GlobalTransform::default(),
        ImpulseJoint::new(
            body,
            FixedJointBuilder::new()
                .local_anchor1(Vec3::new(0.0, length_front, BOT_BUMPER_DIAMETER / 2.0))
                .local_anchor2(Vec3::new(
                    0.0,
                    BOT_OFFSET_Y_FRONT_LENGHT_PERCENT * length_front,
                    body_offset_z + BOT_BODY_HEIGHT * 0.5,
                )),
        ),
    ));

    // let wheel_joints_x = 1.5;

    // let left_wheel_joint: RevoluteJointBuilder = RevoluteJointBuilder::new(Vec3::Y)
    //     .local_anchor1(Vec3::new(wheel_joints_x, 0.5, 0.0))
    //     .local_anchor2(Vec3::new(0.0, -0.5, 0.0));
    // let _left_wheel = commands
    //     .spawn((
    //         Collider::cylinder(0.5, 0.5),
    //         Transform::from_xyz(wheel_joints_x, 1.0, 0.0),
    //         GlobalTransform::default(),
    //         RigidBody::Dynamic,
    //         Friction {
    //             coefficient: 0.95,
    //             combine_rule: CoefficientCombineRule::Max,
    //         },
    //         ColliderMassProperties::Density(1.0),
    //         Wheel {
    //             axle: Vec3::Y,
    //             side: WheelSide::Left,
    //         },
    //         ExternalForce::default(),
    //         ImpulseJoint::new(body, left_wheel_joint),
    //     ))
    //     .id();

    // let right_wheel_joint = RevoluteJointBuilder::new(Vec3::Y)
    //     .local_anchor1(Vec3::new(wheel_joints_x, -0.5, 0.0))
    //     .local_anchor2(Vec3::new(0.0, 0.5, 0.0));
    // let _right_wheel = commands
    //     .spawn((
    //         Collider::cylinder(0.5, 0.5),
    //         Transform::from_xyz(wheel_joints_x, -1.0, 0.0),
    //         GlobalTransform::default(),
    //         RigidBody::Dynamic,
    //         Friction {
    //             coefficient: 0.95,
    //             combine_rule: CoefficientCombineRule::Max,
    //         },
    //         ColliderMassProperties::Density(1.0),
    //         Wheel {
    //             axle: Vec3::NEG_Y,
    //             side: WheelSide::Right,
    //         },
    //         ExternalForce::default(),
    //         ImpulseJoint::new(body, right_wheel_joint),
    //     ))
    //     .id();
}

fn setup_track(commands: Commands, track: Res<Track>) {
    track.spawn_bundles(commands);

    // Static floor
    // let _floor = commands
    //     .spawn((
    //         Collider::cuboid(0.5, 0.5, 0.5),
    //         RigidBody::Fixed,
    //         Friction::new(0.5),
    //         Transform::from_xyz(0.0, 0.0, -4.0).with_scale(Vec3::new(50.0, 50.0, 0.1)),
    //     ))
    //     .id();
}

fn setup_ui(mut commands: Commands) {
    // // Directional light
    // commands.spawn((
    //     DirectionalLight {
    //         illuminance: 2000.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::default().looking_at(Vec3::new(-1.0, -1.5, -2.5), Vec3::Z),
    // ));

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
        Transform::from_translation(Vec3::Z * 1.0).looking_at(
            Vec3::Y,
            Vec3 {
                x: 0.0,
                y: 0.5,
                z: 1.0,
            },
        ),
    ));
}
