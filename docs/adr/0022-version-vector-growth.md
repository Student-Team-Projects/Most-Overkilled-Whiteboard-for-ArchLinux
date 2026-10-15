# 0022 — Version vector grows by one author per run

Status: accepted · 2026-10-15 · closes G09

- **Decision:** accept one version-vector author entry per run in v1. The later `VersionVector`
  summary pages like every other summary; if it does not fit one datagram, fall back to range pages
  comparing stroke ids and tombstone targets. During join classification, this fallback is
  comparison-only; it does not push or apply operations. Measure its size in the simulator and
  revisit the representation as needed.
- **Why:** keep the initial representation simple while measuring realistic peer growth.
