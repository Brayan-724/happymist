use core::f32;

use bevy::{log, prelude::*};

use crate::dialogue::{Dialogue, DialogueAnimated, DialogueId, NextDialogue};
use crate::game::GameState;

use self::loader::{ClientAsset, ClientAssetLoader};
use self::task::TaskId;

mod loader;
pub mod task;

pub fn plugin(app: &mut App) {
    app.init_asset::<ClientAsset>()
        .init_asset_loader::<ClientAssetLoader>()
        .add_systems(
            Update,
            (load_client, setup_client, update_approaching)
                .chain()
                .run_if(in_state(GameState::Dialogue)),
        )
        .add_observer(on_client_approached.run_if(in_state(GameState::Dialogue)))
        .add_observer(on_next_dialogue.run_if(in_state(GameState::Dialogue)));
}

#[derive(Component, Clone, Copy, Debug)]
pub enum Client {
    Ñoquer,
}

#[derive(Component, Clone, Debug)]
pub struct ClientDetails {
    pub name: String,
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

const APPROACHING_TIME: f32 = 3.;
const APPROACHING_MIN: Vec2 = vec2(150., 150.);
const APPROACHING_MAX: Vec2 = vec2(550., 550.);
const APPROACHING_BOUNCE: f32 = 7.;

#[derive(Component, Default)]
pub struct ClientApproaching {
    elapsed: f32,
}

#[derive(EntityEvent)]
pub struct ClientApproached(Entity);

#[derive(Component, Reflect)]
pub struct ClientTasking(pub TaskId);

#[derive(Component, Reflect)]
pub struct ClientTalking(pub DialogueId);

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

        let loaded_client = assets.get(asset);

        if let Some(loaded_client) = loaded_client {
            commands.entity(entity).insert((
                LoadedClient(asset.clone()),
                ClientDetails {
                    name: loaded_client.name.clone(),
                },
            ));
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
    query: Query<(Entity, &mut Sprite, &mut Transform, &mut ClientApproaching)>,
) {
    for (client, mut sprite, mut transform, mut approaching) in query {
        let t = approaching.elapsed / APPROACHING_TIME;

        approaching.elapsed += time.delta_secs();

        sprite.custom_size = Some(APPROACHING_MIN.lerp(
            APPROACHING_MAX,
            EaseFunction::SmoothStepIn.sample_unchecked(t),
        ));

        transform.translation.y =
            (t * f32::consts::PI * APPROACHING_BOUNCE).sin().abs() * 50. - 200.;

        if t > 1. {
            commands
                .entity(client)
                .remove::<ClientApproaching>()
                .trigger(ClientApproached);
        }
    }
}

fn on_client_approached(
    event: On<ClientApproached>,
    mut commands: Commands,
    query: Query<(&ClientDetails, &LoadedClient)>,
    assets: Res<Assets<ClientAsset>>,
) {
    let Ok((client, client_asset)) = query.get(event.0) else {
        return;
    };

    let Some(client_asset) = assets.get(&client_asset.0) else {
        return;
    };

    let task_id = TaskId(0);
    let dialogue_id = DialogueId(task_id, 0);

    let Some(dialogue) = client_asset.get_dialogue(dialogue_id) else {
        log::warn!("Check configuration of clients, {dialogue_id} no dialogue found");
        return;
    };

    commands
        .entity(event.0)
        .insert((ClientTasking(task_id), ClientTalking(dialogue_id)));

    commands.spawn((
        dialogue_id,
        Dialogue {
            name: client.name.clone(),
            text_len: dialogue.text.len(),
            text: dialogue.text.clone().into_chars(),
            font_size: 18.,
        },
        DialogueAnimated::default(),
    ));
}

fn on_next_dialogue(
    event: On<NextDialogue>,
    mut commands: Commands,
    client: Single<(&ClientDetails, &LoadedClient)>,
    query_dialogues: Query<&DialogueId>,
    assets: Res<Assets<ClientAsset>>,
) {
    commands.entity(event.0).despawn();

    let Ok(dialogue_id) = query_dialogues.get(event.0) else {
        return;
    };

    let (client, client_asset) = *client;
    let Some(client_asset) = assets.get(&client_asset.0) else {
        return;
    };

    let dialogue_id = DialogueId(dialogue_id.0, dialogue_id.1 + 1);

    if let Some(dialogue) = client_asset.get_dialogue(dialogue_id) {
        commands.spawn((
            dialogue_id,
            Dialogue {
                name: client.name.clone(),
                text_len: dialogue.text.len(),
                text: dialogue.text.clone().into_chars(),
                font_size: 18.,
            },
            DialogueAnimated::default(),
        ));
    }
}
