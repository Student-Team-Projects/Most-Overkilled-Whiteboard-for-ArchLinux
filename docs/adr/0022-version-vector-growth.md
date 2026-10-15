# 0022 — Version vector grows by one author per run

Status: accepted · 2026-10-15 · closes G09

- **Decision:** accept one version-vector author entry per run in v1. The later `VersionVector`
  summary pages like every other summary. If it does not fit one datagram, or two vectors differ,
  compare range pages of stroke ids and tombstone targets: redundant erases make vectors differ
  while the canonical state is equal. During join classification this is comparison-only; it does
  not push or apply operations. Measure its size in the simulator and revisit as needed.
- **Why:** keep the initial representation simple while measuring realistic peer growth.
