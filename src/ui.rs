use bevy::app::App;

pub mod time_bar;

pub fn plugin(app: &mut App) {
    app.add_plugins(time_bar::plugin);
}
