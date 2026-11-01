use bevy::{
    app::App,
    state::{app::AppExtStates, state::States},
};

use crate::{game, main_menu};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Scene {
    MainMenu,
    #[default]
    Game,
}

pub fn plugin(app: &mut App) {
    app.init_state::<Scene>()
        .add_plugins(game::plugin)
        .add_plugins(main_menu::plugin);
}
