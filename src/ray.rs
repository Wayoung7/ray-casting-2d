use std::{cmp::Ordering, f32::consts::PI};

use bevy::prelude::*;

use bevy_prototype_lyon::prelude::*;

use crate::obstacle::Obstacle;

const EPS: f32 = 0.00001;
const ANGLE_OFFSET: f32 = 0.005;

pub struct RayPlugin;

impl Plugin for RayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            .add_systems(Startup, setup_rays)
            .add_systems(PreUpdate, update_rays)
            .add_systems(Update, handle_collision)
            .add_systems(PostUpdate, update_snap_to_cursor);
    }
}

#[derive(Component, Debug)]
pub struct CastRays {
    pub num_rays: u8,
    pub rays: Vec<Ray>,
}

impl CastRays {
    fn new(num_rays: u8) -> Self {
        let mut rays: Vec<Ray> = Vec::new();
        let angle_diff = 2. * PI / num_rays as f32;
        for i in 0..num_rays {
            rays.push(Ray {
                start_point: Vec2::ZERO,
                end_point: Vec2::from_angle(angle_diff * i as f32) * 100000.,
            });
        }
        CastRays { num_rays, rays }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub start_point: Vec2,
    pub end_point: Vec2,
}

impl Default for Ray {
    fn default() -> Self {
        Ray {
            start_point: Vec2::ZERO,
            end_point: Vec2::ZERO,
        }
    }
}

#[allow(unused)]
impl Ray {
    fn new(start_point: Vec2, end_point: Vec2) -> Self {
        Ray {
            start_point,
            end_point,
        }
    }

    pub fn set_source_pos(&mut self, pos: Vec2) {
        self.start_point = pos;
    }

    pub fn set_end_pos(&mut self, pos: Vec2) {
        self.end_point = pos;
    }

    pub fn translate(&mut self, vec: Vec2) {
        self.start_point += vec;
        self.end_point += vec;
    }

    pub fn translate_to(&mut self, point: Vec2) {
        let offset: Vec2 = point - self.start_point;
        self.start_point += offset;
        self.end_point += offset;
    }

    pub fn scale(&mut self, value: f32) {
        let vec = self.end_point - self.start_point;
        self.end_point = self.start_point + vec * value;
    }
}

fn setup_rays(mut commands: Commands) {
    commands.spawn((CastRays::new(18), SnapToCursor, Transform::default()));
}

fn update_rays(
    mut ray_query: Query<(&mut CastRays, &Transform)>,
    obstacle_query: Query<(&Obstacle, &Transform)>,
) {
    for (mut ray_source, transform) in ray_query.iter_mut() {
        let current_start_point = transform.translation.xy();
        ray_source.rays.clear();
        for (obstacle, obs_transform) in obstacle_query.iter() {
            let start_angle = cal_angle_from_points(
                current_start_point,
                obs_transform.translation.xy() + obstacle.start_offset,
            );
            ray_source.rays.push(Ray::new(
                current_start_point,
                obs_transform.translation.xy() + obstacle.start_offset,
            ));
            ray_source.rays.push(Ray::new(
                current_start_point,
                current_start_point + Vec2::from_angle(start_angle + ANGLE_OFFSET),
            ));
            ray_source.rays.push(Ray::new(
                current_start_point,
                current_start_point + Vec2::from_angle(start_angle - ANGLE_OFFSET),
            ));
            let end_angle = cal_angle_from_points(
                current_start_point,
                obs_transform.translation.xy() + obstacle.end_offset,
            );
            ray_source.rays.push(Ray::new(
                current_start_point,
                obs_transform.translation.xy() + obstacle.end_offset,
            ));
            ray_source.rays.push(Ray::new(
                current_start_point,
                current_start_point + Vec2::from_angle(end_angle + ANGLE_OFFSET),
            ));
            ray_source.rays.push(Ray::new(
                current_start_point,
                current_start_point + Vec2::from_angle(end_angle - ANGLE_OFFSET),
            ));
        }
    }
}

#[derive(Component, Debug)]
pub struct SnapToCursor;

fn update_snap_to_cursor(
    mut object_query: Query<&mut Transform, With<SnapToCursor>>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = camera_query.single();
    let Some(cursor_position) = window_query.single().cursor_position() else {
        return;
    };
    let Some(cursor_point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for mut object_transform in object_query.iter_mut() {
        object_transform.translation = Vec3::new(cursor_point.x, cursor_point.y, 0.);
    }
}

#[derive(Component, Debug)]
pub struct IllumTri;

fn handle_collision(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut ray_query: Query<(&mut CastRays, &Transform)>,
    obstacle_query: Query<(&Obstacle, &Transform)>,
    illum_query: Query<Entity, With<IllumTri>>,
) {
    for tri_entity in illum_query.iter() {
        commands.entity(tri_entity).despawn();
    }
    let mut collision: Vec<Vec2> = Vec::new();
    for (mut ray_source, transform) in ray_query.iter_mut() {
        for ray in ray_source.rays.iter_mut() {
            let mut collision_one_ray: Vec<(Vec2, f32)> = Vec::new();
            let (x1, y1): (f32, f32) = (ray.start_point.x, ray.start_point.y);
            let (x2, y2): (f32, f32) = (ray.end_point.x, ray.end_point.y);
            for (obstacle, obs_transform) in obstacle_query.iter() {
                let (x3, y3): (f32, f32) = (
                    obs_transform.translation.x + obstacle.start_offset.x,
                    obs_transform.translation.y + obstacle.start_offset.y,
                );
                let (x4, y4): (f32, f32) = (
                    obs_transform.translation.x + obstacle.end_offset.x,
                    obs_transform.translation.y + obstacle.end_offset.y,
                );
                let den: f32 = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
                let t: f32 = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / den;
                let u: f32 = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / den;
                if 0. - EPS < u && u < 1. + EPS && t > 0. {
                    let collision_point = Vec2::new(x1 + t * (x2 - x1), y1 + t * (y2 - y1));
                    collision_one_ray.push((collision_point, t));
                }
            }
            if let Some(nearest) = collision_one_ray
                .into_iter()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            {
                collision.push(nearest.0);
                ray.set_end_pos(nearest.0);
            }
        }
        // render_collision_point(&mut gizmos, &collision);
        collision.sort_by(|a, b| {
            cal_angle_from_points(transform.translation.xy(), *a)
                .partial_cmp(&cal_angle_from_points(transform.translation.xy(), *b))
                .unwrap_or(Ordering::Equal)
        });
        if let Some(first) = collision.first() {
            collision.push(*first);
            for pair in collision.windows(2) {
                let points = vec![pair[0], pair[1], transform.translation.xy()];
                let shape = shapes::Polygon {
                    points: points.clone(),
                    closed: true,
                };
                commands.spawn((
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shape),
                        ..default()
                    },
                    IllumTri,
                    Fill::color(Color::hex("#ffb327").unwrap()),
                ));
            }
        };
    }
}

fn render_collision_point(gizmos: &mut Gizmos, collision_data: &Vec<Vec2>) {
    for point in collision_data {
        gizmos.circle_2d(*point, 5., Color::PURPLE);
    }
}

fn cal_angle_from_points(a: Vec2, b: Vec2) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    dy.atan2(dx)
}
