#[cfg(not(target_arch = "wasm32"))]
fn main() {
    bevy::app::App::new().add_plugins(terp3d::AppPlugin).run();
}
#[cfg(target_arch = "wasm32")]
fn main() {}
