use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod client;
mod dialogue;
mod game;
mod main_menu;
mod scene;

pub const GAME_WIDTH: f32 = 1980.;
pub const GAME_HEIGHT: f32 = 1080.;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(scene::plugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins((dialogue::plugin, client::plugin))
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: GAME_WIDTH,
                min_height: GAME_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
