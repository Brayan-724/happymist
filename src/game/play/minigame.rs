use bevy::prelude::*;

use crate::{
    game::{GameState, play::PlayPhase},
    ui::time_bar::{TimeBarFinished, time_bar},
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayPhase::Minigame), setup)
        .add_systems(Update, update.run_if(in_state(PlayPhase::Minigame)));
}

fn setup(mut commands: Commands) {
    commands
        .spawn((time_bar(5.), DespawnOnExit(GameState::Game)))
        .observe(on_time_up);
}

fn update() {}

fn on_time_up(_event: On<TimeBarFinished>) {
    info!("time's up!");
}
