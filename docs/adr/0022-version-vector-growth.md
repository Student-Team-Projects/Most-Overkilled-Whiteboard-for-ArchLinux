# 0022 — Version vector grows by one author per run

Status: accepted · 2026-10-15 · closes G09

- **Decision:** accept one version-vector author entry per run in v1. Measure its size in the
  simulator; revisit if a digest no longer fits in one datagram.
- **Why:** keep the initial representation simple while measuring realistic peer growth.
