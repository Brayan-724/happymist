use bevy::prelude::*;

use crate::game::GameState;

pub mod minigame;
pub mod title;

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Game)]
pub enum PlayPhase {
    #[default]
    Title,
    Minigame,
}

pub fn plugin(app: &mut App) {
    app.add_sub_state::<PlayPhase>()
        .add_plugins(title::plugin)
        .add_plugins(minigame::plugin);
}
