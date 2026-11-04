use bevy::prelude::*;

use crate::{
    game::{
        GameState,
        level::{LevelProgress, MinigameFinished},
        play::PlayPhase,
    },
    ui::{
        anim::{FallAway, Shake},
        time_bar::{TimeBar, TimeBarFinished, time_bar},
    },
};

const MAX_LIVES: u32 = 3;
const LOSE_DELAY: f32 = 1.;

#[derive(Component)]
struct MinigameRoot;

#[derive(Component)]
pub struct FallsOnLose;

#[derive(Resource)]
pub struct LoseDelay(Timer);

#[derive(Resource)]
pub struct Lives(pub u32);

impl Default for Lives {
    fn default() -> Self {
        Self(MAX_LIVES)
    }
}
#[derive(Component)]
struct LifeIcon(u32);

pub fn plugin(app: &mut App) {
    app.init_resource::<Lives>()
        .add_systems(OnEnter(GameState::Game), reset_lives)
        .add_systems(OnEnter(PlayPhase::Minigame), setup)
        .add_systems(
            Update,
            (update, drop_lost_lives).run_if(in_state(PlayPhase::Minigame)),
        )
        .add_systems(Update, finish_losing.run_if(resource_exists::<LoseDelay>))
        .add_observer(stop_time_bar);
}

fn reset_lives(mut lives: ResMut<Lives>) {
    *lives = default();
}

fn stop_time_bar(_: On<MinigameFinished>, mut bars: Query<&mut TimeBar>) {
    for mut bar in &mut bars {
        bar.0.pause();
    }
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    lives: Res<Lives>,
    progress: Res<LevelProgress>,
) {
    let Some(minigame) = progress.current_minigame() else {
        return;
    };

    let life_image: Handle<Image> = assets.load("sprites/ui/live.png");

    commands
        .spawn((
            Name::new("MinigameRoot"),
            MinigameRoot,
            DespawnOnExit(PlayPhase::Minigame),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((time_bar(minigame.info().time_limit), FallsOnLose))
                .observe(on_time_up);

            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: px(20),
                right: px(20),
                column_gap: px(10),
                ..default()
            })
            .with_children(|row| {
                for i in (MAX_LIVES - lives.0)..MAX_LIVES {
                    row.spawn((
                        LifeIcon(i),
                        FallsOnLose,
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
    mut lives: ResMut<Lives>,
    root: Query<Entity, With<MinigameRoot>>,
    fallers: Query<Entity, With<FallsOnLose>>,
) {
    lives.0 = lives.0.saturating_sub(1);

    if let Ok(root) = root.single() {
        commands.entity(root).insert(Shake::new(0.3, 15.));
    }

    if lives.0 == 0 {
        for entity in &fallers {
            commands.entity(entity).insert(FallAway::default());
        }
        commands.insert_resource(LoseDelay(Timer::from_seconds(LOSE_DELAY, TimerMode::Once)));

        // nothing can be clicked while everything falls
        commands.spawn((
            Name::new("LoseBlocker"),
            DespawnOnExit(PlayPhase::Minigame),
            GlobalZIndex(10),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ));
        return;
    }

    // restart the countdown
    if let Ok(mut bar) = bars.get_mut(event.0) {
        bar.0.reset();
    }
}

fn finish_losing(
    mut commands: Commands,
    time: Res<Time>,
    mut delay: ResMut<LoseDelay>,
    mut next: ResMut<NextState<PlayPhase>>,
) {
    if delay.0.tick(time.delta()).just_finished() {
        commands.remove_resource::<LoseDelay>();
        next.set(PlayPhase::GameOver);
    }
}

fn drop_lost_lives(mut commands: Commands, lives: Res<Lives>, icons: Query<(Entity, &LifeIcon)>) {
    if !lives.is_changed() {
        return;
    }

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
