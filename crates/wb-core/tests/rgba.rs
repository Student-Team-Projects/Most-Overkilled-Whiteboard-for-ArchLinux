//! `Rgba` to egui's `Color32` and back.

use ecolor::Color32;
use wb_core::Rgba;

#[test]
fn an_opaque_colour_keeps_its_channels() {
    assert_eq!(
        Color32::from(Rgba([220, 40, 40, 255])),
        Color32::from_rgb(220, 40, 40)
    );
}

#[test]
fn a_translucent_colour_is_taken_as_unmultiplied() {
    assert_eq!(
        Color32::from(Rgba([200, 100, 50, 128])),
        Color32::from_rgba_unmultiplied(200, 100, 50, 128)
    );
}

#[test]
fn a_colour_comes_back_unchanged() {
    let colour = Rgba([30, 90, 220, 255]);
    assert_eq!(Rgba::from(Color32::from(colour)), colour);
}
