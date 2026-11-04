use bevy::prelude::*;

use crate::game::play::PlayPhase;

pub mod cut;
pub mod hud;
pub mod level;

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(PlayPhase = PlayPhase::Minigame)]
pub enum ActiveMinigame {
    #[default]
    None,
    Cut,
    Pot,
    InFire,
}

pub struct MinigameInfo {
    pub title: &'static str,
    pub goal: u32,
    pub time_limit: f32,
}

impl ActiveMinigame {
    pub fn info(self) -> MinigameInfo {
        match self {
            Self::None => MinigameInfo {
                title: "No minigame",
                goal: 0,
                time_limit: 0.,
            },
            Self::Cut => MinigameInfo {
                title: "Cut the sugarcane!!!",
                goal: 3,
                time_limit: 5.,
            },
            Self::Pot => MinigameInfo {
                title: "Into the pot!!!",
                goal: 1,
                time_limit: 5.,
            },
            ActiveMinigame::InFire => MinigameInfo {
                title: "Put it in the fire!!!",
                goal: 1,
                time_limit: 5.,
            },
        }
    }
}

// Here we should define the "game scene" having each level a list of something like:
// Level = [cut, mix, shake]

pub fn plugin(app: &mut App) {
    app.add_sub_state::<ActiveMinigame>()
        .add_plugins((hud::plugin, level::plugin, cut::plugin));
}
