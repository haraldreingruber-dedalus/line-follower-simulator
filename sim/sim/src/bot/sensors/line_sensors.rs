use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::track::{TRACK_HALF_WIDTH, TrackSegment};
use crate::utils::{NormalRandom, point_to_new_origin};
use execution_data::SensorsData;

fn line_reflection(x: f32) -> f32 {
    const LINE_SIZE: f32 = 0.02; // 20 mm

    // Model: black line of width LINE_SIZE centered at 0 on a white floor.
    // The sensor doesn't have infinite spatial resolution, so we smooth the
    // transition between black and white across a finite transition region.
    // We return 0.0 for pure black, 100.0 for pure white.

    // Transition width (how quickly the sensor goes from black to white).
    // Using one line-width gives a reasonably soft sensor response; tweak if needed.
    const TRANSITION: f32 = LINE_SIZE;

    // Treat NaN/inf as far away (white)
    if !x.is_finite() {
        return 100.0;
    }

    let half = LINE_SIZE * 0.5;
    let d = x.abs();

    if d <= half {
        // Fully over the black line
        0.0
    } else if d >= half + TRANSITION {
        // Far enough to see full white
        100.0
    } else {
        // Smooth interpolation between black and white using smoothstep
        let t = (d - half) / TRANSITION; // normalized 0..1
        // smoothstep (cubic hermite) -> smooth start/end
        let s = t * t * (3.0 - 2.0 * t);
        100.0 * s
    }
}

trait TrackSimulateLine {
    fn intersection_to_sensor_value(&self, point: Vec3, transform: &GlobalTransform) -> f32;
}

impl TrackSimulateLine for TrackSegment {
    fn intersection_to_sensor_value(&self, point: Vec3, transform: &GlobalTransform) -> f32 {
        let local_point = point_to_new_origin(point, transform);

        match *self {
            TrackSegment::Start | TrackSegment::End => line_reflection(local_point.x),
            TrackSegment::Straight(_) => line_reflection(local_point.x),
            TrackSegment::NinetyDegTurn(data) => {
                let turn_y = (data.line_half_length - TRACK_HALF_WIDTH) / 2.0;
                let dist_to_line = if local_point.y < data.side.sign() * local_point.x + turn_y {
                    local_point.x
                } else {
                    data.side.sign() * (local_point.y - turn_y)
                };
                line_reflection(dist_to_line)
            }
            TrackSegment::CyrcleTurn(data) => {
                let dist_to_line = (local_point.length() - data.radius) * data.side.sign();
                line_reflection(dist_to_line)
            }
        }
    }
}

#[derive(Component, Default)]
pub struct LineSensor {}

pub fn compute_sensor_readings(
    read_rapier_context: ReadRapierContext,
    sensors_query: Query<&GlobalTransform, With<LineSensor>>,
    track_segments_query: Query<(&TrackSegment, &GlobalTransform)>,
    mut rng: ResMut<NormalRandom>,
    mut sensors_data: ResMut<SensorsData>,
) {
    let rapier_context = read_rapier_context.single().unwrap();

    for (i, sensor_tf) in sensors_query.iter().enumerate() {
        const NOISE: f32 = 1.0;
        let origin = sensor_tf.translation();
        let dir = sensor_tf.rotation().mul_vec3(Vec3::NEG_Z);
        let max_toi = 0.1;

        if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            origin,
            dir,
            max_toi,
            true,
            QueryFilter::default().predicate(&|entity| track_segments_query.get(entity).is_ok()),
        ) {
            // Sensor is over the track
            let point: Vec3 = intersection.point.into();
            let (track_segment, transform) = track_segments_query.get(entity).unwrap();
            sensors_data.line_sensors[i] = rng
                .noisy_value(
                    track_segment.intersection_to_sensor_value(point, transform),
                    NOISE,
                )
                .clamp(0.0, 100.0);
        } else {
            // Sensor is out
            sensors_data.line_sensors[i] = rng.noisy_value(100.0, NOISE).clamp(0.0, 100.0);
        }
    }
}
