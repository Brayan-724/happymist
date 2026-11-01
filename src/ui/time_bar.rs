use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_time_bars);
}

#[derive(Component)]
pub struct TimeBar(pub Timer);

#[derive(Component)]
struct TimeBarFill;

#[derive(EntityEvent)]
pub struct TimeBarFinished(Entity);

pub fn time_bar(seconds: f32) -> impl Bundle {
    (
        Name::new("TimeBar"),
        TimeBar(Timer::from_seconds(seconds, TimerMode::Once)),
        Node {
            width: percent(100),
            height: px(5),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        children![(
            TimeBarFill,
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgb(0.9, 0.3, 0.3)),
        )],
    )
}

fn update_time_bars(
    mut commands: Commands,
    time: Res<Time>,
    mut bars: Query<(Entity, &mut TimeBar, &Children)>,
    mut fills: Query<&mut Node, With<TimeBarFill>>,
) {
    for (entity, mut bar, children) in &mut bars {
        bar.0.tick(time.delta());

        for child in children.iter() {
            if let Ok(mut node) = fills.get_mut(child) {
                node.width = percent(bar.0.fraction_remaining() * 100.);
            }
        }

        if bar.0.just_finished() {
            commands.entity(entity).trigger(TimeBarFinished);
        }
    }
}
