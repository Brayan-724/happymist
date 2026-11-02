use bevy::prelude::*;

use super::ActiveMinigame;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(ActiveMinigame::Cut), setup)
        .add_systems(Update, update.run_if(in_state(ActiveMinigame::Cut)));
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn((
            Name::new("CutMinigame"),
            DespawnOnExit(ActiveMinigame::Cut),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("Sugarcane"),
                ImageNode::new(assets.load("sprites/minigames/sugarcane.png")),
                Node {
                    width: percent(40),
                    ..default()
                },
            ));
        });
}

pub fn update() {}
