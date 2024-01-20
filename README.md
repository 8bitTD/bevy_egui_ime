This is a plugin that supports Japanese input with bevy_egui
![240120](https://github.com/8bitTD/bevy_egui_ime/assets/19583059/e1d3780c-8ced-4dfa-8aee-a2e757801ae8)

```Cargo.toml
bevy = "*"
bevy_egui = "*"
bevy_egui_ime = "*"
```
```main.rs
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::prelude::*;
use bevy_egui_ime::*;

#[derive(Resource, Default)] 
pub struct MyApp{
    single_text: String,
    multi_text: String,
}

fn main() {
    App::new()   
    .add_plugins(DefaultPlugins)
    .add_plugins(EguiPlugin)
    .add_plugins(ImePlugin) 
    .insert_resource(MyApp::default())
    .add_systems(Startup, setup_system)
    .add_systems(Update, ui_system)      
    .run();
}

pub fn setup_system(
    mut egui_context: EguiContexts,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();
    window.ime_enabled = true;
    let mut txt_font = egui::FontDefinitions::default();
    txt_font.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "Meiryo".to_owned());
    let fd = egui::FontData::from_static(include_bytes!("C:/Windows/Fonts/Meiryo.ttc"));
    txt_font.font_data.insert("Meiryo".to_owned(), fd);
    egui_context.ctx_mut().set_fonts(txt_font); 
}

pub fn ui_system(
    mut contexts: EguiContexts, 
    mut app: ResMut<MyApp>, 
    mut ime: ResMut<ImeManager>, 
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        let _teo_s = ime.text_edit_singleline(&mut app.single_text, 400.0, ui, ctx);
        let _teo_m = ime.text_edit_multiline(&mut app.multi_text, 400.0, ui, ctx);
    });
}
```
