use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use self::client::Client;

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

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
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

    commands.spawn(Client::Ñoquer);

    commands.spawn((
        Sprite {
            image: assets.load("ñoquer.jpg"),
            custom_size: Some(vec2(1800., 250.)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0., -(GAME_HEIGHT / 2. - 250. / 2.), 1.)),
    ));
}
