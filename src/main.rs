mod camera;
mod obstacle;
mod ray;

use bevy::prelude::*;
use camera::CameraPlugin;
use obstacle::ObstaclePlugin;
use ray::RayPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_plugins(RayPlugin)
        .add_plugins(ObstaclePlugin)
        .run();
}
