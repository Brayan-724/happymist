use bevy::prelude::*;

use crate::{
    game::level::MinigameFinished,
    ui::{
        anim::Shake,
        grab::{Grabbable, hit_area},
    },
};

use super::{
    ActiveMinigame,
    hud::{FallsOnLose, LoseDelay},
};

const POT_SPRITE_SIZE: Vec2 = Vec2::new(454., 429.);
const POT_WIDTH: f32 = 40.;
const POT_HIDDEN: f32 = 0.25;
const WATER_CENTER: Vec2 = Vec2::new(226., 176.);
const WATER_RADIUS: Vec2 = Vec2::new(196., 92.);
const SUGARCANE_SPRITE_HEIGHT: f32 = 105.;
const PIECE_HEIGHT: f32 = 8.;
const PIECE_HIT_PADDING: f32 = 1.5;
const SINK_TIME: f32 = 0.5;
const LID_SPRITE_SIZE: Vec2 = Vec2::new(454., 292.);
const LID_REST_TOP: f32 = 1.;
const LID_LOCKED_COLOR: Color = Color::srgb(0.55, 0.55, 0.55);
const LID_CLOSE_TIME: f32 = 0.25;

const PIECES: [(f32, f32, f32, f32); 4] = [
    (0., 135., 5., 35.),
    (135., 237., 17., 60.),
    (237., 342., 76., 60.),
    (342., 454., 87., 35.),
];

#[derive(Resource, Default)]
struct PotCount(u32);

#[derive(Component)]
struct Pot;

#[derive(Component, Default)]
struct PotDraggable {
    offset: Vec2,
    drag_start: Vec2,
}

#[derive(Component)]
struct PotLid;

#[derive(Component)]
struct ClosingLid {
    from: Vec2,
    to: Vec2,
    start_scale: f32,
    elapsed: f32,
}

#[derive(Component)]
struct Sinking {
    elapsed: f32,
    start_scale: f32,
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(ActiveMinigame::Pot), setup)
        .add_systems(
            Update,
            (sink, close_lid).run_if(in_state(ActiveMinigame::Pot)),
        );
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(PotCount::default());

    let sugarcane: Handle<Image> = assets.load("sprites/minigames/sugarcane.png");
    let pot_height = POT_WIDTH * POT_SPRITE_SIZE.y / POT_SPRITE_SIZE.x;

    commands
        .spawn((
            Name::new("PotMinigame"),
            DespawnOnExit(ActiveMinigame::Pot),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("Pot"),
                Pot,
                FallsOnLose,
                ImageNode::new(assets.load("sprites/minigames/Pot.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    bottom: Val::Vw(-pot_height * POT_HIDDEN),
                    width: Val::Vw(POT_WIDTH),
                    height: Val::Vw(pot_height),
                    margin: UiRect::left(Val::Vw(-POT_WIDTH / 2.)),
                    ..default()
                },
            ));

            // can't be grabbed until every piece is in the pot
            root.spawn((
                Name::new("PotLid"),
                PotLid,
                PotDraggable::default(),
                FallsOnLose,
                Pickable::IGNORE,
                ImageNode::new(assets.load("sprites/minigames/Tapa.png"))
                    .with_color(LID_LOCKED_COLOR),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Vw(50. - POT_WIDTH / 2.),
                    top: Val::Vw(LID_REST_TOP),
                    width: Val::Vw(POT_WIDTH),
                    height: Val::Vw(POT_WIDTH * LID_SPRITE_SIZE.y / LID_SPRITE_SIZE.x),
                    ..default()
                },
            ))
            .observe(on_piece_drag_start)
            .observe(on_piece_drag)
            .observe(on_lid_drop);

            for (from, to, left, top) in PIECES {
                root.spawn((
                    Name::new("PotPiece"),
                    PotDraggable::default(),
                    Grabbable::default(),
                    FallsOnLose,
                    ZIndex(1),
                    ImageNode::new(sugarcane.clone()).with_rect(Rect::new(
                        from,
                        0.,
                        to,
                        SUGARCANE_SPRITE_HEIGHT,
                    )),
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(left),
                        top: percent(top),
                        width: Val::Vw(PIECE_HEIGHT * (to - from) / SUGARCANE_SPRITE_HEIGHT),
                        height: Val::Vw(PIECE_HEIGHT),
                        ..default()
                    },
                    children![hit_area(PIECE_HIT_PADDING)],
                ))
                .observe(on_piece_drag_start)
                .observe(on_piece_drag)
                .observe(on_piece_drop);
            }
        });
}

fn on_piece_drag_start(event: On<PointerDragStart>, mut pieces: Query<&mut PotDraggable>) {
    if let Ok(mut piece) = pieces.get_mut(event.entity) {
        piece.drag_start = piece.offset;
    }
}

fn on_piece_drag(
    event: On<PointerDrag>,
    losing: Option<Res<LoseDelay>>,
    mut pieces: Query<(&mut PotDraggable, &mut Node)>,
) {
    if losing.is_some() {
        return;
    }
    let Ok((mut piece, mut node)) = pieces.get_mut(event.entity) else {
        return;
    };

    piece.offset = piece.drag_start + event.distance;
    node.margin.left = px(piece.offset.x);
    node.margin.top = px(piece.offset.y);
}

fn on_piece_drop(
    event: On<PointerDragEnd>,
    mut commands: Commands,
    losing: Option<Res<LoseDelay>>,
    mut count: ResMut<PotCount>,
    pieces: Query<(&UiGlobalTransform, &UiTransform), With<PotDraggable>>,
    pot: Single<(&UiGlobalTransform, &ComputedNode), With<Pot>>,
    lid: Single<(Entity, &mut ImageNode), With<PotLid>>,
) {
    if losing.is_some() {
        return;
    }
    let Ok((piece, transform)) = pieces.get(event.entity) else {
        return;
    };

    let (pot_global, pot_computed) = *pot;
    if !in_water(
        piece.translation,
        pot_global.translation,
        pot_computed.size(),
    ) {
        return;
    }

    commands
        .entity(event.entity)
        .despawn_children()
        .remove::<Grabbable>()
        .insert((
            Sinking {
                elapsed: 0.,
                start_scale: transform.scale.x,
            },
            Pickable::IGNORE,
        ));

    count.0 += 1;
    if count.0 >= ActiveMinigame::Pot.info().goal {
        let (lid_entity, mut lid_image) = lid.into_inner();
        lid_image.color = Color::WHITE;
        commands
            .entity(lid_entity)
            .remove::<Pickable>()
            .insert((Grabbable::default(), Shake::new(0.3, 8.)));
    }
}

fn on_lid_drop(
    event: On<PointerDragEnd>,
    mut commands: Commands,
    losing: Option<Res<LoseDelay>>,
    lids: Query<
        (
            &PotDraggable,
            &UiGlobalTransform,
            &UiTransform,
            &ComputedNode,
        ),
        With<PotLid>,
    >,
    pot: Single<(&UiGlobalTransform, &ComputedNode), With<Pot>>,
) {
    if losing.is_some() {
        return;
    }
    let Ok((lid, lid_global, transform, lid_computed)) = lids.get(event.entity) else {
        return;
    };

    let (pot_global, pot_computed) = *pot;
    if !in_water(
        lid_global.translation,
        pot_global.translation,
        pot_computed.size(),
    ) {
        return;
    }

    // top edges line up, the lid's ellipse is the same shape as the pot's rim
    let aligned = vec2(
        pot_global.translation.x,
        pot_global.translation.y - pot_computed.size().y / 2. + lid_computed.size().y / 2.,
    );
    let to_aligned = (aligned - lid_global.translation) * lid_computed.inverse_scale_factor;

    commands.entity(event.entity).remove::<Grabbable>().insert((
        ClosingLid {
            from: lid.offset,
            to: lid.offset + to_aligned,
            start_scale: transform.scale.x,
            elapsed: 0.,
        },
        Pickable::IGNORE,
    ));

    commands.trigger(MinigameFinished);
}

fn close_lid(
    mut commands: Commands,
    time: Res<Time>,
    mut lids: Query<(Entity, &mut ClosingLid, &mut Node, &mut UiTransform)>,
) {
    for (entity, mut closing, mut node, mut transform) in &mut lids {
        closing.elapsed += time.delta_secs();
        let t = EaseFunction::CubicOut.sample_clamped(closing.elapsed / LID_CLOSE_TIME);

        let offset = closing.from.lerp(closing.to, t);
        node.margin.left = px(offset.x);
        node.margin.top = px(offset.y);
        transform.scale = Vec2::splat(closing.start_scale.lerp(1., t));

        if closing.elapsed >= LID_CLOSE_TIME {
            commands.entity(entity).remove::<ClosingLid>();
        }
    }
}

fn in_water(position: Vec2, pot_center: Vec2, pot_size: Vec2) -> bool {
    let on_sprite = (position - pot_center) / pot_size * POT_SPRITE_SIZE + POT_SPRITE_SIZE / 2.;
    ((on_sprite - WATER_CENTER) / WATER_RADIUS).length() <= 1.
}

fn sink(
    mut commands: Commands,
    time: Res<Time>,
    mut pieces: Query<(Entity, &mut Sinking, &mut UiTransform, &mut ImageNode)>,
) {
    for (entity, mut sinking, mut transform, mut image) in &mut pieces {
        sinking.elapsed += time.delta_secs();
        let t = (sinking.elapsed / SINK_TIME).min(1.);

        transform.scale =
            Vec2::splat(sinking.start_scale * (1. - EaseFunction::QuadraticIn.sample_clamped(t)));
        image.color.set_alpha(1. - t * t);

        if t >= 1. {
            commands.entity(entity).despawn();
        }
    }
}
