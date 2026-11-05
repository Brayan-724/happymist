use bevy::math::FloatExt as _;
use bevy::math::ops::atan2;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::{CursorIcon, SystemCursorIcon};

use crate::client::DialogueFinished;
use crate::client::loader::ClientTaskRequestAsset;
use crate::ext::FloatExt as _;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(
            Update,
            (update_postit_entering, update_postit_hovered, update_postit),
        )
        .add_observer(on_assigned_task);
}

#[derive(Component, Clone, Copy)]
pub struct Player;

#[derive(Component, Clone)]
pub struct PlayerCurrentTask(pub ClientTaskRequestAsset);

fn setup(mut commands: Commands) {
    commands.spawn(Player);
}

fn on_assigned_task(
    _: On<Add<PlayerCurrentTask>>,
    mut commands: Commands,
    current_task: Single<&PlayerCurrentTask>,
    assets: Res<AssetServer>,
) {
    match &current_task.0 {
        ClientTaskRequestAsset::Item { item, .. } => {
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(150),
                        right: px(150),
                        width: px(250),
                        height: px(250 * 2),
                        ..default()
                    },
                    PostitEntering(0.),
                    PostitPhysics {
                        enabled: true,
                        angle: -35.,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_child((
                    Postit,
                    RelativeCursorPosition::default(),
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(0),
                        width: px(250),
                        height: px(250),
                        padding: UiRect {
                            left: px(20),
                            right: px(50),
                            top: px(50),
                            bottom: px(20),
                        },
                        ..default()
                    },
                    ImageNode::new(assets.load("sprites/ui/postit.png"))
                        .with_visual_box(VisualBox::BorderBox),
                    children![(Pickable::IGNORE, Text(item.clone()),)],
                ))
                .observe(on_click_task)
                .observe(on_over_task)
                .observe(on_move_task)
                .observe(on_out_task);
        }
    }
}

fn on_click_task(
    mut event: On<PointerClick>,
    mut commands: Commands,
    postit: Single<Entity, With<PostitPhysics>>,
) {
    event.propagate(false);
    commands.trigger(DialogueFinished);
    commands.entity(*postit).despawn();
}

fn on_over_task(
    mut event: On<PointerOver>,
    mut commands: Commands,
    mut cursor: Single<(Entity, Option<&mut CursorIcon>), With<Window>>,
    mut query: Single<(&mut PostitHovered, &mut PostitPhysics)>,
    postit: Single<&RelativeCursorPosition, With<Postit>>,
) {
    event.propagate(false);

    // Set physics
    query.1.enabled = false;
    query.1.elapsed = 0.;

    let Some(diff) = postit.normalized else {
        return;
    };

    query.1.angle = -atan2(diff.x, (diff.y + 0.5).min(0.5)).to_degrees();

    // Set hovered
    query.0.0 = true;

    if let Some(cursor) = &mut cursor.1 {
        **cursor = CursorIcon::System(SystemCursorIcon::Pointer);
    } else {
        commands
            .entity(cursor.0)
            .insert(CursorIcon::System(SystemCursorIcon::Pointer));
    }
}

fn on_move_task(
    mut event: On<PointerMove>,
    mut physics: Single<&mut PostitPhysics, With<PostitHovered>>,
    postit: Single<&RelativeCursorPosition, With<Postit>>,
) {
    event.propagate(false);

    let Some(diff) = postit.normalized else {
        return;
    };

    physics.angle = -atan2(diff.x, (diff.y + 0.5).min(0.5)).to_degrees();
}

fn on_out_task(
    mut event: On<PointerOut>,
    mut cursor: Single<&mut CursorIcon, With<Window>>,
    mut query: Single<(&mut PostitHovered, &mut PostitPhysics)>,
) {
    event.propagate(false);

    query.1.enabled = true;
    query.1.elapsed = 0.;
    query.1.angle = query.1.real_angle;

    query.0.0 = false;

    **cursor = CursorIcon::System(SystemCursorIcon::Default);
}

#[derive(Component)]
struct Postit;

#[derive(Component)]
struct PostitHovered(bool, f32);

const POSTIT_HOVERING_TIME: f32 = 0.4;
const POSTIT_HOVERING_SCALE: f32 = 0.1;

fn update_postit_hovered(time: Res<Time>, query: Query<(&mut PostitHovered, &mut UiTransform)>) {
    for (mut hovered, mut trans) in query {
        if hovered.0 {
            hovered.1 += time.delta_secs();
        } else {
            hovered.1 -= time.delta_secs();
        }

        hovered.1 = hovered.1.clamp(0., POSTIT_HOVERING_TIME);

        if !(hovered.1.close_to(0.) && hovered.1.close_to(POSTIT_HOVERING_TIME)) {
            trans.scale = Vec2::splat(
                1. + (if hovered.0 {
                    EaseFunction::ElasticOut
                } else {
                    EaseFunction::ElasticIn
                })
                .sample_unchecked(hovered.1 / POSTIT_HOVERING_TIME)
                    * POSTIT_HOVERING_SCALE,
            );
        }
    }
}

#[derive(Component)]
struct PostitEntering(f32);

const POSTIT_ENTERING_TIME: f32 = 1.5;

fn update_postit_entering(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &mut PostitEntering, &mut UiTransform)>,
) {
    for (entity, mut entering, mut trans) in query {
        entering.0 += time.delta_secs();

        if entering.0 > POSTIT_ENTERING_TIME {
            commands
                .entity(entity)
                .remove::<PostitEntering>()
                .insert(PostitHovered(false, 0.))
                .insert(Pickable::default());
        } else {
            trans.scale = Vec2::splat(
                EaseFunction::ElasticOut.sample_unchecked(entering.0 / POSTIT_ENTERING_TIME),
            );
        }
    }
}

#[derive(Component, Default)]
struct PostitPhysics {
    enabled: bool,
    elapsed: f32,
    angle: f32,
    real_angle: f32,
}

const POSTIT_PHYSICS_DRAG: f32 = 12.;
const POSTIT_PHYSICS_VEL: f32 = 3.;

fn update_postit(time: Res<Time>, query: Query<(&mut PostitPhysics, &mut UiTransform)>) {
    for (mut physics, mut ui) in query {
        if physics.enabled {
            if physics.angle.close_to(0.) {
                continue;
            }

            if physics.angle > 0. {
                physics.angle -= POSTIT_PHYSICS_DRAG * time.delta_secs();
                if physics.angle <= 0. {
                    physics.angle = 0.;
                }
            } else if physics.angle < 0. {
                physics.angle += POSTIT_PHYSICS_DRAG * time.delta_secs();
                if physics.angle >= 0. {
                    physics.angle = 0.;
                }
            }

            physics.elapsed += time.delta_secs();
            physics.real_angle = physics.angle * (physics.elapsed * POSTIT_PHYSICS_VEL).cos();

            ui.rotation = Rot2::degrees(physics.real_angle);
        } else {
            physics.real_angle = physics
                .real_angle
                .lerp(physics.angle, time.delta_secs() * 3.);
            ui.rotation = Rot2::degrees(physics.real_angle);
        }
    }
}
