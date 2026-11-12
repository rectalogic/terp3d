use std::iter;

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use bevy::{pbr::ExtendedMaterial, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{drawings::Drawing, interpolation::Interpolation, rim_effect::RimEffect};

#[derive(Serialize, Deserialize)]
pub struct TerpProject {
    pub camera: Transform,
    pub drawings: Vec<TerpDrawing>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TerpDrawing {
    pub colors: (Oklaba, Oklaba),
    pub scales: (f32, f32),
    pub translations: Vec<(Vec3, Vec3)>,
}

impl TerpProject {
    pub fn build(
        camera_entity: Entity,
        q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
        q_transforms: Query<&Transform>,
        extended_materials: Res<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    ) -> Self {
        let drawings = iter::zip(
            iter_drawings(Interpolation::Source, &q_drawings),
            iter_drawings(Interpolation::Target, &q_drawings),
        )
        .map(
            |((source_drawing, source_children), (target_drawing, target_children))| {
                let scales = (source_drawing.scale, target_drawing.scale);

                let target_material = extended_materials
                    .get(target_drawing.material.id())
                    .expect("material");
                let source_material = extended_materials
                    .get(source_drawing.material.id())
                    .expect("material");
                let colors: (Oklaba, Oklaba) = (
                    source_material.base.base_color.into(),
                    target_material.base.base_color.into(),
                );

                let source_translations: Vec<_> =
                    child_translations(source_children, &q_transforms);
                let target_translations: Vec<_> =
                    child_translations(target_children, &q_transforms);
                let translations: Vec<_> =
                    zip_pad(&source_translations, &target_translations).collect();

                TerpDrawing {
                    colors,
                    scales,
                    translations,
                }
            },
        )
        .collect();

        let transform = q_transforms.get(camera_entity).unwrap();
        TerpProject {
            camera: *transform,
            drawings,
        }
    }

    pub fn save(&self) -> Result<String> {
        let output: Vec<u8> = postcard::to_allocvec(self)?;
        let compressed =
            yazi::compress(&output, yazi::Format::Zlib, yazi::CompressionLevel::Default)
                .map_err(|e| format!("{e:?}"))?;
        Ok(URL_SAFE.encode(&compressed))
    }

    pub fn load(data: &str) -> Result<TerpProject> {
        let compressed = URL_SAFE.decode(data)?;
        let (decompressed, checksum) =
            yazi::decompress(&compressed, yazi::Format::Zlib).map_err(|e| format!("{e:?}"))?;
        if let Some(checksum) = checksum
            && yazi::Adler32::from_buf(&decompressed).finish() != checksum
        {
            return Err("invalid checksum".into());
        }
        let project: TerpProject = postcard::from_bytes(&decompressed)?;
        Ok(project)
    }
}

fn iter_drawings<'q>(
    interpolation: Interpolation,
    q_drawings: &'q Query<(&Drawing, &Interpolation, &Children)>,
) -> impl Iterator<Item = (&'q Drawing, &'q Children)> + 'q {
    q_drawings
        .iter()
        .filter_map(move |(drawing, interp, children)| {
            (*interp == interpolation).then_some((drawing, children))
        })
}

fn child_translations(children: &Children, q_transforms: &Query<&Transform>) -> Vec<Vec3> {
    children
        .iter()
        .filter_map(|entity| q_transforms.get(entity).map(|t| t.translation).ok())
        .collect()
}

fn zip_pad(a: &[Vec3], b: &[Vec3]) -> impl Iterator<Item = (Vec3, Vec3)> {
    let (m, n) = (a.len(), b.len());
    (0..m.max(n)).map(move |i| {
        if m == 0 {
            (b[i], b[i])
        } else if n == 0 {
            (a[i], a[i])
        } else if m <= n {
            let j = (2 * i + 1) * m / (2 * n);
            (a[j], b[i])
        } else {
            let j = (2 * i + 1) * n / (2 * m);
            (a[i], b[j])
        }
    })
}
