/*
    This code its the level management, it should allow us to define a level being something like:

    level = [cut, mix, shake]

    and based on it it should generate the minigame structure
*/

use bevy::prelude::*;

use crate::game::play::{PlayPhase, minigames::ActiveMinigame};

#[derive(Event)]
pub struct MinigameFinished;

#[derive(Resource)]
pub struct Level {
    minigames: Vec<ActiveMinigame>,
    current: usize,
}

pub struct LevelDef {
    pub minigames: &'static [ActiveMinigame],
}

const LEVELS: &[LevelDef] = &[LevelDef {
    minigames: &[
        ActiveMinigame::Pot,
        ActiveMinigame::Cut,
        ActiveMinigame::InFire,
    ],
}];

pub fn plugin(app: &mut App) {
    app.init_resource::<LevelProgress>()
        .add_systems(OnEnter(PlayPhase::Minigame), start_current)
        .add_observer(on_minigame_finished);
}

#[derive(Resource, Default)]
pub struct LevelProgress {
    level: usize,
    minigame: usize,
}

impl LevelProgress {
    pub fn current_minigame(&self) -> Option<ActiveMinigame> {
        LEVELS
            .get(self.level)?
            .minigames
            .get(self.minigame)
            .copied()
    }
}

fn start_current(progress: Res<LevelProgress>, mut next: ResMut<NextState<ActiveMinigame>>) {
    if let Some(minigame) = progress.current_minigame() {
        next.set(minigame);
    }
}

fn on_minigame_finished(
    _: On<MinigameFinished>,
    mut progress: ResMut<LevelProgress>,
    mut next: ResMut<NextState<PlayPhase>>,
) {
    progress.minigame += 1;
    if progress.current_minigame().is_some() {
        next.set(PlayPhase::Title);
    } else {
        println!("level finished");
    }
}
