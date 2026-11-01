use bevy::{
    app::App,
    ecs::system::Commands,
    state::{
        app::AppExtStates,
        state::{OnEnter, States, SubStates},
        state_scoped::DespawnOnExit,
    },
    ui::{Node, widget::Text},
};

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

fn enter_dialogue(mut commands: Commands) {
    commands.spawn((
        Text::new("Dialogue"),
        DespawnOnExit(GameState::Dialogue), // gone when you switch to GameState::Game
    ));
}

fn enter_game(mut commands: Commands) {
    commands.spawn((Text::new("Game"), DespawnOnExit(GameState::Game)));
}

fn setup(mut commands: Commands) {
    commands.spawn((Text::new("Playing State"),));
}

fn update() {}
