use std::path::PathBuf;

use bevy::asset::{Asset, AssetLoader, AsyncReadExt};
use bevy::reflect::{Reflect, TypePath};
use serde::Deserialize;
use thiserror::Error;

#[derive(Asset, Reflect, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientAsset {
    pub name: String,
    pub sprite: PathBuf,
    #[serde(rename = "task")]
    pub tasks: Vec<ClientTaskAsset>,
}

#[derive(Reflect, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientTaskAsset {
    pub id: String,
    pub request: String,
    #[serde(rename = "dialogue")]
    pub dialogues: Vec<ClientTaskDialogueAsset>,
}

#[derive(Reflect, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientTaskDialogueAsset {
    pub id: Option<String>,
    pub text: String,
}

#[derive(Default, TypePath)]
pub struct ClientAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ClientAssetLoaderError {
    #[error("Cannot load file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cannot parse file: {0}")]
    TomlParse(#[from] toml::de::Error),
}

impl AssetLoader for ClientAssetLoader {
    type Asset = ClientAsset;
    type Error = ClientAssetLoaderError;
    type Settings = ();

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _: &Self::Settings,
        _: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf).await?;
        Ok(toml::from_str(&buf)?)
    }

    fn extensions(&self) -> &[&str] {
        &["client.toml"]
    }
}
