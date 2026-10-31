use core::f32;

use bevy::prelude::*;

pub use self::loader::ClientAsset;
use self::loader::ClientAssetLoader;

mod loader;

pub fn plugin(app: &mut App) {
    app.init_asset::<ClientAsset>()
        .init_asset_loader::<ClientAssetLoader>()
        .add_systems(
            Update,
            (load_client, setup_client, update_approaching).chain(),
        );
}

#[derive(Component, Clone, Copy, Debug)]
pub enum Client {
    Ñoquer,
}

impl Client {
    pub fn get_client_file(self) -> &'static str {
        match self {
            Self::Ñoquer => "clients/ñoquer.client.toml",
        }
    }
}

#[derive(Component)]
pub struct LoadedClient(Handle<ClientAsset>);
#[derive(Component)]
pub struct LoadingClient(Handle<ClientAsset>);

const APPROACHING_TIME: f32 = 4.;
const APPROACHING_MIN: Vec2 = vec2(50., 50.);
const APPROACHING_MAX: Vec2 = vec2(350., 350.);
const APPROACHING_BOUNCE: f32 = 8.;

#[derive(Component, Default)]
pub struct ClientApproaching {
    elapsed: f32,
}

#[derive(EntityEvent)]
pub struct ClientApproached(Entity);

fn load_client(
    mut commands: Commands,
    query: Query<(Entity, &Client, Option<&LoadingClient>), Without<LoadedClient>>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<ClientAsset>>,
) {
    for (entity, client, loading_asset) in query {
        let Some(LoadingClient(asset)) = loading_asset else {
            let asset = asset_server.load::<ClientAsset>(client.get_client_file());
            commands.entity(entity).insert(LoadingClient(asset));
            continue;
        };

        let is_loaded = assets.get(asset).is_some();

        if is_loaded {
            commands.entity(entity).insert(LoadedClient(asset.clone()));
        }
    }
}

fn setup_client(
    mut commands: Commands,
    query: Query<(Entity, &LoadedClient), Added<LoadedClient>>,
    assets: Res<AssetServer>,
    client_assets: Res<Assets<ClientAsset>>,
) {
    for (entity, client) in query {
        let Some(client) = client_assets.get(client.0.id()) else {
            continue;
        };

        commands.entity(entity).insert((
            ClientApproaching::default(),
            Sprite {
                image: assets.load(client.sprite.clone()),
                custom_size: Some(vec2(50., 50.)),
                image_mode: SpriteImageMode::Scale(SpriteScalingMode::FitCenter),
                ..default()
            },
        ));
    }
}

fn update_approaching(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut Transform, &mut ClientApproaching)>,
) {
    let Ok((client, mut sprite, mut transform, mut approaching)) = query.single_mut() else {
        return;
    };

    let t = approaching.elapsed / APPROACHING_TIME;

    approaching.elapsed += time.delta_secs();

    sprite.custom_size = Some(APPROACHING_MIN.lerp(APPROACHING_MAX, t));

    transform.translation.y = (t * f32::consts::PI * APPROACHING_BOUNCE).sin().abs() * 100. - 200.;

    if t > 1. {
        commands
            .entity(client)
            .remove::<ClientApproaching>()
            .trigger(ClientApproached);
    }
}
