use bevy::prelude::*;

use crate::GAME_HEIGHT;
use crate::game::level::LevelProgress;
use crate::scene::Scene;

pub mod level;
mod play;

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(Scene = Scene::Game)]
pub enum GameState {
    #[default]
    Dialogue,
    Game,
}

pub fn plugin(app: &mut App) {
    app.add_sub_state::<GameState>()
        .add_plugins((level::plugin, play::plugin))
        .add_systems(OnEnter(Scene::Game), setup)
        .add_systems(OnExit(Scene::Game), cleanup)
        .add_systems(OnEnter(GameState::Dialogue), enter_dialogue);
}

fn enter_dialogue(
    mut commands: Commands,
    assets: Res<AssetServer>,
    progress: Res<LevelProgress>,
) {
    let Some(level) = progress.current_level() else {
        return;
    };

    commands.spawn((Text::new("Dialogue"), DespawnOnExit(GameState::Dialogue)));

    commands.spawn((level.client, DespawnOnExit(GameState::Dialogue)));

    commands.spawn((
        DespawnOnExit(GameState::Dialogue),
        Sprite {
            image: assets.load("ñoquer.jpg"),
            custom_size: Some(vec2(1800., 250.)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0., -(GAME_HEIGHT / 2. - 250. / 2.), 1.)),
    ));
}

fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::srgb_u8(0x2A, 0x1E, 0x3A)));
}

fn cleanup(mut commands: Commands) {
    commands.insert_resource(ClearColor::default());
}

fn update() {}
