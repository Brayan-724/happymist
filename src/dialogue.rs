use core::fmt;
use std::string::IntoChars;

use bevy::color::palettes::tailwind;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

use crate::client::task::TaskId;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_dialogue_animation, update_dialogue_char_animation),
    )
    .add_observer(setup_dialogue)
    .add_observer(handle_click.run_if(is_dialogue_active));
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
    pub text: IntoChars,
    pub text_len: usize,
    pub font_size: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DialogueState {
    Presenting,
    Ended,
}

#[derive(Component)]
struct DialogueText(Entity);

#[derive(Component, Default)]
pub struct DialogueAnimated {
    pub elapsed: f32,
    pub next_char: f32,
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

    commands
        .spawn((
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
        .add_child(text);

    commands
        .entity(event.entity)
        .insert((DialogueState::Presenting, DialogueText(text)));
}

const TIME_PER_CHAR: f32 = 0.08;
const MIN_TIME: f32 = 0.5;
const MAX_TIME: f32 = 5.0;

fn update_dialogue_animation(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(
        Entity,
        &mut DialogueState,
        &mut Dialogue,
        &mut DialogueAnimated,
        &DialogueText,
    )>,
) {
    for (dialogue_entity, mut state, mut dialogue, mut animation, text) in query {
        let max_t = (dialogue.text_len as f32 * TIME_PER_CHAR).clamp(MIN_TIME, MAX_TIME);
        let time_per_char = max_t / dialogue.text_len as f32;

        if animation.elapsed >= animation.next_char {
            let chars_to_advance = ((animation.elapsed.min(max_t) - animation.next_char)
                / time_per_char)
                .ceil() as usize;

            for i in 0..(chars_to_advance + 1) {
                // Only animate the last 5 chars
                let is_last = i + 5 >= chars_to_advance;

                if let Some(char) = dialogue.text.next() {
                    animation.next_char += time_per_char;

                    commands.entity(text.0).with_child((
                        TextSpan::new(char),
                        TextFont {
                            font_size: FontSize::Px(1.),
                            ..default()
                        },
                        DialogueCharAnimation {
                            elapsed: if is_last { 0. } else { CHAR_TIME },
                            target_font: dialogue.font_size,
                        },
                    ));
                } else {
                    *state = DialogueState::Ended;
                    commands
                        .entity(dialogue_entity)
                        .remove::<DialogueAnimated>();
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

fn is_dialogue_active(query: Query<(), With<Dialogue>>) -> bool {
    !query.is_empty()
}

fn handle_click(
    mut event: On<PointerPress>,
    mut commands: Commands,
    query: Query<(Entity, &mut DialogueState, Option<&mut DialogueAnimated>)>,
) {
    event.propagate(false);

    for (entity, mut state, animation) in query {
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
}
