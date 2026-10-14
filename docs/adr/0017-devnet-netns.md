# 0017 — Dev network: Linux namespaces on one laptop

Status: accepted · 2026-10-14

- **Context:** the other team's virtual network will be usable late in the semester; we need several
  "devices" with real networking from M1.
- **Decision:** `scripts/devnet.sh` creates N network namespaces on one bridge — each instance gets its
  own IP, broadcast works, `tc netem` adds loss / latency / partitions. Plain `127.0.0.1` ports stay
  for quick runs and CI (no root).
- **Constraints:** Arch Linux, only `iproute2` (in base), needs `sudo`; light and fully reversible —
  `devnet.sh down` removes everything, nothing persists across reboot.
