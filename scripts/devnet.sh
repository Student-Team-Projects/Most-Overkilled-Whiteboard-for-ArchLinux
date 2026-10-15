#!/usr/bin/env bash
set -euo pipefail

# Show the supported commands and their arguments.
usage() {
  cat <<'EOF'
Usage:
  devnet.sh up N
  devnet.sh down
  devnet.sh run <i> <cmd...>
  devnet.sh chaos <i> <netem args...>
  devnet.sh chaos <i> clear
  devnet.sh partition <i>
  devnet.sh heal <i>
  devnet.sh status

Creates namespaces wb0 through wb<N-1> on bridge wbbr0 (ADR 0017).
Each namespace has address 10.77.0.<i+10>/24. `up` resets: it runs `down` first.
`heal <i>` clears any chaos on that peer; partition and chaos share the netem qdisc.
Chaos/partition netem is applied on both ends of the peer's veth, so values apply
in each direction: `chaos i delay 50ms` adds ~100 ms round trip with i; `loss 20%`
drops ~36% of request+reply exchanges. `heal` also flushes stale ARP entries so
the peer is reachable at once.

Self-check (needs sudo + socat): scripts/devnet-test.sh

No traffic across the bridge? Docker's br_netfilter may be filtering it:
  sudo sysctl net.bridge.bridge-nf-call-iptables=0
EOF
}

# Remove all namespaces and links owned by this script.
down() {
  local name i
  while read -r name _; do
    if [[ "$name" =~ ^wb[0-9]+$ ]]; then
      sudo ip netns del "$name" 2>/dev/null || true
    fi
  done < <(sudo ip netns list)
  for ((i = 0; i < 50; i++)); do
    sudo ip link del "wbv${i}h" 2>/dev/null || true
  done
  sudo ip link del wbbr0 2>/dev/null || true
}

# Recreate a bridge and between one and fifty connected namespaces.
up() {
  local count=$1 i ns host peer address
  if [[ ! "$count" =~ ^[0-9]+$ ]] || ((count < 1 || count > 50)); then
    echo "up requires N between 1 and 50" >&2
    return 2
  fi
  down
  sudo ip link add wbbr0 type bridge
  sudo ip link set wbbr0 up
  for ((i = 0; i < count; i++)); do
    ns="wb$i"
    host="wbv${i}h"
    peer="wbv${i}n"
    address=$((i + 10))
    sudo ip netns add "$ns"
    sudo ip link add "$host" type veth peer name "$peer"
    sudo ip link set "$host" master wbbr0
    sudo ip link set "$host" up
    sudo ip link set "$peer" netns "$ns"
    sudo ip -n "$ns" link set lo up
    sudo ip -n "$ns" link set "$peer" name eth0
    sudo ip -n "$ns" address add "10.77.0.$address/24" dev eth0
    sudo ip -n "$ns" link set eth0 up
    sudo ip -n "$ns" route add default dev eth0
  done
}

# Run a command in a namespace as the user who invoked this script.
run() {
  local index=$1 user=${SUDO_USER:-$(id -un)}
  shift
  if [[ ! "$index" =~ ^[0-9]+$ ]] || ((index > 49)) || (($# == 0)); then
    echo "run requires a namespace index from 0 to 49 and a command" >&2
    return 2
  fi
  sudo --preserve-env=WAYLAND_DISPLAY,XDG_RUNTIME_DIR,DISPLAY,HOME \
    ip netns exec "wb$index" \
    sudo --preserve-env=WAYLAND_DISPLAY,XDG_RUNTIME_DIR,DISPLAY,HOME \
      -u "$user" -- "$@"
}

# Apply or clear a netem qdisc on a namespace interface.
chaos() {
  local index=$1
  shift
  if [[ ! "$index" =~ ^[0-9]+$ ]] || ((index > 49)) || (($# == 0)); then
    echo "chaos requires a namespace index from 0 to 49 and netem arguments" >&2
    return 2
  fi
  if [[ "$1" == clear ]]; then
    sudo ip netns exec "wb$index" tc qdisc del dev eth0 root 2>/dev/null || true
    sudo tc qdisc del dev "wbv${index}h" root 2>/dev/null || true
    local ns
    while read -r ns _; do
      if [[ "$ns" =~ ^wb[0-9]+$ ]]; then
        sudo ip -n "$ns" neigh flush nud failed 2>/dev/null || true
        sudo ip -n "$ns" neigh flush nud incomplete 2>/dev/null || true
      fi
    done < <(sudo ip netns list)
  else
    sudo ip netns exec "wb$index" tc qdisc replace dev eth0 root netem "$@"
    sudo tc qdisc replace dev "wbv${index}h" root netem "$@"
  fi
}

# Print each managed namespace, address, and current qdisc.
status() {
  local i ns address qdisc
  for ((i = 0; i < 50; i++)); do
    ns="wb$i"
    if sudo ip netns exec "$ns" ip link show eth0 >/dev/null 2>&1; then
      address=$((i + 10))
      qdisc=$(sudo ip netns exec "$ns" tc qdisc show dev eth0)
      printf '%s 10.77.0.%s/24 %s\n' "$ns" "$address" "$qdisc"
    fi
  done
}

if (($# == 0)); then
  usage
  exit 2
fi

command=$1
shift
case "$command" in
  help) usage ;;
  up) (($# == 1)) || { usage >&2; exit 2; }; up "$1" ;;
  down) (($# == 0)) || { usage >&2; exit 2; }; down ;;
  run) (($# >= 2)) || { usage >&2; exit 2; }; run "$@" ;;
  chaos) (($# >= 2)) || { usage >&2; exit 2; }; chaos "$@" ;;
  partition) (($# == 1)) || { usage >&2; exit 2; }; chaos "$1" loss 100% ;;
  heal) (($# == 1)) || { usage >&2; exit 2; }; chaos "$1" clear ;;
  status) (($# == 0)) || { usage >&2; exit 2; }; status ;;
  *) usage >&2; exit 2 ;;
esac
