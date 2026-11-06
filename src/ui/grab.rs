use bevy::{
    picking::hover::Hovered,
    prelude::*,
    window::{CursorIcon, SystemCursorIcon},
};

const HOVER_SCALE: f32 = 1.1;
const HELD_SCALE: f32 = 1.2;
const SCALE_SMOOTHING: f32 = 25.;

#[derive(Component, Default)]
#[require(Hovered)]
pub struct Grabbable {
    held: bool,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (scale_grabbables, update_cursor))
        .add_observer(on_press)
        .add_observer(on_release)
        .add_observer(on_drag_end);
}

pub fn hit_area(padding_vw: f32) -> impl Bundle {
    let padding = Val::Vw(-padding_vw);
    (
        Name::new("HitArea"),
        Node {
            position_type: PositionType::Absolute,
            left: padding,
            top: padding,
            right: padding,
            bottom: padding,
            ..default()
        },
    )
}

fn set_held(entity: Entity, held: bool, grabbables: &mut Query<&mut Grabbable>) {
    if let Ok(mut grabbable) = grabbables.get_mut(entity) {
        grabbable.held = held;
    }
}

fn on_press(event: On<PointerPress>, mut grabbables: Query<&mut Grabbable>) {
    set_held(event.entity, true, &mut grabbables);
}

fn on_release(event: On<PointerRelease>, mut grabbables: Query<&mut Grabbable>) {
    set_held(event.entity, false, &mut grabbables);
}

fn on_drag_end(event: On<PointerDragEnd>, mut grabbables: Query<&mut Grabbable>) {
    set_held(event.entity, false, &mut grabbables);
}

fn scale_grabbables(
    time: Res<Time>,
    mut grabbables: Query<(&Grabbable, &Hovered, &mut UiTransform)>,
) {
    for (grabbable, hovered, mut transform) in &mut grabbables {
        let target = if grabbable.held {
            HELD_SCALE
        } else if hovered.get() {
            HOVER_SCALE
        } else {
            1.
        };

        transform
            .scale
            .smooth_nudge(&Vec2::splat(target), SCALE_SMOOTHING, time.delta_secs());
    }
}

fn update_cursor(
    mut commands: Commands,
    window: Single<Entity, With<Window>>,
    grabbables: Query<(&Grabbable, &Hovered)>,
    mut shown: Local<Option<SystemCursorIcon>>,
) {
    let wanted = if grabbables.iter().any(|(grabbable, _)| grabbable.held) {
        Some(SystemCursorIcon::Grabbing)
    } else if grabbables.iter().any(|(_, hovered)| hovered.get()) {
        Some(SystemCursorIcon::Grab)
    } else {
        None
    };

    if *shown != wanted {
        *shown = wanted;
        commands.entity(*window).insert(CursorIcon::System(
            wanted.unwrap_or(SystemCursorIcon::Default),
        ));
    }
}
