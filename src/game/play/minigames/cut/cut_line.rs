use bevy::{
    prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

use crate::{game::play::minigames::ActiveMinigame, ui::anim::Launched};

const LINE_WIDTH: f32 = 20.;
const LINE_START: f32 = 50. - LINE_WIDTH / 2.;
const LINE_ANGLE: std::ops::Range<f32> = 60.0..120.0; // 90 = straight across the sugarcane
const KNOB_SIZE: f32 = 3.9; // % of the window width
const KNOB_SMOOTHING: f32 = 40.;
const CUT_DONE_AT: f32 = 0.8;
pub const CUTS_TO_WIN: u32 = 3;
const FINISH_COLOR: Color = Color::srgb(0.2, 0.8, 0.3);
const LINE_SPRITE_SIZE: Vec2 = Vec2::new(585., 20.);
const STAR_COUNT: u32 = 14;
const STAR_SIZE: Vec2 = Vec2::new(30., 27.);

#[derive(Resource, Default)]
pub struct CutCount(pub u32);

#[derive(Event)]
pub struct CutFinished {
    pub rotation: Rot2,
}

#[derive(Component)]
struct CutLineRoot {
    star: Handle<Image>,
}

#[derive(Component)]
struct DashedLine;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct CutLineMaterial {
    #[uniform(0)]
    line: Vec2,
    #[uniform(1)]
    finish_color: LinearRgba,
    #[texture(2)]
    #[sampler(3)]
    image: Handle<Image>,
}

impl UiMaterial for CutLineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cut_line.wgsl".into()
    }
}

#[derive(Component, Default)]
pub struct CutKnob {
    pub progress: f32,
    shown: f32,
    drag_start: f32,
}

pub fn plugin(app: &mut App) {
    app.add_plugins(UiMaterialPlugin::<CutLineMaterial>::default())
        .add_systems(OnEnter(ActiveMinigame::Cut), setup)
        .add_systems(Update, update.run_if(in_state(ActiveMinigame::Cut)));
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<CutLineMaterial>>,
) {
    commands.insert_resource(CutCount::default());
    spawn_cut_line(&mut commands, &assets, &mut materials);
}

fn spawn_cut_line(
    commands: &mut Commands,
    assets: &AssetServer,
    materials: &mut Assets<CutLineMaterial>,
) {
    let star = assets.load("sprites/ui/Star.png");
    let line_material = materials.add(CutLineMaterial {
        line: Vec2::new(0., CUT_DONE_AT),
        finish_color: FINISH_COLOR.to_linear(),
        image: assets.load("sprites/ui/DashedLine.png"),
    });
    let from_bottom = if rand::random() { 180. } else { 0. };

    commands
        .spawn((
            CutLineRoot { star },
            DespawnOnExit(ActiveMinigame::Cut),
            ZIndex(1),
            UiTransform::from_rotation(Rot2::degrees(
                rand::random_range(LINE_ANGLE) + from_bottom,
            )),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Name::new("DashedLine"),
                DashedLine,
                MaterialNode(line_material),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(LINE_START),
                    top: percent(50),
                    width: percent(LINE_WIDTH),
                    height: Val::Vw(LINE_WIDTH * LINE_SPRITE_SIZE.y / LINE_SPRITE_SIZE.x),
                    ..default()
                },
                UiTransform::from_translation(Val2::new(px(0), percent(-50.))),
            ));

            root.spawn((
                Name::new("CutStart"),
                CutKnob::default(),
                ImageNode::new(assets.load("sprites/ui/Draggable.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(LINE_START),
                    top: percent(50),
                    width: Val::Vw(KNOB_SIZE),
                    height: Val::Vw(KNOB_SIZE),
                    margin: UiRect {
                        left: Val::Vw(-KNOB_SIZE / 2.),
                        top: Val::Vw(-KNOB_SIZE / 2.),
                        ..default()
                    },
                    ..default()
                },
            ))
            .observe(on_knob_drag_start)
            .observe(on_knob_drag)
            .observe(on_knob_drop);
        });
}

fn on_knob_drag_start(event: On<PointerDragStart>, mut knobs: Query<&mut CutKnob>) {
    if let Ok(mut knob) = knobs.get_mut(event.entity) {
        knob.drag_start = knob.progress;
    }
}

fn on_knob_drag(
    event: On<PointerDrag>,
    mut knobs: Query<&mut CutKnob>,
    root: Single<(&UiTransform, &ComputedNode), With<CutLineRoot>>,
) {
    let Ok(mut knob) = knobs.get_mut(event.entity) else {
        return;
    };

    // only the part of the mouse movement that goes along the line counts
    let (transform, computed) = *root;
    let direction = transform.rotation * Vec2::X;
    let along_line = event.distance.dot(direction);
    let line_length = computed.size().x * computed.inverse_scale_factor * LINE_WIDTH / 100.;
    if line_length <= 0. {
        return;
    }

    let target = (knob.drag_start + along_line / line_length).clamp(0., 1.);
    knob.progress = knob.progress.max(target);
}

fn on_knob_drop(event: On<PointerDragEnd>, mut knobs: Query<&mut CutKnob>) {
    if let Ok(mut knob) = knobs.get_mut(event.entity)
        && knob.progress >= CUT_DONE_AT
    {
        knob.progress = 1.;
    }
}

fn update(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<CutLineMaterial>>,
    time: Res<Time>,
    mut count: ResMut<CutCount>,
    knob: Single<(&mut CutKnob, &mut Node, &UiGlobalTransform, &ComputedNode)>,
    root: Single<(Entity, &CutLineRoot, &UiTransform)>,
    line: Single<
        (&mut Node, &MaterialNode<CutLineMaterial>),
        (With<DashedLine>, Without<CutKnob>),
    >,
) {
    let (mut knob, mut node, global, computed) = knob.into_inner();
    let (root_entity, root, root_transform) = *root;

    let target = knob.progress;
    knob.shown.smooth_nudge(&target, KNOB_SMOOTHING, time.delta_secs());
    if target - knob.shown < 0.01 {
        knob.shown = target;
    }
    let shown = knob.shown;

    node.left = percent(LINE_START + LINE_WIDTH * shown);

    let (mut line_node, line_material) = line.into_inner();
    line_node.left = node.left;
    line_node.width = percent(LINE_WIDTH * (1. - shown));
    if let Some(mut material) = materials.get_mut(&line_material.0) {
        material.line.x = shown;
    }

    if shown >= 1. {
        count.0 += 1;
        commands.trigger(CutFinished {
            rotation: root_transform.rotation,
        });

        let position = global.translation * computed.inverse_scale_factor;
        for _ in 0..STAR_COUNT {
            commands.spawn(star(root.star.clone(), position));
        }

        commands.entity(root_entity).despawn();
        if count.0 < CUTS_TO_WIN {
            spawn_cut_line(&mut commands, &assets, &mut materials);
        }
    }
}

fn star(image: Handle<Image>, position: Vec2) -> impl Bundle {
    (
        Name::new("Star"),
        DespawnOnExit(ActiveMinigame::Cut),
        ZIndex(2),
        Launched::random(),
        ImageNode::new(image),
        Node {
            position_type: PositionType::Absolute,
            left: px(position.x - STAR_SIZE.x / 2.),
            top: px(position.y - STAR_SIZE.y / 2.),
            width: px(STAR_SIZE.x),
            height: px(STAR_SIZE.y),
            ..default()
        },
    )
}
