use crate::AppPlugin;
use bevy::prelude::*;
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    #[cfg(feature = "debug")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn start() {
    set_panic_hook();
    App::new().add_plugins(AppPlugin).run();
}
