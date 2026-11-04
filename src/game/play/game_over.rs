use bevy::prelude::*;

use crate::game::play::PlayPhase;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayPhase::GameOver), setup);
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("GameOver"),
        DespawnOnExit(PlayPhase::GameOver),
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![Text::new("Game over")],
    ));
}
