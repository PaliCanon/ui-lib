//! Spheres around groups of points, seen through (drawn first, so lines and points stay on
//! top): their GPU layout, and how the page's three flat lists become them.

use bytemuck::{Pod, Zeroable};

use crate::check;

/// One sphere, as the vertex shader reads it: a group of points seen through.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default, Debug, PartialEq)]
pub struct SphereInstance {
    /// Its centre.
    pub center: [f32; 3],
    /// Its radius, world units; 0 hides it.
    pub radius: f32,
    /// Red, green, blue and the opacity at its rim, 0–1 each.
    pub color: [f32; 4],
}

/// The spheres three flat lists describe, checked: as many centres (3 values), radii and
/// colours (4 values), every value finite.
pub(crate) fn spheres_of(
    centers: &[f32],
    radii: &[f32],
    colors: &[f32],
) -> Result<Vec<SphereInstance>, String> {
    let n = radii.len();
    check::len("sphere centers", centers.len(), n, 3)?;
    check::len("sphere colors", colors.len(), n, 4)?;
    if centers
        .iter()
        .chain(radii)
        .chain(colors)
        .any(|v| !v.is_finite())
    {
        return Err("spheres: every value must be finite".into());
    }
    Ok(centers
        .as_chunks::<3>()
        .0
        .iter()
        .zip(radii)
        .zip(colors.as_chunks::<4>().0)
        .map(|((&center, &radius), &color)| SphereInstance {
            center,
            radius,
            color,
        })
        .collect())
}

#[cfg(test)]
mod sphere_tests {
    use super::spheres_of;

    #[test]
    fn spheres_are_read_from_three_lists() {
        let spheres = spheres_of(&[1.0, 2.0, 3.0], &[4.0], &[0.1, 0.2, 0.3, 0.4]).unwrap();
        assert_eq!(spheres.len(), 1);
        assert_eq!(
            (spheres[0].center, spheres[0].radius),
            ([1.0, 2.0, 3.0], 4.0)
        );
        assert_eq!(spheres[0].color, [0.1, 0.2, 0.3, 0.4]);
        assert!(
            spheres_of(&[1.0, 2.0], &[4.0], &[0.0; 4]).is_err(),
            "a centre short"
        );
        assert!(
            spheres_of(&[0.0; 3], &[4.0], &[0.0; 3]).is_err(),
            "a colour short"
        );
        assert!(
            spheres_of(&[0.0; 3], &[f32::NAN], &[0.0; 4]).is_err(),
            "not finite"
        );
        assert_eq!(
            spheres_of(&[], &[], &[]).unwrap().len(),
            0,
            "none clears them"
        );
    }

    #[test]
    fn the_shader_focal_follows_the_camera() {
        let shader = include_str!("shaders.wgsl");
        let line = shader
            .lines()
            .find(|l| l.starts_with("const FOCAL"))
            .unwrap();
        let value: f32 = line
            .split('=')
            .nth(1)
            .unwrap()
            .trim()
            .trim_end_matches(';')
            .parse()
            .unwrap();
        let expected = 1.0 / (crate::camera::FOV_Y.to_radians() / 2.0).tan();
        assert!((value - expected).abs() < 1e-5, "{value} vs {expected}");
    }
}
