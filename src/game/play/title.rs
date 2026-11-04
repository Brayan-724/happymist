use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    client::DialogueFinished,
    game::{GameState, level::LevelProgress, play::PlayPhase},
    ui::anim::{SlideInFinished, SlideOut, slide_in_random},
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

const TITLE_HOLD: f32 = 2.;
const COUNTDOWN_FROM: u32 = 3;
const COUNTDOWN_STEP: f32 = 0.6;
const COVER_TIME: f32 = 0.6;

#[derive(Component, Default)]
struct TitleSequence {
    elapsed: f32,
    shown: u32,
}

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, load_background)
        .add_systems(OnEnter(PlayPhase::Title), setup)
        .add_systems(
            Update,
            (update, oscillate, title_sequence).run_if(in_state(PlayPhase::Title)),
        )
        .add_systems(Update, scroll_background)
        .add_observer(on_dialogue_finished);
}

#[derive(Component)]
struct TitleRoot;

#[derive(Resource)]
struct TitleBackground(Handle<Image>);

fn load_background(mut commands: Commands, assets: Res<AssetServer>) {
    let image = assets
        .load_builder()
        .with_settings(|s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest())
        .load("sprites/background/title-bg.png");

    commands.insert_resource(TitleBackground(image));
}

fn scrolling_background(image: Handle<Image>) -> impl Bundle {
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
    )
}

fn on_dialogue_finished(
    _: On<DialogueFinished>,
    mut commands: Commands,
    background: Res<TitleBackground>,
) {
    commands
        .spawn((
            Name::new("TitleCover"),
            GlobalZIndex(1),
            DespawnOnExit(GameState::Dialogue),
            slide_in_random(COVER_TIME),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            children![scrolling_background(background.0.clone())],
        ))
        .observe(on_cover_finished);
}

fn on_cover_finished(_: On<SlideInFinished>, mut next: ResMut<NextState<GameState>>) {
    next.set(GameState::Game);
}

fn setup(mut commands: Commands, background: Res<TitleBackground>, progress: Res<LevelProgress>) {
    let title = progress
        .current_minigame()
        .map_or("", |minigame| minigame.info().title);

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
            scrolling_background(background.0.clone()),
            (
                Text::new(title),
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
