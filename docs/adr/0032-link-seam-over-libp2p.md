# 0032 — Network model: Link seam over libp2p

Status: accepted · 2026-11-16 · amends ADR 0003, 0004, 0006, 0019, 0021, 0025, 0031

- **Decision:**
  1. **Seam:** `Link` replaces `Transport`. `send(peer, bytes)` is best-effort and accepts at most
     1200 bytes. Events are `Received(peer, message)`, `PeerUp(peer)`, and `PeerDown(peer)`; the
     driver feeds them to the sans-IO core. There is no delivery, order, or duplicate guarantee.
     `Broadcast` and `AntiEntropy` remain protocol contracts; their wire unit is a message.
  2. **Framing:** messages use one libp2p stream with a u16 big-endian length prefix. The prefix
     is outside the limit; the ADR 0021 envelope itself is at most 1200 B. A length over 1200
     resets the stream because framing cannot resynchronize. A validly sized invalid envelope is
     dropped and counted by the T26 reject counters (`WrongMagic`, `UnknownVersion`,
     `Undecodable`, `ForeignBoard`); the stream stays up. `ForeignBoard` filters envelopes for a
     different board; it is not admission control.
  3. **Identity:** each run creates an ephemeral libp2p keypair. `PeerId(u64)` is the first eight
     bytes, big-endian, of SHA-256(libp2p `PeerId` bytes), retaining ADR 0006's fresh-per-run identity.
  4. **Lifecycle:** the adapter reconnects with backoff from about 1 s to 30 s and forgets a peer
     after about 2 min without contact. The protocol sees only `PeerUp`/`PeerDown`; the adapter
     does not replay messages. `AntiEntropy` heals after `PeerUp`.
  5. **Testing:** `Sim` implements `Link` with seeded loss, duplication, reordering, partitions,
     and up/down events. Test the real libp2p adapter only in the ADR 0017 devnet integration:
     direct, DCUtR, and relay paths.
  6. **Bootstrap and relay:** the homelab Kademlia bootstrap and Circuit Relay v2 address live
     only in a user config file (TOML in the user config directory, e.g. `bootstrap = ["<multiaddr>"]`); it is never committed to the repo. The devnet supplies its own config file. With no user config file, use Amino bootstrap and manual join only; there is no relay. Connect order: direct dial; if AutoNAT says private, reserve a relay; attempt DCUtR upgrade; stay on relay if upgrade fails.
  7. **Rendezvous:** the Kademlia provider key is
     `SHA-256("wb-rendezvous-v1" || session_code)`. Never publish or log the code. The code is
     needed to find and address the right board; it is not admission control. Admission checking is
     deferred, so anyone who knows the code can join. DHT observers see only the hash and peer
     addresses.
  8. **Manual join:** `join <code> [ip:port | multiaddr]`. Translate `ip:port` to
     `/ip4/<ip>/udp/<port>/quic-v1`. The code remains required to find and address the intended
     board. `ForeignBoard` drops and counts envelopes for another board; admission by code is
     deferred.
  9. **Local instances — Settles G14 (#14):** by default each instance listens on
     `/ip4/0.0.0.0/udp/0/quic-v1`, letting the OS choose an ephemeral port. `--port <n>` overrides
     the port. Each run has a fresh libp2p key, so many instances on one machine work without
     coordination. To manually join a local instance, use `join <code> 127.0.0.1:<port>`.
  10. **Runtime:** GUI has its own thread. The net thread runs a Tokio runtime (`current_thread`
     is sufficient) that owns the swarm and drives timers and ticks, including `AntiEntropy`
     cadence and `RejectLog::tick`. Tokio supplies `now` to the sans-IO core; the core never
     awaits, preserving deterministic simulation. GUI and net thread use bounded
     `tokio::sync::mpsc` channels in both directions. The UI uses `try_send` and never blocks; a
     full channel means drop and log. It uses `try_recv` each frame. Net events wake the UI with
     egui `request_repaint`.
  11. **M1 network done:** two peers on different networks draw together using a session code,
      directly or through the homelab relay; manual LAN join works; links use Noise encryption;
      the ADR 0017 devnet NAT scenario is the proof.
- **Consequences:** ADR 0003's best-effort message semantics remain visible to protocols on top of
  reliable streams. ADR 0021 keeps its envelope and rejection accounting; its 1200 B limit applies
  to the envelope. ADR 0025's discovery flow and ADR 0031's initial network choices are replaced
  as detailed above. ADR 0006 retains fresh per-run peer identity.

## Open

- Session-code admission is not cryptographic authorization; anyone who knows the code can join.
