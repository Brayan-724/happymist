use core::fmt;

use bevy::prelude::*;

use crate::client::task::TaskId;

pub fn plugin(app: &mut App) {
    app;
}

#[derive(Reflect, Clone, Copy, Debug, Default)]
pub struct DialogueId(pub TaskId, pub usize);

impl fmt::Display for DialogueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:0>2}:{:0>2}", self.0.0, self.1))
    }
}

pub struct Dialogue;
