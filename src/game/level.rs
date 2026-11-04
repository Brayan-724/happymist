use bevy::prelude::*;

use crate::{
    client::{Client, task::TaskId},
    game::{
        GameState,
        play::{PlayPhase, minigames::ActiveMinigame},
    },
    scene::Scene,
};

const FINISH_DELAY: f32 = 0.7;

#[derive(Event)]
pub struct MinigameFinished;

#[derive(Resource)]
struct FinishDelay(Timer);

pub struct LevelDef {
    pub client: Client,
    pub task: TaskId,
    pub minigames: &'static [ActiveMinigame],
}

const LEVELS: &[LevelDef] = &[LevelDef {
    client: Client::Ñoquer,
    task: TaskId(0),
    minigames: &[ActiveMinigame::Cut, ActiveMinigame::Pot],
}];

pub fn plugin(app: &mut App) {
    app.init_resource::<LevelProgress>()
        .add_systems(OnEnter(PlayPhase::Minigame), start_current)
        .add_systems(
            Update,
            finish_minigame.run_if(resource_exists::<FinishDelay>),
        )
        .add_observer(on_minigame_finished);
}

#[derive(Resource, Default)]
pub struct LevelProgress {
    level: usize,
    minigame: usize,
}

impl LevelProgress {
    pub fn current_level(&self) -> Option<&'static LevelDef> {
        LEVELS.get(self.level)
    }

    pub fn current_minigame(&self) -> Option<ActiveMinigame> {
        self.current_level()?.minigames.get(self.minigame).copied()
    }

    pub fn restart_level(&mut self) {
        self.minigame = 0;
    }
}

fn start_current(progress: Res<LevelProgress>, mut next: ResMut<NextState<ActiveMinigame>>) {
    if let Some(minigame) = progress.current_minigame() {
        next.set(minigame);
    }
}

fn on_minigame_finished(_: On<MinigameFinished>, mut commands: Commands) {
    commands.insert_resource(FinishDelay(Timer::from_seconds(
        FINISH_DELAY,
        TimerMode::Once,
    )));
}

fn finish_minigame(
    mut commands: Commands,
    time: Res<Time>,
    mut delay: ResMut<FinishDelay>,
    mut progress: ResMut<LevelProgress>,
    mut next_phase: ResMut<NextState<PlayPhase>>,
    mut next_game: ResMut<NextState<GameState>>,
    mut next_scene: ResMut<NextState<Scene>>,
) {
    if !delay.0.tick(time.delta()).just_finished() {
        return;
    }
    commands.remove_resource::<FinishDelay>();

    progress.minigame += 1;
    if progress.current_minigame().is_some() {
        next_phase.set(PlayPhase::Title);
        return;
    }

    progress.level += 1;
    progress.minigame = 0;
    if progress.current_level().is_some() {
        next_game.set(GameState::Dialogue);
    } else {
        progress.level = 0;
        next_scene.set(Scene::MainMenu);
    }
}
