use bevy::app::App;

pub mod anim;
pub mod grab;
pub mod time_bar;

pub fn plugin(app: &mut App) {
    app.add_plugins((anim::plugin, grab::plugin, time_bar::plugin));
}
