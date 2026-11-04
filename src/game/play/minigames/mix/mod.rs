use bevy::color::palettes::tailwind;
use bevy::{
    prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

use cut_line::{CUTS_TO_WIN, CutCount, CutFinished};

use super::ActiveMinigame;

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
