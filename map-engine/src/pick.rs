//! Which point is under the pointer: project every shown point to the screen and keep the
//! nearest within its drawn radius (at least a few pixels, so small points stay clickable).

use glam::{Mat4, Vec2, Vec3};

/// Minimum pick radius, CSS pixels.
const MIN_RADIUS: f32 = 6.0;

/// The point under `at` (CSS pixels from the canvas' top left), or None.
/// `radius_px(i, depth)` gives point i's drawn radius in CSS pixels; 0 means hidden.
pub fn pick(
    at: Vec2,
    size: Vec2,
    view_proj: Mat4,
    positions: &[Vec3],
    radius_px: impl Fn(usize, f32) -> f32,
) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, p) in positions.iter().enumerate() {
        let clip = view_proj * p.extend(1.0);
        if clip.w <= 0.0 {
            continue; // behind the camera
        }
        let r = radius_px(i, clip.w);
        if r <= 0.0 {
            continue;
        }
        let ndc = clip.truncate() / clip.w;
        let screen = Vec2::new(
            f32::midpoint(ndc.x, 1.0) * size.x,
            (1.0 - ndc.y) * 0.5 * size.y,
        );
        let d = screen.distance(at);
        if d <= r.max(MIN_RADIUS) && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((i, d));
        }
    }
    best.map(|(i, _)| i)
}

/// The innermost sphere under `at`: of those whose drawn disc holds it, the smallest on screen
/// (a module inside its crate). `focal` is 1 / tan(half the vertical field of view).
pub fn pick_sphere(
    at: Vec2,
    size: Vec2,
    view_proj: Mat4,
    spheres: &[(Vec3, f32)],
    focal: f32,
) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, &(center, radius)) in spheres.iter().enumerate() {
        let clip = view_proj * center.extend(1.0);
        if clip.w <= 0.0 || radius <= 0.0 {
            continue;
        }
        let ndc = clip.truncate() / clip.w;
        let screen = Vec2::new(
            f32::midpoint(ndc.x, 1.0) * size.x,
            (1.0 - ndc.y) * 0.5 * size.y,
        );
        let r = radius * focal * (size.y / 2.0) / clip.w;
        if screen.distance(at) <= r && best.is_none_or(|(_, br)| r < br) {
            best = Some((i, r));
        }
    }
    best.map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cam() -> Mat4 {
        glam::camera::rh::proj::directx::perspective(1.0, 1.0, 0.1, 100.0)
            * glam::camera::rh::view::look_at_mat4(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y)
    }

    #[test]
    fn the_innermost_sphere_under_the_pointer_is_picked() {
        let focal = 1.0 / 0.5_f32.tan();
        let big = (Vec3::ZERO, 3.0);
        let small = (Vec3::new(0.5, 0.0, 0.0), 0.8);
        let far = (Vec3::new(8.0, 0.0, 0.0), 0.5);
        let spheres = [big, small, far];
        let at = |x| {
            pick_sphere(
                Vec2::new(x, 50.0),
                Vec2::splat(100.0),
                cam(),
                &spheres,
                focal,
            )
        };
        assert_eq!(at(52.0), Some(1), "inside both: the smaller one");
        assert_eq!(at(35.0), Some(0), "inside the big one only");
        assert_eq!(at(1.0), None, "outside every sphere");
    }

    #[test]
    fn picks_the_point_at_the_centre() {
        let pts = [Vec3::new(3.0, 0.0, 0.0), Vec3::ZERO];
        assert_eq!(
            pick(
                Vec2::splat(50.0),
                Vec2::splat(100.0),
                cam(),
                &pts,
                |_, _| 4.0
            ),
            Some(1)
        );
    }

    #[test]
    fn skips_hidden_points() {
        let pts = [Vec3::ZERO];
        assert_eq!(
            pick(
                Vec2::splat(50.0),
                Vec2::splat(100.0),
                cam(),
                &pts,
                |_, _| 0.0
            ),
            None
        );
    }
}
