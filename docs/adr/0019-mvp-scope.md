# 0019 — Paged MVP before performance and resilience

Status: accepted · 2026-10-15 · stakeholder re-plan

- **Decision:**
  - MVP for the grade is a vertical stack of A4 pages, PDF export, internet connection through STUN
    and a DHT rendezvous (ADR 0025) with manual `ip:port` join, and both front ends: CLI (ADR 0016, preferred) and
    an egui start menu + connect modal (egui already a dependency, ADR 0007).
  - MVP is functional by M1 with layer 0 everywhere: `LinearIndex`, `Raw`, `SendToAll`, and `Naive`.
    Coordinates remain `i64` world units (ADR 0014); drawing is limited to pages, with an add-page
    action.
  - Kacper owns networking; Mateusz owns core, pages, save/load, PDF export, and sim; Filip owns
    the GUI, including the start menu and connect modal.
- **Consequences:** quadtree, infinite canvas, chunks, `Packed`, `ChunkWorkingSet`, LAN discovery,
  virtual network, gossip, and large-scale sim move to M4. The cut line moves to M0–M3 as the
  hardened MVP. ADR 0007 still holds: egui is the only UI library. PDF export lives in `wb-app`.
