use crate::{AppPlugin, player::PlayProject, project::TerpProject};
use bevy::prelude::*;
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    #[cfg(feature = "debug")]
    console_error_panic_hook::set_once();
}

#[derive(Resource)]
struct InitialProject(TerpProject);

#[wasm_bindgen]
pub fn start(project_data: Option<String>) {
    set_panic_hook();
    let mut app = App::new();
    if let Some(project_data) = project_data
        && let Ok(project) = TerpProject::load(&project_data)
    {
        app.insert_resource(InitialProject(project))
            .add_systems(PostStartup, setup);
    };
    app.add_plugins(AppPlugin).run();
}

fn setup(world: &mut World) {
    if let Some(initial_project) = world.remove_resource::<InitialProject>() {
        world.commands().trigger(PlayProject {
            project: initial_project.0,
        });
    }
}
