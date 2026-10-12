use bevy::{camera::visibility::RenderLayers, color::palettes::css, prelude::*};

use crate::editor::{DrawingEditor, EditorPlugin};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Terp3D".into(),
                    resolution: (1200, 600).into(),
                    ..default()
                }),
                ..default()
            }),
            EditorPlugin,
        ))
        .add_systems(Startup, (setup, setup_axes));
    }
}

const SOURCE_LAYER: RenderLayers = RenderLayers::layer(1);
const TARGET_LAYER: RenderLayers = RenderLayers::layer(2);

fn setup(mut commands: Commands) {
    let all_layers = SOURCE_LAYER.union(&TARGET_LAYER);
    commands.spawn_scene_list(bsn_list! {
        Camera2d
        IsDefaultUiCamera
        --
        @DrawingEditor { @render_layers: SOURCE_LAYER }
        BorderColor::all(css::YELLOW)
        FixedNode
        Node {
            left: percent(0),
            width: percent(50),
            height: percent(100),
            border: UiRect::all(px(8)),
        }
        --
        @DrawingEditor { @render_layers: TARGET_LAYER }
        BorderColor::all(css::BLUE)
        FixedNode
        Node {
            left: percent(50),
            width: percent(50),
            height: percent(100),
            border: UiRect::all(px(8)),
        }
        --
        @cube(Color::srgb_u8(124, 144, 255))
        SOURCE_LAYER
        Transform::from_xyz(1.0, 1.0, 1.0)
        --
        @cube(Color::srgb_u8(124, 0, 0))
        TARGET_LAYER
        Transform::from_xyz(-1.0, -1.0, -1.0)
        --
        all_layers
        DirectionalLight {
            illuminance: light_consts::lux::RAW_SUNLIGHT,
        }
        Transform::from_xyz(1.0, 0.15, 1.0).looking_at(Vec3::ZERO, Vec3::Y)
    });
}

fn cube(color: Color) -> impl Scene {
    bsn! {
        #Cube
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(color))
    }
}

fn setup_axes(mut commands: Commands, mut gizmo_assets: ResMut<Assets<GizmoAsset>>) {
    let mut gizmo_asset = GizmoAsset::default();
    let cells = UVec2::splat(15);
    let spacing = Vec2::splat(0.3);
    let alpha = 0.3;

    gizmo_asset
        .grid(
            Isometry3d::IDENTITY,
            cells,
            spacing,
            Srgba::new(1.0, 0.0, 0.0, alpha),
        )
        .outer_edges();

    let xz_iso = Isometry3d::new(
        Vec3::ZERO,
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
    );
    gizmo_asset
        .grid(xz_iso, cells, spacing, Srgba::new(0.0, 1.0, 0.0, alpha))
        .outer_edges();

    let yz_iso = Isometry3d::new(
        Vec3::ZERO,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    );
    gizmo_asset
        .grid(yz_iso, cells, spacing, Srgba::new(0.0, 0.0, 1.0, alpha))
        .outer_edges();

    let gizmo_handle = gizmo_assets.add(gizmo_asset);
    let all_layers = SOURCE_LAYER.union(&TARGET_LAYER);
    commands.spawn_scene(bsn! {
        Gizmo {
            handle: gizmo_handle,
        }
        all_layers
    });
}
