use bevy::prelude::*;

use crate::game::{GameState, level::LevelProgress, play::PlayPhase};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayPhase::GameOver), setup);
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Name::new("GameOver"),
            DespawnOnExit(PlayPhase::GameOver),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(20),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("Game over"));

            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(px(40), px(15)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    children![Text::new("Retry")],
                ))
                .observe(on_retry_click);
        });
}

fn on_retry_click(
    _click: On<PointerClick>,
    mut progress: ResMut<LevelProgress>,
    mut next: ResMut<NextState<GameState>>,
) {
    progress.restart_level();
    next.set(GameState::Dialogue);
}
