use bevy::prelude::*;

use crate::scene::Scene;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Scene::MainMenu), setup);
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(Scene::MainMenu),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(20),
                ..default()
            },
            Text::new("MAIN MENU"),
        ))
        .with_children(|parent| {
            parent.spawn(Text::new("MAIN MENU"));

            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(px(40), px(15)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    children![Text::new("Play")],
                ))
                .observe(on_play_click); // <- the onclick
        });
}

fn on_play_click(_click: On<PointerClick>, mut next: ResMut<NextState<Scene>>) {
    next.set(Scene::Game);
}
