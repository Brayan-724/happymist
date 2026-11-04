use bevy::prelude::*;

#[derive(Component)]
pub struct SlideOut {
    direction: Vec2,
    timer: Timer,
}

impl SlideOut {
    pub fn random(seconds: f32) -> Self {
        Self {
            direction: random_direction(),
            timer: Timer::from_seconds(seconds, TimerMode::Once),
        }
    }
}

fn random_direction() -> Vec2 {
    match rand::random_range(0..4) {
        0 => Vec2::X,
        1 => Vec2::NEG_X,
        2 => Vec2::Y,
        _ => Vec2::NEG_Y,
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (slide_in, slide_out, shake, fall_away, launched));
}

#[derive(Component)]
pub struct SlideIn {
    direction: Vec2,
    timer: Timer,
}

#[derive(EntityEvent)]
pub struct SlideInFinished(pub Entity);

// starts off screen so it doesn't flash in place on the first frame
pub fn slide_in_random(seconds: f32) -> impl Bundle {
    let direction = random_direction();
    (
        SlideIn {
            direction,
            timer: Timer::from_seconds(seconds, TimerMode::Once),
        },
        UiTransform {
            translation: Val2::new(percent(direction.x * 100.), percent(direction.y * 100.)),
            ..default()
        },
    )
}

fn slide_in(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SlideIn, &mut UiTransform)>,
) {
    for (entity, mut slide, mut transform) in &mut query {
        slide.timer.tick(time.delta());

        let t = (1. - EaseFunction::CubicOut.sample_clamped(slide.timer.fraction())) * 100.;
        transform.translation = Val2::new(
            percent(slide.direction.x * t),
            percent(slide.direction.y * t),
        );

        if slide.timer.just_finished() {
            commands
                .entity(entity)
                .remove::<SlideIn>()
                .trigger(SlideInFinished);
        }
    }
}

fn slide_out(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SlideOut, &mut UiTransform)>,
) {
    for (entity, mut slide, mut transform) in &mut query {
        slide.timer.tick(time.delta());

        let t = EaseFunction::CubicIn.sample_clamped(slide.timer.fraction()) * 100.;
        transform.translation = Val2::new(
            percent(slide.direction.x * t),
            percent(slide.direction.y * t),
        );

        if slide.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct Shake {
    timer: Timer,
    strength: f32,
}

impl Shake {
    pub fn new(seconds: f32, strength: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            strength,
        }
    }
}

fn shake(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Shake, &mut UiTransform)>,
) {
    for (entity, mut shake, mut transform) in &mut query {
        shake.timer.tick(time.delta());

        let strength = shake.strength * shake.timer.fraction_remaining();
        transform.translation = Val2::new(
            px(rand::random_range(-1.0f32..1.0) * strength),
            px(rand::random_range(-1.0f32..1.0) * strength),
        );

        if shake.timer.just_finished() {
            transform.translation = Val2::new(px(0), px(0));
            commands.entity(entity).remove::<Shake>();
        }
    }
}

const FALL_TIME: f32 = 0.8;
const FALL_GRAVITY: f32 = 1500.;
const FALL_DRIFT: f32 = 60.;
const FALL_SPIN: f32 = 60.;

#[derive(Component, Default)]
pub struct FallAway {
    elapsed: f32,
    start_rotation: Option<Rot2>,
}

impl FallAway {
    pub fn opacity(&self) -> f32 {
        (1. - self.elapsed / FALL_TIME).max(0.)
    }
}

fn fall_away(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FallAway, &mut UiTransform, &mut Visibility)>,
    children: Query<&Children>,
    mut images: Query<&mut ImageNode>,
    mut backgrounds: Query<&mut BackgroundColor>,
    mut texts: Query<&mut TextColor>,
) {
    for (entity, mut fall, mut transform, mut visibility) in &mut query {
        fall.elapsed += time.delta_secs();
        let t = fall.elapsed;
        let start_rotation = *fall.start_rotation.get_or_insert(transform.rotation);
        let opacity = fall.opacity();

        transform.translation = Val2::new(px(t * FALL_DRIFT), px(0.5 * FALL_GRAVITY * t * t));
        transform.rotation = start_rotation * Rot2::degrees(t * FALL_SPIN);

        for target in std::iter::once(entity).chain(children.iter_descendants(entity)) {
            if let Ok(mut image) = images.get_mut(target) {
                image.color.set_alpha(opacity);
            }
            if let Ok(mut background) = backgrounds.get_mut(target)
                && background.0.alpha() > 0.
            {
                background.0.set_alpha(opacity);
            }
            if let Ok(mut text) = texts.get_mut(target) {
                text.0.set_alpha(opacity);
            }
        }

        if t >= FALL_TIME {
            *visibility = Visibility::Hidden;
            commands.entity(entity).remove::<FallAway>();
        }
    }
}

const LAUNCH_TIME: f32 = 0.9;
const LAUNCH_GRAVITY: f32 = 1200.;

#[derive(Component)]
pub struct Launched {
    velocity: Vec2,
    spin: f32,
    elapsed: f32,
}

impl Launched {
    pub fn random() -> Self {
        let direction = Rot2::degrees(rand::random_range(0.0..360.)) * Vec2::X;
        Self {
            velocity: direction * rand::random_range(250.0..650.),
            spin: rand::random_range(-360.0..360.),
            elapsed: 0.,
        }
    }
}

fn launched(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Launched, &mut UiTransform, &mut ImageNode)>,
) {
    for (entity, mut launch, mut transform, mut image) in &mut query {
        launch.elapsed += time.delta_secs();
        let t = launch.elapsed;

        transform.translation = Val2::new(
            px(launch.velocity.x * t),
            px(launch.velocity.y * t + 0.5 * LAUNCH_GRAVITY * t * t),
        );
        transform.rotation = Rot2::degrees(t * launch.spin);
        image.color = Color::WHITE.with_alpha(1. - t / LAUNCH_TIME);

        if t >= LAUNCH_TIME {
            commands.entity(entity).despawn();
        }
    }
}
