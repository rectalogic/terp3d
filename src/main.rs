use bevy::prelude::*;
use terp3d::AppPlugin;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    App::new().add_plugins(AppPlugin).run();
}
