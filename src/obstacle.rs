use bevy::prelude::*;

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_obstacle, set_gizmo_config))
            .add_systems(Update, render_obstacles);
    }
}

#[derive(Component, Debug)]
pub struct Obstacle {
    pub start_offset: Vec2,
    pub end_offset: Vec2,
}

fn setup_obstacle(mut commands: Commands) {
    let obstacle_data: [((f32, f32), (f32, f32)); 16] = [
        ((-800., 500.), (800., 500.)),
        ((-800., -500.), (800., -500.)),
        ((-800., -500.), (-800., 500.)),
        ((800., 500.), (800., -500.)),
        ((100., 400.), (400., 400.)),
        ((400., 100.), (400., 400.)),
        ((-100., -400.), (-400., -400.)),
        ((-400., -100.), (-400., -400.)),
        ((-100., 400.), (-400., 400.)),
        ((-400., 100.), (-400., 400.)),
        ((100., -400.), (400., -400.)),
        ((400., -100.), (400., -400.)),
        ((100., 100.), (100., -100.)),
        ((-100., -100.), (100., -100.)),
        ((-100., -100.), (-100., 100.)),
        ((-100., 100.), (100., 100.)),
    ];

    for (a, b) in obstacle_data.into_iter() {
        commands.spawn((
            Transform::default(),
            Obstacle {
                start_offset: Vec2::new(a.0, a.1),
                end_offset: Vec2::new(b.0, b.1),
            },
        ));
    }
}

fn set_gizmo_config(mut gizmo_config: ResMut<GizmoConfig>) {
    gizmo_config.line_width = 5.;
}

fn render_obstacles(mut gizmos: Gizmos, obstacle_query: Query<(&Obstacle, &Transform)>) {
    for (obstacle, transform) in obstacle_query.iter() {
        gizmos.line_2d(
            transform.translation.xy() + obstacle.start_offset,
            transform.translation.xy() + obstacle.end_offset,
            Color::GRAY,
        );
    }
}
