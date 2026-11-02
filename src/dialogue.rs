use core::{f32, fmt};

use bevy::color::palettes::tailwind;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};
use serde::Deserialize;

use crate::client::task::TaskId;
use crate::fonts::Fonts;

use self::parser::{DialogueContent, DialogueSegment};

pub mod parser;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_dialogue_animation,
            update_dialogue_char_animation,
            update_dancing_anim,
        )
            .run_if(is_dialogue_active),
    )
    .add_observer(setup_dialogue)
    .add_observer(handle_click.run_if(is_dialogue_active));
}

#[derive(Reflect, Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DialogueRef {
    Explain(String),
}

#[derive(Component, Reflect, Clone, Copy, Debug, Default)]
pub struct DialogueId(pub TaskId, pub usize);

impl fmt::Display for DialogueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:0>2}:{:0>2}", self.0.0, self.1))
    }
}

#[derive(EntityEvent, Debug)]
pub struct NextDialogue(pub Entity);

#[derive(Component)]
pub struct Dialogue {
    pub name: String,
    pub content: DialogueContent,
    pub refs: HashMap<String, DialogueRef>,
    pub font_size: f32,
    pub segment: Option<DialogueSegment>,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DialogueState {
    Presenting,
    Ended,
}

#[derive(Component)]
struct DialogueUi(Entity);

#[derive(Component)]
struct DialogueText(Entity);

#[derive(Component)]
struct DialogueContainer(Entity);

#[derive(Component, Default)]
pub struct DialogueAnimated {
    pub elapsed: f32,
    pub next_char: f32,
    pub chars_advanced: usize,
}

#[derive(Component)]
#[relationship(relationship_target = LifetimeOwner)]
pub struct LifetimeShared(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = LifetimeShared)]
#[component(on_despawn)]
pub struct LifetimeOwner(Vec<Entity>);

impl LifetimeOwner {
    fn on_despawn(mut world: DeferredWorld, hook: HookContext) {
        let Some(lifetime) = world.get::<LifetimeOwner>(hook.entity) else {
            return;
        };

        for entity in lifetime.0.clone() {
            world.commands().entity(entity).despawn();
        }
    }
}

fn setup_dialogue(event: On<Add<Dialogue>>, mut commands: Commands, query: Query<&Dialogue>) {
    let Ok(dialogue) = query.get(event.entity) else {
        return;
    };

    let text = commands.spawn(Text(String::new())).id();

    let container = commands.spawn(Node::default()).add_child(text).id();

    commands
        .spawn((
            DialogueUi(event.entity),
            LifetimeShared(event.entity),
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                bottom: px(20),
                width: percent(70),
                max_width: px(1700),
                min_width: px(200),
                border_radius: BorderRadius::all(px(20)),
                padding: UiRect::all(px(25)),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            UiTransform::from_xy(percent(-50), percent(0)),
            BackgroundColor(tailwind::GRAY_800.into()),
        ))
        .with_child((
            Text(dialogue.name.clone()),
            TextFont {
                font_size: FontSize::Px(24.),
                weight: FontWeight(800),
                ..default()
            },
        ))
        .add_child(container);

    commands.entity(event.entity).insert((
        DialogueState::Presenting,
        DialogueText(text),
        DialogueContainer(container),
    ));
}

const TIME_PER_CHAR: f32 = 0.06;
const MIN_TIME: f32 = 0.5;
const MAX_TIME: f32 = 5.0;

fn update_dialogue_animation(
    mut commands: Commands,
    time: Res<Time>,
    fonts: Res<Fonts>,
    query: Query<(
        Entity,
        &mut DialogueState,
        &mut Dialogue,
        &mut DialogueAnimated,
        &mut DialogueText,
        &DialogueContainer,
    )>,
    mut texts: Query<(&mut Text, &Children)>,
) {
    for (dialogue_entity, mut state, mut dialogue, mut animation, mut text, container) in query {
        let max_t = (dialogue.content.len as f32 * TIME_PER_CHAR).clamp(MIN_TIME, MAX_TIME);
        let time_per_char = max_t / dialogue.content.len as f32;

        if animation.elapsed >= animation.next_char {
            let target_font = dialogue.font_size;

            let chars_to_advance = ((animation.elapsed.min(max_t) - animation.next_char)
                / time_per_char)
                .ceil() as usize;

            let mut i = 0;
            while i < chars_to_advance.max(1) {
                dialogue.segment = dialogue
                    .segment
                    .take()
                    .or_else(|| dialogue.content.segments.pop_front());

                let Some(segment) = &mut dialogue.segment else {
                    *state = DialogueState::Ended;
                    commands
                        .entity(dialogue_entity)
                        .remove::<DialogueAnimated>();
                    break;
                };

                // Only animate the last 5 chars
                let is_last = i + 5 >= chars_to_advance;

                if let Some(char) = segment.chars.next() {
                    i += 1;
                    animation.chars_advanced += 1;
                    animation.next_char += time_per_char;

                    let base = (
                        TextFont {
                            font: FontSource::Handle(
                                fonts.get(segment.style.bold, segment.style.italic),
                            ),
                            font_size: FontSize::Px(1.),
                            ..default()
                        },
                        DialogueCharAnimation {
                            elapsed: if is_last { 0. } else { CHAR_TIME },
                            target_font,
                        },
                    );

                    let current_text: Entity;

                    if segment.style.dancing {
                        let is_empty = texts
                            .get_mut(text.0)
                            .ok()
                            .filter(|(t, c)| t.is_empty() && c.is_empty());

                        if let Some((mut text_ref, _)) = is_empty {
                            text_ref.0 = char.into();
                        } else {
                            text.0 = commands.spawn((Text::new(char), base)).id();
                            commands.entity(container.0).add_child(text.0);
                        }

                        current_text = text.0;

                        commands
                            .entity(text.0)
                            .insert(DialogueDancingAnim(animation.chars_advanced));

                        text.0 = commands.spawn(Text::default()).id();
                        commands.entity(container.0).add_child(text.0);
                    } else {
                        current_text = commands.spawn((TextSpan::new(char), base)).id();

                        commands.entity(text.0).add_child(current_text);
                    }

                    if let Some(label) = segment.style.reference.clone() {
                        let Some(content) = dialogue.refs.get(&label) else {
                            continue;
                        };

                        let mut e = commands.entity(current_text);

                        match content {
                            DialogueRef::Explain(content) => {
                                e.insert(DialogueReference(content.clone()))
                            }
                        };

                        e.insert(Underline)
                            .observe(on_over_reference)
                            .observe(on_move_reference)
                            .observe(on_out_reference)
                            .observe(on_remove_reference);
                    }
                } else {
                    _ = dialogue.segment.take();
                }
            }
        }

        animation.elapsed += time.delta_secs();
    }
}

const CHAR_TIME: f32 = 0.2;

#[derive(Component)]
struct DialogueCharAnimation {
    elapsed: f32,
    target_font: f32,
}

fn update_dialogue_char_animation(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut TextFont, &mut DialogueCharAnimation)>,
) {
    for (entity, mut text_font, mut animation) in query {
        animation.elapsed += time.delta_secs();

        let new_size = 0.0
            .lerp(
                animation.target_font,
                EaseFunction::CubicOut.sample_unchecked(animation.elapsed / CHAR_TIME),
            )
            .ceil();

        text_font.font_size = FontSize::Px(new_size);

        if animation.elapsed > CHAR_TIME {
            commands.entity(entity).remove::<DialogueCharAnimation>();
        }
    }
}

const DANCING_SPEED: f32 = 2.3;
const DANCING_DIFF: f32 = 0.6;

#[derive(Component)]
struct DialogueDancingAnim(usize);

fn update_dancing_anim(time: Res<Time>, query: Query<(&mut UiTransform, &DialogueDancingAnim)>) {
    for (mut trans, animation) in query {
        let offset_t = animation.0 as f32 * DANCING_DIFF;

        let t = ((time.elapsed_secs() + offset_t) % DANCING_SPEED) / DANCING_SPEED;

        let new_y = (t * f32::consts::PI * 2.).sin();

        trans.translation.y = px(new_y);
    }
}

#[derive(Component)]
struct DialogueReference(String);

#[derive(Component)]
struct DialogueReferenceUi(Entity);

fn on_over_reference(
    event: On<PointerOver>,
    mut commands: Commands,
    mut cursor: Single<(Entity, Option<&mut CursorIcon>), With<Window>>,
    query: Query<&DialogueReference>,
) {
    let Ok(reference) = query.get(event.entity) else {
        return;
    };

    if let Some(cursor) = &mut cursor.1 {
        **cursor = CursorIcon::System(SystemCursorIcon::Help);
    } else {
        commands
            .entity(cursor.0)
            .insert(CursorIcon::System(SystemCursorIcon::Help));
    }

    let hit = event.pointer.position;

    let ui = commands
        .spawn((
            LifetimeShared(event.entity),
            Node {
                position_type: PositionType::Absolute,
                top: px(hit.y - 10.),
                left: px(hit.x),
                padding: UiRect::all(px(5)),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            UiTransform::from_xy(percent(-50), percent(-100)),
            BackgroundColor(tailwind::GRAY_900.into()),
            Text(reference.0.clone()),
            TextFont {
                font_size: FontSize::Rem(0.8),
                ..default()
            },
        ))
        .id();

    commands
        .entity(event.entity)
        .insert(DialogueReferenceUi(ui));
}

fn on_move_reference(
    event: On<PointerMove>,
    query: Query<&DialogueReferenceUi>,
    mut nodes: Query<&mut Node>,
) {
    let Ok(reference) = query.get(event.entity) else {
        return;
    };

    let Ok(mut node) = nodes.get_mut(reference.0) else {
        return;
    };

    let hit = event.pointer.position;

    node.top = px(hit.y - 10.);
    node.left = px(hit.x);
}

fn on_out_reference(
    event: On<PointerOut>,
    mut cursor: Single<&mut CursorIcon, With<Window>>,
    mut commands: Commands,
    query: Query<&DialogueReferenceUi>,
) {
    let Ok(ui) = query.get(event.entity) else {
        return;
    };

    **cursor = CursorIcon::System(SystemCursorIcon::Default);

    commands.entity(ui.0).despawn();
}

fn on_remove_reference(
    _: On<Despawn<DialogueReferenceUi>>,
    mut cursor: Single<&mut CursorIcon, With<Window>>,
) {
    **cursor = CursorIcon::System(SystemCursorIcon::Default);
}

fn is_dialogue_active(query: Query<(), With<Dialogue>>) -> bool {
    !query.is_empty()
}

fn handle_click(
    mut event: On<PointerClick>,
    mut commands: Commands,
    mut ui: Query<&DialogueUi>,
    mut query: Query<(Entity, &mut DialogueState, Option<&mut DialogueAnimated>)>,
) {
    let Ok(ui) = ui.get_mut(event.entity) else {
        return;
    };

    event.propagate(false);

    let Ok((entity, mut state, animation)) = query.get_mut(ui.0) else {
        return;
    };

    match *state {
        DialogueState::Presenting => {
            *state = DialogueState::Ended;
            if let Some(mut animation) = animation {
                animation.elapsed = f32::MAX;
            }
        }
        DialogueState::Ended => {
            commands.trigger(NextDialogue(entity));
        }
    }
}
