Dated notes: weekly syncs and debt payouts; newest first. Status lives on the board.

Sync-entry format — heading `## YYYY-MM-DD — sync`, then one line each; link issues, never list tasks:

- **Present:** who attended.
- **Landed:** merged PRs since the last sync.
- **Blocked:** item → what unblocks it, who.
- **Decided:** one line each, with the ADR or issue link.
- **Next week:** person → issues they own.

## 2026-11-12 — debt payout + M1 re-plan

- **Context:** M0 presented 11.11; M1 needs ~3 more weeks.
- **Decided:** M1 → Tue 01.12, pulls in `Infinite` mode (T48 #116, Filip), `Packed` codec (G17 #17, T23 #47) and the in-process sim (T20 #44) — both Mateusz; Mateusz also takes T21 #45, G14 #14; Kacper keeps the net core incl. T18 #42. M2 + M3 merged into "M2 Bulletproof + stakeholder demo" → 21.12; range hashes (G27 #61) → M4.
- **Board:** 14 closed issues moved to Done; spikes T28 #52 (PR #105) and T29 #53 closed as superseded by T11 and T14; milestones M0 and M3 closed.
- **Docs:** roadmap re-planned; camera rows in `architecture.md` match the code; spike line in `team.md` no longer points at T08.
- **ADRs:** 0026, 0027, 0029 accepted (all 3; missed during the M0 rush). Drafts left only on closed spike branches recovered as proposed: 0028 opaque colours (#104), 0030 zoom scale + opening view (#105) — to agree at the next sync.
- **Code:** `wb-core` postcard round-trip test moved from `src/types.rs` to `tests/types.rs` (R2). Evidence: fmt, clippy, `cargo test --workspace` green.

## 2026-10-20 — sync

- **Present:** Kacper, Mateusz, Filip.
- **Landed:** #64 #65 #68 #77 #80 #81 #83 #85 #89 #91 #92.
- **Blocked:** nothing beyond G24 → T08, unblocked today.
- **Decided:** all 10 contracts agreed (G24 #24), each owner reads their share by Fri 23.10 (T45 #93); ADR 0025, 0024 and 0006 confirmed; `BoardId` = 8 bytes.
- **Next week:** Kacper → #32 #50 #70 #71 #78 #86 · Mateusz → #66 #33 #36 #87 · Filip → #51 #52 #53 #67 #35.

## 2026-10-15 — debt payout

- Aligned seam milestones and discovery scope with ADR 0019; clarified spatial coordinates and renderer invalidation wording.
- Documented MVP join options and LAN discovery dependency.
- Corrected the two specified README items and reduced its location map to three rows.
- Consolidated board guidance, seam implementation sources, and glossary references.
- Updated team guidance for CI merge queue, benchmark method, journal, and spike stand-in types.
- Trimmed domain guidance to the repository's single context and noted skill vendoring.
- Updated devnet script references and removed roadmap status fields.
