use bevy::prelude::*;

use crate::GAME_HEIGHT;
use crate::client::Client;
use crate::scene::Scene;

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(Scene = Scene::Game)]
pub enum GameState {
    Dialogue,
    #[default]
    Game,
}

pub fn plugin(app: &mut App) {
    app.add_sub_state::<GameState>()
        .add_systems(OnEnter(Scene::Game), setup)
        .add_systems(OnEnter(GameState::Dialogue), enter_dialogue)
        .add_systems(OnEnter(GameState::Game), enter_game);
}

fn enter_dialogue(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        Text::new("Dialogue"),
        DespawnOnExit(GameState::Dialogue), // gone when you switch to GameState::Game
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

fn enter_game(mut commands: Commands) {
    commands.spawn((Text::new("Game"), DespawnOnExit(GameState::Game)));
}

fn setup(mut commands: Commands) {
    commands.spawn((Text::new("Playing State"),));
}

fn update() {}
