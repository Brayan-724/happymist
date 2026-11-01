use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    game::{GameState, play::PlayPhase},
    ui::anim::SlideOut,
};

const TILE_STRETCH: f32 = 20.;
const TILE_SIZE: f32 = 16. * TILE_STRETCH;
const SCROLL_SPEED: f32 = 30.;

#[derive(Component)]
struct ScrollingBackground;

#[derive(Component)]
struct Oscillate {
    angle: f32,
    bob: f32,
    speed: f32,
}

#[derive(Component)]
struct ScaleIn(Timer);

const TITLE_HOLD: f32 = 2.; // seconds the title stays before the countdown
const COUNTDOWN_FROM: u32 = 3;
const COUNTDOWN_STEP: f32 = 0.6;

#[derive(Component, Default)]
struct TitleSequence {
    elapsed: f32,
    shown: u32, // number currently on screen (0 = still the title)
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayPhase::Title), setup)
        .add_systems(
            Update,
            (update, oscillate, title_sequence).run_if(in_state(PlayPhase::Title)),
        )
        .add_systems(Update, scroll_background);
}

#[derive(Component)]
struct TitleRoot;

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let image = assets
        .load_builder()
        .with_settings(|s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest())
        .load("sprites/background/title-bg.png");

    commands.spawn((
        TitleRoot,
        GlobalZIndex(1),
        DespawnOnExit(GameState::Game),
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                ScrollingBackground,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    right: px(-TILE_SIZE),
                    bottom: px(-TILE_SIZE),
                    ..default()
                },
                ImageNode {
                    image,
                    image_mode: NodeImageMode::Tiled {
                        tile_x: true,
                        tile_y: true,
                        stretch_value: TILE_STRETCH,
                    },
                    ..default()
                },
                ZIndex(-1),
            ),
            (
                Text::new("Cut the sugarcane!!!"),
                TitleSequence::default(),
                TextFont {
                    font_size: FontSize::Px(80.),
                    ..default()
                },
                UiTransform {
                    scale: Vec2::ZERO,
                    ..default()
                },
                ScaleIn(Timer::from_seconds(1.5, TimerMode::Once)),
                Oscillate {
                    angle: 4.,
                    bob: 8.,
                    speed: 0.5
                },
            )
        ],
    ));
}

fn oscillate(time: Res<Time>, mut query: Query<(&Oscillate, &mut UiTransform)>) {
    for (osc, mut transform) in &mut query {
        let wave = (time.elapsed_secs() * osc.speed * std::f32::consts::TAU).sin();

        transform.rotation = Rot2::degrees(wave * osc.angle);
        transform.translation = Val2::new(px(0), px(wave * osc.bob));
    }
}

fn scroll_background(
    time: Res<Time>,
    mut query: Query<&mut UiTransform, With<ScrollingBackground>>,
) {
    let offset = (time.elapsed_secs() * SCROLL_SPEED) % TILE_SIZE;

    for mut transform in &mut query {
        transform.translation = Val2::new(px(-offset), px(-offset));
    }
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScaleIn, &mut UiTransform)>,
) {
    for (entity, mut scale_in, mut transform) in &mut query {
        scale_in.0.tick(time.delta());

        let t = EaseFunction::BackOut.sample_clamped(scale_in.0.fraction());
        transform.scale = Vec2::splat(t);

        if scale_in.0.just_finished() {
            commands.entity(entity).remove::<ScaleIn>();
        }
    }
}

fn title_sequence(
    mut commands: Commands,
    time: Res<Time>,
    mut next: ResMut<NextState<PlayPhase>>,
    mut query: Query<(Entity, &mut TitleSequence, &mut Text)>,
    roots: Query<Entity, With<TitleRoot>>,
) {
    for (entity, mut sequence, mut text) in &mut query {
        sequence.elapsed += time.delta_secs();

        let countdown_elapsed = sequence.elapsed - TITLE_HOLD;
        if countdown_elapsed < 0. {
            continue;
        }

        let step = (countdown_elapsed / COUNTDOWN_STEP) as u32;
        if step >= COUNTDOWN_FROM {
            for root in &roots {
                commands.entity(root).insert(SlideOut::random(0.6));
            }
            next.set(PlayPhase::Minigame);
            continue;
        }

        let number = COUNTDOWN_FROM - step;
        if sequence.shown != number {
            sequence.shown = number;
            text.0 = number.to_string();
            commands.entity(entity).insert(ScaleIn(Timer::from_seconds(
                COUNTDOWN_STEP * 0.6,
                TimerMode::Once,
            )));
        }
    }
}
