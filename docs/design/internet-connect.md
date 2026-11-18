# Design: connecting peers over the internet

NAT behavior and the libp2p connection path. Decision → ADR 0032 · link contract →
`docs/contracts/link.md` · join flow → `join-flow.md`.

## Connection path

```mermaid
sequenceDiagram
    participant A as Peer A
    participant K as Amino Kademlia DHT
    participant H as Homelab bootstrap and relay
    participant B as Peer B
    Note over A,B: key = SHA-256("wb-rendezvous-v1" || session_code); code is never published
    A->>K: provide under hashed session key
    B->>K: find providers under hashed session key
    B->>A: direct QUIC dial
    Note over A,B: if AutoNAT says private: reserve Circuit Relay v2 path
    A->>B: DCUtR upgrade attempt
    B-->>A: PeerUp over direct path, or remain on relay
    A-->>B: Noise-encrypted protocol messages
```

Connection order: direct dial; if AutoNAT reports private, reserve a relay; try DCUtR; remain on
relay if the upgrade fails. Homelab bootstrap and Circuit Relay v2 addresses live only in a user TOML config file in the
user config directory (for example, `bootstrap = ["<multiaddr>"]`); it is never committed to the
repo. The devnet supplies its own config file. With no user config file, use Amino bootstrap and
manual join only; relay is unavailable. Manual join is `join <code> [ip:port | multiaddr]`; translate `ip:port` to
`/ip4/<ip>/udp/<port>/quic-v1`.

## NAT background

This section is background on common NAT behavior, not a guarantee for every router.

### NAT types and direct connectivity

NAT creates outbound mappings and filters inbound traffic. A direct dial fails when the remote
address or port is not reachable through that mapping. AutoNAT estimates public reachability;
DCUtR coordinates simultaneous direct dials (hole punching); Circuit Relay v2 forwards traffic
when direct paths fail.

| NAT type | Mapping and inbound filtering | Direct-path effect |
|---|---|---|
| Full cone | One public mapping accepts inbound traffic from any address | Direct dial can work once mapped |
| Restricted cone | Inbound traffic allowed only from previously contacted IPs | Both peers must exchange traffic first |
| Port-restricted cone | Inbound traffic allowed only from previously contacted IP and port | DCUtR coordinates matching traffic |
| Symmetric | Mapping varies by destination; inbound filtering is restrictive | Peers may see unusable public endpoints; relay often required |

### Scenario matrix

| Scenario | Expected path | How libp2p handles it |
|---|---|---|
| Both public | direct | QUIC dial with Noise encryption |
| One behind cone NAT | DCUtR | AutoNAT identifies private reachability; DCUtR coordinates the direct attempt |
| Both cone NAT | DCUtR | DCUtR coordinates simultaneous outbound traffic through mappings |
| One symmetric NAT | relay | AutoNAT prompts relay reservation; try DCUtR, remain relayed if it fails |
| Both symmetric NAT | relay | DCUtR may fail; Circuit Relay v2 carries the connection |
| UDP blocked | fails | QUIC-only transport (ADR 0031); Circuit Relay v2 also uses QUIC |

Probe-lab DCUtR success rates: **82.9%** for port-restricted cone NAT and **39.7%** for symmetric
NAT. These measurements describe the probe lab, not a guarantee for a deployment.

The session code is needed to find and address the right board; it is not admission control.
`ForeignBoard` filters, drops, and counts envelopes for another board. Admission by code is
deferred, so anyone who knows the code can join. DHT observers see only the provider key hash and
peer addresses.

## Reading guide

| Rule | Source |
|---|---|
| Link size, framing, lifecycle | ADR 0032 · `docs/contracts/link.md` |
| PeerId mapping, GUI/net channels, and bootstrap config | ADR 0032 · `docs/architecture.md` |
| Bootstrap, relay and connect order | ADR 0032 · `docs/contracts/discovery.md` |
| `join <code> [ip:port | multiaddr]` | ADR 0032 |
| NAT behavior | background above |
