# Architecture

The structure we build once and keep: crates, seams, and what each seam starts as and grows into.
Intent → `docs/README.md` · decisions → `docs/adr/` · words → `GLOSSARY.md` ·
what each seam must do → `docs/contracts/` · diagrams of complex topics → `docs/design/`.

## Overview in diagrams

Crates and who may depend on whom (arrow = "depends on"; `wb-core` knows no IO, egui or sockets):

```mermaid
flowchart TD
    app["wb-app (binary)"]
    net["wb-net"]
    core["wb-core"]
    eframe(["eframe"])
    egui(["egui"])
    app --> net
    app --> core
    app --> eframe
    app --> egui
    net --> core
```

Seams on the local side — layer 0 is the oracle, the arrow is the later fast impl (milestone):

```mermaid
flowchart LR
    si0["SpatialIndex: LinearIndex · M0"] -->|M4| si1["QuadTree"]
    sc0["StrokeCache: DecodeAll · M0"] -->|M4| sc1["ChunkWorkingSet"]
    sr0["StrokeRenderer: Naive · M0"] -->|M4| sr1["ChunkMesh → ChunkTexture"]
    cd0["StrokeCodec: Raw · M0"] -->|M4| cd1["Packed"]
    cam["Camera: one impl · M0"]
```

Seams on the network side:

```mermaid
flowchart LR
    tr["Transport: Udp + Sim · M1"]
    di0["Discovery: candidates + STUN punching · M1 · DHT feeds candidates from the driver"] -->|M4| di1["LAN / virtual network"]
    br0["Broadcast: SendToAll · M1"] -->|M4| br1["Gossip"]
    ae0["AntiEntropy: FullSet · M2"] -->|later| ae1["VersionVector"]
```

Runtime (ADR 0004): main and net threads in the app, plus the DHT threads (ADR 0025); the simulator drives the same sans-IO protocol code:

```mermaid
flowchart LR
    subgraph appproc["wb-app process"]
        subgraph mainth["main thread"]
            ui["egui UI + tools"] --> board["Board replica"]
            board --> rend["StrokeRenderer"]
        end
        subgraph netth["net thread"]
            drv["driver: Time + datagrams"] --> proto["sans-IO protocol: Discovery · Broadcast · AntiEntropy"]
        end
        mainth -->|"channel: local ops, commands"| netth
        netth -->|"channel: remote ops, events + repaint"| mainth
        drv <-->|"Transport: Udp"| udp[("UDP")]
        dht["DHT threads: mainline"] -->|"channel: candidates"| drv
    end
    subgraph simproc["wb-sim binary (wb-net)"]
        sdrv["sim driver: simulated Time"] --> sproto["same sans-IO protocol"]
        sdrv <-->|"Transport: Sim, seeded chaos"| chaos[("loss · latency · partitions")]
    end
```

Channel details (bounded or not, crate) are open in G21. The app net thread uses a blocking UDP
socket with timers.

## Layering rule

Every performance idea lives behind a **seam** (a trait). Layer 0 is the simplest correct impl and
stays forever as the **oracle**: the fast impl must pass the same test suite and agree with the
oracle on random inputs, and must show a benchmark win, before it replaces layer 0 as the default.

## Crates (Cargo workspace, `crates/<name>`)

| Crate | Contents | Depends on |
|---|---|---|
| `wb-core` | geometry, `Stroke`, `StrokeId`, `Board` seam (CRDT state + `merge`), ops, pages, `StrokeCodec` + postcard format, `SpatialIndex` seam | serde, postcard, xxhash-rust |
| `wb-net` | sans-IO protocol, `Transport` seam + UDP/Sim drivers, discovery, broadcast, anti-entropy, deterministic sim module and `wb-sim` binary | core |
| `wb-app` | the binary: eframe window, tools, menu + connect modal, threads, save/load, join flow, `Camera`, `StrokeRenderer` + `StrokeCache`, chunks, PDF export | core, net, eframe/egui |

Rule: `wb-core` knows nothing about IO, egui or sockets; sockets and threads live only in `wb-net` drivers and `wb-app`.

## Seams

| Seam | Layer 0 (simple) | Later layer (fast) |
|---|---|---|
| `Board` | `TombstoneBoard`: ordered sets of strokes and tombstones | — |
| `SpatialIndex` | `Vec` + linear scan | quadtree |
| `StrokeRenderer` | egui painter, every visible stroke, every frame | cached chunk meshes → chunk textures (wgpu) |
| `StrokeCache` | `DecodeAll`: unpack once on arrival, keep all | `ChunkWorkingSet`: resident chunks + margin, byte budget |
| `Camera` | affine matrix (pan + zoom) from day 1 | — (already cheap) |
| `StrokeCodec` | origin + plain `i32` offsets via postcard | point reduction → delta → bit packing |
| `Transport` | real UDP socket · `SimTransport` (seeded loss/latency/partition) | — |
| `Discovery` | candidates (manual ip:port; DHT from the driver) + STUN punching (M1) | LAN / virtual network (M4) |
| `Broadcast` | send to every known peer | gossip (fanout k) |
| `AntiEntropy` | exchange full id sets | compact digests (version vectors / hashes) |

## Runtime (ADR 0004)

The protocol is a pure state machine: `handle(event, now) → Vec<Outgoing>`. The net thread and the
deterministic sim module are two drivers of the same code.

## Data flow: drawing a stroke

1. Pointer down/move → points collected in screen space → camera inverse → `i64` world space.
2. Pointer up → codec simplifies and encodes into one or more pieces (ADR 0005, 0014; stays
   encoded in memory). For each piece: compute its bounding box → `Board::draw` assigns the `OpId`
   (ADR 0009) and Lamport timestamp (ADR 0008) → insert into the spatial index → broadcast.
3. Anti-entropy heals anything broadcast lost (ADR 0003).
4. Remote op → net → main → `Board::apply` → index + invalidate the affected area.
5. Drawing and erasing read unpacked points only through `StrokeCache` (ADR 0018).

The same flow as a sequence diagram → `docs/design/stroke-lifecycle.md`.
