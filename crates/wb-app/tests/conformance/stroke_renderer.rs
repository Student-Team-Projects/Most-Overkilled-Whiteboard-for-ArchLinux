//! `docs/contracts/stroke-renderer.md`, run on every renderer.

use wb_app::stroke_renderer::{Naive, StrokeRenderer, to_color32};
use wb_core::WorldPos;

use crate::support::config::{BLUE, FAR_OFF_SCREEN, RED};
use crate::support::stroke_renderer::Scene;

/// `None` if a test stroke does not fit one piece.
fn paints_only_strokes_in_view_bottom_first(mut renderer: impl StrokeRenderer) -> Option<()> {
    let mut scene = Scene::new();
    scene.add(&[WorldPos::new(0, 0), WorldPos::new(160, 0)], RED)?;
    scene.add(&[FAR_OFF_SCREEN], RED)?;
    scene.add(&[WorldPos::new(0, 160)], BLUE)?;
    assert_eq!(
        scene.painted_dots(&mut renderer),
        [to_color32(RED), to_color32(RED), to_color32(BLUE)]
    );
    Some(())
}

/// `None` if the test stroke does not fit one piece.
fn invalidate_then_paint_shows_add_and_erase(mut renderer: impl StrokeRenderer) -> Option<()> {
    let mut scene = Scene::new();
    assert!(scene.painted_dots(&mut renderer).is_empty());
    let (id, bbox) = scene.add(&[WorldPos::new(0, 0)], BLUE)?;
    renderer.invalidate(bbox);
    assert_eq!(scene.painted_dots(&mut renderer), [to_color32(BLUE)]);
    scene.erase(id);
    renderer.invalidate(bbox);
    assert!(scene.painted_dots(&mut renderer).is_empty());
    Some(())
}

mod naive {
    use super::*;

    #[test]
    fn paints_only_strokes_in_view_bottom_first() {
        super::paints_only_strokes_in_view_bottom_first(Naive).unwrap();
    }

    #[test]
    fn invalidate_then_paint_shows_add_and_erase() {
        super::invalidate_then_paint_shows_add_and_erase(Naive).unwrap();
    }
}
