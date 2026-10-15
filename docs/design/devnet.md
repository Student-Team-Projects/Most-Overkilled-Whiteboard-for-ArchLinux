# Design: dev network on one laptop

Topology and lifecycle of the namespace dev network (ADR 0017). Script: T24 on the board.
Overview → `docs/architecture.md` · discovery contract → `docs/contracts/discovery.md`.

Namespaces are `wb0` through `wb<N-1>`, attached to bridge `wbbr0`. Namespace `wb<i>` uses
`10.77.0.<i+10>/24` and has a default route via `eth0`. Netem is applied on both sides of each
veth, so chaos and partition affect both directions. Values apply in each direction: `chaos i delay
50ms` adds about 100 ms to a round trip with i, and `loss 20%` fails about 36% of
request-and-reply exchanges. `heal` also flushes stale ARP entries so the peer is reachable at once.
Self-check: `scripts/devnet-test.sh`.

```mermaid
flowchart TB
    subgraph host["laptop · root namespace"]
        br["bridge wbbr0"]
    end
    subgraph n0["netns wb0 · 10.77.0.10"]
        a0["wb instance"] --- q0["netem: loss · latency"]
    end
    subgraph n1["netns wb1 · 10.77.0.11"]
        a1["wb instance"] --- q1["netem"]
    end
    subgraph nN["netns wbN-1 · 10.77.0.N+9"]
        aN["wb instance"] --- qN["netem"]
    end
    q0 ---|veth| br
    q1 ---|veth| br
    qN ---|veth| br
```

```mermaid
flowchart LR
    up["devnet.sh up N"] --> run["devnet.sh run i cmd"]
    run --> chaos["devnet.sh chaos i netem…"]
    chaos --> clear["devnet.sh chaos i clear"]
    run --> partition["devnet.sh partition i"]
    partition --> heal["devnet.sh heal i"]
    clear --> down["devnet.sh down"]
    heal --> down
    status["devnet.sh status"]
```

## Reading guide

| Rule | Source |
|---|---|
| each instance has its own IP; broadcast works on the bridge | ADR 0017 |
| only `iproute2` (Arch base) + `sudo`; nothing persists | ADR 0017 |
| plain `127.0.0.1` ports stay for quick runs and CI | ADR 0017 |
| port range for the plain mode | open (G14) |
| real virtual network of the other team | open (G11) |
