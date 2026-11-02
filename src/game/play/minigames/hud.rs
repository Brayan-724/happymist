use bevy::prelude::*;

use crate::{
    game::{
        GameState,
        play::PlayPhase,
    },
    ui::{
        anim::{FallAway, Shake},
        time_bar::{TimeBar, TimeBarFinished, time_bar},
    },
};

const MAX_LIVES: u32 = 3;
const TIME_LIMIT: f32 = 5.;

#[derive(Component)]
struct MinigameRoot;

#[derive(Component)]
pub struct Lives(pub u32);

#[derive(Component)]
struct LifeIcon(u32);

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayPhase::Minigame), setup)
        .add_systems(
            Update,
            (update, drop_lost_lives).run_if(in_state(PlayPhase::Minigame)),
        );
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let life_image: Handle<Image> = assets.load("sprites/ui/live.png");

    commands
        .spawn((
            Name::new("MinigameRoot"),
            MinigameRoot,
            DespawnOnExit(GameState::Game),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(time_bar(TIME_LIMIT)).observe(on_time_up);

            root.spawn((
                Lives(MAX_LIVES),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(20),
                    right: px(20),
                    column_gap: px(10),
                    ..default()
                },
            ))
            .with_children(|row| {
                for i in 0..MAX_LIVES {
                    row.spawn((
                        LifeIcon(i),
                        ImageNode::new(life_image.clone()),
                        Node {
                            height: px(80),
                            ..default()
                        },
                    ));
                }
            });
        });
}

fn update() {}

fn on_time_up(
    event: On<TimeBarFinished>,
    mut commands: Commands,
    mut bars: Query<&mut TimeBar>,
    mut lives: Query<&mut Lives>,
    root: Query<Entity, With<MinigameRoot>>,
) {
    let Ok(mut lives) = lives.single_mut() else {
        return;
    };
    lives.0 = lives.0.saturating_sub(1);

    if let Ok(root) = root.single() {
        commands.entity(root).insert(Shake::new(0.3, 15.));
    }

    if lives.0 == 0 {
        info!("game over");
        return;
    }

    // restart the countdown
    if let Ok(mut bar) = bars.get_mut(event.0) {
        bar.0.reset();
    }
}

fn drop_lost_lives(
    mut commands: Commands,
    lives: Query<&Lives, Changed<Lives>>,
    icons: Query<(Entity, &LifeIcon)>,
) {
    let Ok(lives) = lives.single() else {
        return;
    };

    let lost = MAX_LIVES - lives.0;
    if lost == 0 {
        return;
    }

    for (entity, icon) in &icons {
        if icon.0 == lost - 1 {
            commands.entity(entity).insert(FallAway::default());
        }
    }
}
