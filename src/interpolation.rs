use bevy::{camera::visibility::RenderLayers, prelude::*};

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Interpolation {
    #[default]
    Source,
    Target,
}

static SOURCE_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);
static TARGET_RENDER_LAYER: RenderLayers = RenderLayers::layer(2);

impl Interpolation {
    pub fn render_layers(&self) -> &'static RenderLayers {
        match self {
            Interpolation::Source => &SOURCE_RENDER_LAYER,
            Interpolation::Target => &TARGET_RENDER_LAYER,
        }
    }

    pub fn partition<'s, T>(mut sequence: impl Iterator<Item = (&'s Self, T)>) -> Option<(T, T)> {
        match (sequence.next(), sequence.next()) {
            (Some((&Interpolation::Source, s)), Some((&Interpolation::Target, t))) => Some((s, t)),
            (Some((&Interpolation::Target, t)), Some((&Interpolation::Source, s))) => Some((s, t)),
            _ => None,
        }
    }
}
