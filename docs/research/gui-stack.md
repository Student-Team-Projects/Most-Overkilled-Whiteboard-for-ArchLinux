# Research: Rust GUI / rendering stack for the whiteboard
Purpose: compare GUI/render stacks for the one-window infinite-canvas client (Arch, Wayland + X11). Project intent → `docs/README.md`; the chosen stack becomes an ADR in `docs/adr/`.

Researched 2026-10-13. Every claim links its source; **[measured]** = built/run here; **[unverified]** = not confirmed in a primary source.

## 1. Versions and maintenance (crates.io API, 2026-10-13)

| Crate | Latest stable | Last publish | Note |
|---|---|---|---|
| eframe / egui / egui-wgpu | 0.36.2 | 2026-09-08 | 0.33 → 0.36 shipped 2025-10-09 → 2026-08-05 ([eframe CHANGELOG](https://github.com/emilk/egui/blob/main/crates/eframe/CHANGELOG.md)); sponsored by Rerun ([README](https://github.com/emilk/egui)) |
| wgpu | 30.0.1 | 2026-08-21 | v30.0.0 2026-07-01, v29 earlier ([CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)) — breaking major ~quarterly |
| winit | 0.30.13 | 2026 | 0.31.0-beta.3 out, not stable ([releases](https://github.com/rust-windowing/winit/releases)) |
| softbuffer | 0.4.8 | 2025-12-13 | Wayland + X11 are Tier 1 ([README](https://github.com/rust-windowing/softbuffer)) |
| tiny-skia | 0.12.0 | 2026-02-02 | now under linebender ([crates.io](https://crates.io/crates/tiny-skia)) |
| vello / vello_cpu | 0.11.0 / 0.3.0 | 2026-10-02 | compute renderer "remains an experimental implementation" ([README](https://github.com/linebender/vello)) |
| femtovg | 0.27.0 | 2026-08-31 | nanovg port, used by Slint ([README](https://github.com/femtovg/femtovg)) |
| macroquad / miniquad | 0.4.16 / 0.4.11 | 2026-07 / 2026-06 | [crates.io](https://crates.io/crates/macroquad) |
| iced | 0.14.0 | 2025-12-07 | "currently experimental software" ([README](https://github.com/iced-rs/iced)) |

Source for versions/dates: `https://crates.io/api/v1/crates/<name>`.

## 2. Size and idle cost [measured]

Hello-world that draws one line. Setup: rustc/cargo 1.98.1, `--release`, `strip = true`, no LTO; Arch, kernel 7.2, Wayland session (X11 = same binary with `WAYLAND_DISPLAY` unset → XWayland). Idle = CPU ticks over 5 s after 3 s warm-up, no input.

| Stack | Binary | RSS Wayland / X11 | Idle CPU | Redraw model |
|---|---|---|---|---|
| eframe, `glow` (no accesskit) | 9.1 MB | 59 / 60 MB | 0 % | on demand ([README](https://github.com/emilk/egui): "only repaints when there is interaction … or an animation") |
| eframe, `wgpu` (default renderer since 0.34) | 14.4 MB | 70 / 65 MB | 0 % | on demand |
| winit + softbuffer + tiny-skia | 3.4 MB | 19 / 11 MB | 0 % | you decide (`request_redraw`) |
| macroquad | 1.1 MB | 56 / 55 MB | ~2.4 % of a core | standard `loop { … next_frame().await }` draws every frame; ran on XWayland (see §3) |
| raw winit + wgpu | not measured | — | — | you decide; expect close to the eframe-wgpu number minus egui [unverified] |

Release with LTO / `opt-level="z"` would shrink all of these; not measured.

## 3. Wayland / X11 on Arch

| Stack | Status | Known issues (open, primary source) |
|---|---|---|
| eframe (winit 0.30) | both, `wayland` + `x11` features on by default ([docs.rs](https://docs.rs/eframe/latest/eframe/)) | ⚠️ idle loop spin on Wayland after repaint burst, eframe 0.35 ([#8523](https://github.com/emilk/egui/issues/8523)) — not reproduced in our hello-world · ⚠️ pinch-zoom gives no `zoom_delta` on Wayland ([#8518](https://github.com/emilk/egui/issues/8518)) · keyboard ignored on Hyprland ([#8249](https://github.com/emilk/egui/issues/8249)) · Wayland DnD ([#1563](https://github.com/emilk/egui/issues/1563)) |
| winit (any raw stack) | both; recent Wayland/X11 fixes in 0.30.10–0.30.13 ([releases](https://github.com/rust-windowing/winit/releases)) | none blocking found |
| softbuffer | Wayland + X11 Tier 1 ([README](https://github.com/rust-windowing/softbuffer)) | none found |
| miniquad / macroquad | default `LinuxBackend::X11Only`; "The Wayland implementation is currently unstable" ([docs.rs](https://docs.rs/miniquad/latest/miniquad/conf/enum.LinuxBackend.html)) | ⚠️ native Wayland not production-ready |

## 4. Drawing 100k+ strokes, tiles, camera

| Stack | Thick polylines | Camera (pan/zoom) | Offscreen tiles → reuse |
|---|---|---|---|
| egui (naive) | `Shape::Path` / `line_segment`; epaint CPU-tessellates shapes into meshes; the documented loop calls `tessellate` every frame ([Context](https://docs.rs/egui/latest/egui/struct.Context.html)) → 100k visible strokes re-tessellated per frame ⚠️ | `TSTransform` = translate + scale, no rotation ([emath](https://docs.rs/emath/latest/emath/struct.TSTransform.html)); `Shape::transform` ([epaint](https://docs.rs/epaint/latest/epaint/enum.Shape.html)); `egui::Scene` pan/zoom container ([docs](https://docs.rs/egui/latest/egui/containers/struct.Scene.html)) | pre-built `Shape::Mesh(Arc<Mesh>)` per chunk ([epaint](https://docs.rs/epaint/latest/epaint/enum.Shape.html)) — skips tessellation, still re-uploaded each frame [unverified] |
| egui + wgpu callback | own shader/pipeline via `egui_wgpu::CallbackTrait` (`prepare` / `finish_prepare` / `paint`) ([docs](https://docs.rs/egui-wgpu/latest/egui_wgpu/trait.CallbackTrait.html)) | uniform matrix in your shader | render chunk into a `wgpu::Texture`, `register_native_texture` → `TextureId`, draw with `painter.image` ([Renderer](https://docs.rs/egui-wgpu/latest/egui_wgpu/struct.Renderer.html), [Painter](https://docs.rs/egui/latest/egui/struct.Painter.html)); texture must be `Rgba8Unorm` |
| egui + glow callback | raw OpenGL in `egui_glow::CallbackFn` ([example](https://github.com/emilk/egui/blob/main/examples/custom_3d_glow/src/main.rs)) | your matrix | FBO textures by hand, raw GL [unverified effort] |
| raw winit + wgpu | no wide lines: `PrimitiveState` has no line-width field ([wgpu-types](https://github.com/gfx-rs/wgpu/blob/trunk/wgpu-types/src/render.rs)) → tessellate with lyon `StrokeTessellator` ([docs](https://docs.rs/lyon_tessellation/latest/lyon_tessellation/struct.StrokeTessellator.html)) or expand quads in shader | uniform matrix | `render_to_texture` example ([wgpu examples](https://github.com/gfx-rs/wgpu/tree/trunk/examples/features/src)) |
| softbuffer + tiny-skia | `Pixmap::stroke_path`, full CPU AA raster | `Transform` per draw call; you manage transforms yourself ([docs](https://docs.rs/tiny-skia/latest/tiny_skia/)) | tile = cached `Pixmap`, blit with `draw_pixmap`; 20–100 % slower than Skia on x86-64 ([README](https://github.com/linebender/tiny-skia)) ⚠️ zoom-out over 100k strokes is pure CPU |
| iced canvas | `Path` + `Stroke`, `canvas::Cache` stores generated geometry ([docs](https://docs.rs/iced/latest/iced/widget/canvas/index.html)); reactive rendering since 0.14 ([CHANGELOG](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md)) | not documented on canvas page | geometry cache, not texture cache |

## 5. Team cost: first line, built-in UI, upgrade path

| Stack | First line on screen | Toolbar / color picker | Naive → cached-chunk path |
|---|---|---|---|
| eframe | ~5 lines (`run_ui_native` + `painter().line_segment`) [measured]; ready freehand demo `painting.rs` ([source](https://github.com/emilk/egui/blob/main/crates/egui_demo_lib/src/demo/painting.rs)) | yes, incl. `color_edit_button_srgba` ([docs](https://docs.rs/egui/latest/egui/widgets/color_picker/fn.color_edit_button_srgba.html)) | naive Painter → `Arc<Mesh>` per chunk → wgpu callback + textures; UI code unchanged |
| raw winit + wgpu | surface, pipeline, WGSL, tessellation first [unverified time] | none — build or add egui later | full control, but naive step already costs a renderer |
| softbuffer + tiny-skia | ~40 lines [measured] | none | Pixmap tiles, easy to test; ceiling = CPU |
| macroquad | ~3 lines [measured] | immediate UI exists, not evaluated | Wayland risk (§3) |

## 6. Recommendation

| Option | Trade-off | Why |
|---|---|---|
| **eframe/egui, `wgpu` backend** (recommended) | 14.4 MB, ~70 MB RSS; Wayland issues #8523/#8518 to watch | first line in minutes, toolbar + color picker free; naive Painter → per-chunk wgpu textures via `CallbackTrait` + `register_native_texture` without touching UI |
| eframe/egui, `glow` backend | 9.1 MB; chunk cache = raw OpenGL FBOs, less typed | same app code; pick if size matters more than ergonomics |
| winit + softbuffer + tiny-skia | 3.4 MB, ~15 MB RSS; no UI, CPU-bound zoom-out | smallest and simplest to reason about; tiles = Pixmaps; good fallback/test renderer |
| raw winit + wgpu (+ lyon) | most code before first line; no UI | max performance control; only if egui gets in the way |

Not shortlisted: macroquad (Wayland unstable, continuous redraw), iced (self-described experimental), vello (compute renderer experimental), femtovg (fine, but no UI and smaller ecosystem than egui).
