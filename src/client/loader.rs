use std::path::PathBuf;

use bevy::asset::{Asset, AssetLoader, AsyncReadExt};
use bevy::platform::collections::HashMap;
use bevy::reflect::{Reflect, TypePath};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use thiserror::Error;

use crate::dialogue::parser::{DialogueContent, dialogue_parse};
use crate::dialogue::{DialogueId, DialogueRef};

#[derive(Asset, Reflect, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ClientAsset {
    pub name: String,
    pub sprite: PathBuf,
    #[serde(rename = "task")]
    pub tasks: Vec<ClientTaskAsset>,
}

impl ClientAsset {
    pub fn get_dialogue(&self, dialogue: DialogueId) -> Option<&ClientTaskDialogueAsset> {
        self.tasks.get(dialogue.0.0)?.dialogues.get(dialogue.1)
    }
}

#[derive(Reflect, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ClientTaskAsset {
    pub id: String,
    pub request: ClientTaskRequestAsset,
    #[serde(rename = "dialogue")]
    pub dialogues: Vec<ClientTaskDialogueAsset>,
}

#[derive(Reflect, Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields, untagged)]
pub enum ClientTaskRequestAsset {
    Item {
        item: String,
        #[serde(default)]
        reveal: bool,
    },
}

#[derive(Reflect, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ClientTaskDialogueAsset {
    pub id: Option<String>,
    #[serde(rename = "text", deserialize_with = "dialogue_de")]
    pub content: DialogueContent,
    #[serde(rename = "ref", default)]
    pub refs: HashMap<String, DialogueRef>,
}

fn dialogue_de<'de, D: Deserializer<'de>>(de: D) -> Result<DialogueContent, D::Error> {
    struct DialogueVisitor;

    impl Visitor<'_> for DialogueVisitor {
        type Value = DialogueContent;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("valid dialogue format")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            dialogue_parse(v).map_err(|err| serde::de::Error::custom(format!("{err:?}")))
        }
    }

    de.deserialize_str(DialogueVisitor)
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
        &["toml"]
    }
}
