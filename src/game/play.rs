use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    game::GameState,
    ui::time_bar::{TimeBarFinished, time_bar},
};

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Game)]
enum PlayPhase {
    Title,
    #[default]
    Minigame,
}

pub fn plugin(app: &mut App) {
    app.add_sub_state::<PlayPhase>()
        .add_plugins(title_plugin)
        .add_plugins(minigame_plugin);
}

fn title_plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayPhase::Title), title_setup)
        .add_systems(Update, title_update.run_if(in_state(PlayPhase::Title)));
}

fn minigame_plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayPhase::Minigame), minigame_setup)
        .add_systems(
            Update,
            minigame_update.run_if(in_state(PlayPhase::Minigame)),
        );
}

fn title_setup(mut commands: Commands, assets: Res<AssetServer>) {
    let image = assets
        .load_builder()
        .with_settings(|s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest())
        .load("sprites/background/title-bg.png");

    commands.spawn((
        DespawnOnExit(PlayPhase::Title),
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ImageNode {
            image,
            image_mode: NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 16.0,
            },
            ..default()
        },
        children![(
            Text::new("Cut the sugarcane!!!"),
            TextFont {
                font_size: FontSize::Px(80.),
                ..default()
            },
            UiTransform {
                scale: Vec2::ZERO,
                ..default()
            },
            ScaleIn(Timer::from_seconds(1.5, TimerMode::Once))
        )],
        DespawnOnExit(GameState::Game),
    ));
}

#[derive(Component)]
struct ScaleIn(Timer);

fn title_update(
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

fn minigame_setup(mut commands: Commands) {
    commands
        .spawn((time_bar(5.), DespawnOnExit(GameState::Game)))
        .observe(on_time_up);
}

fn on_time_up(_event: On<TimeBarFinished>) {
    info!("time's up!");
}

fn minigame_update() {}
