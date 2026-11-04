use bevy::color::palettes::tailwind;
use bevy::{
    prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

use cut_line::{CutCount, CutFinished};

use crate::game::level::MinigameFinished;

use super::{ActiveMinigame, hud::FallsOnLose};

pub mod cut_line;

const HALF_FALL_TIME: f32 = 0.7;
const HALF_SPEED: f32 = 150.;
const HALF_GRAVITY: f32 = 2200.;
const HALF_SPIN: f32 = 25.;

#[derive(Component)]
struct Sugarcane;

#[derive(Component)]
struct CutCountText;

#[derive(Component)]
struct CutHalf {
    normal: Vec2,
    side: f32,
    elapsed: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct CutHalfMaterial {
    #[uniform(0)]
    cut: Vec4,
    #[texture(1)]
    #[sampler(2)]
    image: Handle<Image>,
}

impl UiMaterial for CutHalfMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cut_half.wgsl".into()
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins((
        UiMaterialPlugin::<CutHalfMaterial>::default(),
        cut_line::plugin,
    ))
    .add_systems(OnEnter(ActiveMinigame::Cut), setup)
    .add_systems(
        Update,
        (update, animate_halves).run_if(in_state(ActiveMinigame::Cut)),
    )
    .add_observer(on_cut_finished);
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<CutHalfMaterial>>,
) {
    let image: Handle<Image> = assets.load("sprites/minigames/sugarcane.png");

    commands
        .spawn((
            Name::new("CutMinigame"),
            DespawnOnExit(ActiveMinigame::Cut),
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
                Name::new("Sugarcane"),
                Sugarcane,
                FallsOnLose,
                ImageNode::new(image.clone()),
                Node {
                    width: percent(40),
                    ..default()
                },
            ));

            root.spawn((
                MaterialNode(materials.add(CutHalfMaterial {
                    cut: Vec4::ZERO,
                    image,
                })),
                Node {
                    position_type: PositionType::Absolute,
                    width: px(1),
                    height: px(1),
                    ..default()
                },
            ));

            root.spawn((
                Name::new("CutCounter"),
                FallsOnLose,
                BackgroundColor(tailwind::GRAY_800.into()),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(20),
                    left: px(20),
                    padding: UiRect::axes(px(25), px(10)),
                    border_radius: BorderRadius::all(px(20)),
                    ..default()
                },
            ))
            .with_child((
                CutCountText,
                Text::new("0"),
                TextFont {
                    font_size: FontSize::Px(40.),
                    ..default()
                },
            ));
        });
}

pub fn update(mut gizmos: Gizmos) {}

fn on_cut_finished(
    event: On<CutFinished>,
    mut commands: Commands,
    mut materials: ResMut<Assets<CutHalfMaterial>>,
    count: Res<CutCount>,
    mut count_text: Single<&mut Text, With<CutCountText>>,
    sugarcane: Single<(Entity, &ImageNode, &ComputedNode, &ChildOf), With<Sugarcane>>,
) {
    let (entity, image, computed, parent) = *sugarcane;
    let size = computed.size() * computed.inverse_scale_factor;
    let normal = event.rotation * Vec2::Y;

    count_text.0 = count.0.to_string();
    if count.0 >= ActiveMinigame::Cut.info().goal {
        commands.entity(entity).insert(Visibility::Hidden);
        commands.trigger(MinigameFinished);
    }

    for side in [1., -1.] {
        let material = materials.add(CutHalfMaterial {
            cut: normal.extend(side).extend(1.),
            image: image.image.clone(),
        });

        commands.entity(parent.parent()).with_child((
            Name::new("SugarcaneHalf"),
            CutHalf {
                normal,
                side,
                elapsed: 0.,
            },
            MaterialNode(material),
            ZIndex(1),
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                width: px(size.x),
                height: px(size.y),
                margin: UiRect {
                    left: px(-size.x / 2.),
                    top: px(-size.y / 2.),
                    ..default()
                },
                ..default()
            },
        ));
    }
}

fn animate_halves(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<CutHalfMaterial>>,
    mut halves: Query<(
        Entity,
        &mut CutHalf,
        &mut UiTransform,
        &MaterialNode<CutHalfMaterial>,
    )>,
) {
    for (entity, mut half, mut transform, material) in &mut halves {
        half.elapsed += time.delta_secs();
        let t = half.elapsed;

        let apart = half.normal * half.side * HALF_SPEED * t;
        transform.translation = Val2::new(px(apart.x), px(apart.y + 0.5 * HALF_GRAVITY * t * t));
        transform.rotation = Rot2::degrees(half.side * HALF_SPIN * t);

        if let Some(mut material) = materials.get_mut(&material.0) {
            material.cut.w = ((HALF_FALL_TIME - t) / (HALF_FALL_TIME * 0.5)).clamp(0., 1.);
        }

        if t >= HALF_FALL_TIME {
            commands.entity(entity).despawn();
        }
    }
}
