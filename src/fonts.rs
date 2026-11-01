use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, load_fonts);
}

fn load_fonts(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Fonts {
        regular: assets.load("font/Regular.ttf"),
        regular_italic: assets.load("font/Italic.ttf"),
        bold: assets.load("font/Bold.ttf"),
        bold_italic: assets.load("font/BoldItalic.ttf"),
    });
}

#[derive(Resource)]
pub struct Fonts {
    pub regular: Handle<Font>,
    pub regular_italic: Handle<Font>,
    pub bold: Handle<Font>,
    pub bold_italic: Handle<Font>,
}

impl Fonts {
    pub fn get(&self, bold: bool, italic: bool) -> Handle<Font> {
        match (bold, italic) {
            (false, false) => self.regular.clone(),
            (false, true) => self.regular_italic.clone(),
            (true, false) => self.bold.clone(),
            (true, true) => self.bold_italic.clone(),
        }
    }
}
