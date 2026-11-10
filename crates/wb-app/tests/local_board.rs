//! `LocalBoard`: strokes in through the codec, out through the eraser.

use wb_app::eraser_utils::world_points;
use wb_app::local_board::{LocalBoard, SimpleLocalBoard};
use wb_core::{Aabb, Board, Raw, Stroke, StrokeCodec, WorldPos};

use crate::support::config::{AUTHOR, LONG_STROKE_POINTS, LONG_STROKE_STEP, REACH, RED, WIDTH};

fn points_of(stroke: &Stroke) -> Option<Vec<WorldPos>> {
    Some(world_points(&Raw.decode(&stroke.points).ok()?).collect())
}

#[test]
fn a_stroke_is_stored_once_with_bounds_covering_its_width() {
    let mut board = SimpleLocalBoard::new(AUTHOR);
    let points = [WorldPos::new(0, 0), WorldPos::new(100, -50)];
    board.add_stroke(&points, RED, WIDTH).unwrap();
    let strokes: Vec<&Stroke> = board.board().visible().collect();
    let [stroke] = strokes.as_slice() else {
        panic!("expected one stroke, got {}", strokes.len());
    };
    assert_eq!(points_of(stroke).unwrap(), points);
    let half = i64::from(WIDTH / 2);
    assert_eq!(
        stroke.bbox,
        Aabb::new(
            WorldPos::new(-half, -50 - half),
            WorldPos::new(100 + half, half)
        )
    );
    assert_eq!((stroke.color, stroke.width), (RED, WIDTH));
}

#[test]
fn a_long_stroke_becomes_pieces_that_meet_end_to_start() {
    let mut board = SimpleLocalBoard::new(AUTHOR);
    let points: Vec<WorldPos> = (0..LONG_STROKE_POINTS)
        .map(|i| WorldPos::new(i * LONG_STROKE_STEP, 0))
        .collect();
    board.add_stroke(&points, RED, WIDTH).unwrap();
    let pieces: Vec<Vec<WorldPos>> = board
        .board()
        .visible()
        .map(|s| points_of(s).unwrap())
        .collect();
    assert!(
        pieces.len() > 1,
        "the stroke fits one piece; make it longer"
    );
    for pair in pieces.windows(2) {
        assert_eq!(pair[0].last(), pair[1].first());
    }
    let mut joined = pieces[0].clone();
    for piece in &pieces[1..] {
        joined.extend_from_slice(&piece[1..]);
    }
    assert_eq!(joined, points);
}

#[test]
fn no_points_add_nothing() {
    let mut board = SimpleLocalBoard::new(AUTHOR);
    board.add_stroke(&[], RED, WIDTH).unwrap();
    assert_eq!(board.board().visible().count(), 0);
}

#[test]
fn the_eraser_removes_only_the_strokes_it_touches() {
    let mut board = SimpleLocalBoard::new(AUTHOR);
    let touched = [WorldPos::new(0, 0), WorldPos::new(1_000, 0)];
    let untouched = [WorldPos::new(0, 1_000), WorldPos::new(1_000, 1_000)];
    board.add_stroke(&touched, RED, WIDTH).unwrap();
    board.add_stroke(&untouched, RED, WIDTH).unwrap();
    board.erase_touched(WorldPos::new(500, REACH), REACH);
    let left: Vec<Vec<WorldPos>> = board
        .board()
        .visible()
        .map(|s| points_of(s).unwrap())
        .collect();
    assert_eq!(left, [untouched.to_vec()]);
}

#[test]
fn the_eraser_misses_a_stroke_just_out_of_reach() {
    let mut board = SimpleLocalBoard::new(AUTHOR);
    board
        .add_stroke(&[WorldPos::new(0, 0)], RED, WIDTH)
        .unwrap();
    let just_out = REACH + i64::from(WIDTH / 2) + 1;
    board.erase_touched(WorldPos::new(just_out, 0), REACH);
    assert_eq!(board.board().visible().count(), 1);
    board.erase_touched(WorldPos::new(just_out - 1, 0), REACH);
    assert_eq!(board.board().visible().count(), 0);
}
