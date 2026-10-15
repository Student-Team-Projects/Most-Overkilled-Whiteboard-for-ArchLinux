#!/usr/bin/env bash
# Live self-check for devnet.sh (T24, ADR 0017). Needs sudo (asks once) and socat for the broadcast check.
# Usage: scripts/devnet-test.sh   — exits 0 only if every check passes; always tears the devnet down.
set -uo pipefail

D="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/devnet.sh"
BCAST="$(mktemp)"
pass=0
fail=0

# Leave nothing behind, even when a check fails or the run is interrupted.
cleanup() {
  "$D" down >/dev/null 2>&1
  rm -f "$BCAST"
}
trap cleanup EXIT

# check <name> <shell condition>: run the condition, print PASS/FAIL, count it.
check() {
  if eval "$2" >/dev/null 2>&1; then
    echo "PASS  $1"
    ((pass++))
  else
    echo "FAIL  $1"
    ((fail++))
  fi
}

command -v socat >/dev/null || { echo "socat is required: sudo pacman -S socat" >&2; exit 2; }
sudo -v || exit 2

check "syntax (bash -n)"               "bash -n '$D'"
check "up 3"                           "'$D' up 3"
check "up is idempotent (up 3 again)"  "'$D' up 3"
check "wb0 pings wb2"                  "sudo ip netns exec wb0 ping -c1 -W1 10.77.0.12"
check "UDP subnet broadcast wb0 -> wb2" \
  "( '$D' run 2 timeout 3 socat -u UDP-RECV:47000 STDOUT > '$BCAST' & ); sleep 0.5; \
   echo hello | '$D' run 0 socat -u STDIN UDP-DATAGRAM:10.77.0.255:47000,broadcast; sleep 1; grep -q hello '$BCAST'"
check "UDP limited broadcast wb0 -> wb2" \
  "( '$D' run 2 timeout 3 socat -u UDP-RECV:47001 STDOUT > '$BCAST' & ); sleep 0.5; \
   echo hello | '$D' run 0 socat -u STDIN UDP-DATAGRAM:255.255.255.255:47001,broadcast; sleep 1; grep -q hello '$BCAST'"
check "UDP multicast destination is routable" \
  "echo hello | '$D' run 0 socat -u STDIN UDP-DATAGRAM:239.1.2.3:47002"
check "run executes as you, not root"  "[ \"\$('$D' run 1 id -un)\" = \"\$(id -un)\" ]"
check "partition 1 breaks ping"        "'$D' partition 1 && ! sudo ip netns exec wb0 ping -c1 -W1 10.77.0.11"
check "partition 1 blocks incoming UDP" \
  "rm -f '$BCAST'; ( '$D' run 1 timeout 3 socat -u UDP-RECV:47003 STDOUT > '$BCAST' & ); sleep 0.5; \
   echo hello | '$D' run 0 socat -u STDIN UDP-DATAGRAM:10.77.0.11:47003; sleep 1; ! grep -q hello '$BCAST'"
check "heal 1 restores ping"           "'$D' heal 1 && sudo ip netns exec wb0 ping -c1 -W1 10.77.0.11"
check "chaos 2 loss 20% delay 50ms"    "'$D' chaos 2 loss 20% delay 50ms && '$D' status | grep -q 'wb2.*netem'"
check "down"                           "'$D' down"
check "down is idempotent"             "'$D' down"
check "nothing left: no wb* netns"     "! sudo ip netns list | grep -q '^wb'"
check "nothing left: no wb* links"     "! ip -o link | grep -qE ': wb(br0|v[0-9]+h)'"

echo "---- $pass passed, $fail failed"
((fail == 0))
