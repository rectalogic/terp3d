use crate::{AppPlugin, player::PlayProject, project::TerpProject};
use bevy::prelude::*;
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    #[cfg(feature = "debug")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn start(project_data: Option<String>) {
    set_panic_hook();
    let mut app = App::new();
    app.add_plugins(AppPlugin);
    if let Some(project_data) = project_data
        && let Ok(project) = TerpProject::load(&project_data)
    {
        app.world_mut().trigger(PlayProject { project });
    }
    app.run();
}
