use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Interpolation {
    #[default]
    Source,
    Target,
}

impl Interpolation {
    pub fn partition<'s, T>(mut sequence: impl Iterator<Item = (&'s Self, T)>) -> Option<(T, T)> {
        match (sequence.next(), sequence.next()) {
            (Some((&Interpolation::Source, s)), Some((&Interpolation::Target, t))) => Some((s, t)),
            (Some((&Interpolation::Target, t)), Some((&Interpolation::Source, s))) => Some((s, t)),
            _ => None,
        }
    }
}
