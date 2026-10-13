# 0007 — GUI: eframe/egui with the wgpu renderer

Status: accepted · 2026-10-13 · research: `docs/research/gui-stack.md`

- **Context:** one lightweight window on Arch (Wayland + X11), toolbar + colour picker, path to
  chunk rendering for 100k+ strokes.
- **Decision:** eframe/egui with the wgpu backend; versions pinned for the semester.
- **Why:** first stroke in minutes, UI widgets built in, and the renderer can go painter calls →
  cached chunk meshes → chunk textures without touching the UI. Cost: ~14 MB binary, ~70 MB RAM.
  Known: pinch-zoom bug on Wayland (#8518) → Ctrl+scroll fallback.
