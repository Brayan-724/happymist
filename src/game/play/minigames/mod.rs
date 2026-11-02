use bevy::prelude::*;

use crate::game::play::PlayPhase;

pub mod cut;
pub mod hud;

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(PlayPhase = PlayPhase::Minigame)]
pub enum ActiveMinigame {
    #[default]
    Cut,
}

pub fn plugin(app: &mut App) {
    app.add_sub_state::<ActiveMinigame>()
        .add_plugins((hud::plugin, cut::plugin));
}
