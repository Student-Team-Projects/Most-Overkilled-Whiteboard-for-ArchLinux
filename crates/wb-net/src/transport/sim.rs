//! The simulated transport: an in-memory network with seeded chaos.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::net::{Ipv4Addr, SocketAddr};
use std::rc::Rc;
use std::time::Duration;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use wb_core::Time;

use super::{Transport, TransportError, check_size};
use crate::config::{MAX_DATAGRAM, SIM_FIRST_ADDR, SIM_PORT};

/// How the simulated network mistreats datagrams. The default is a perfect network.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Chaos {
    /// Chance that a datagram is lost, from 0 to 1.
    pub loss: f64,
    /// Chance that a datagram that is not lost arrives twice, from 0 to 1.
    pub duplicate: f64,
    /// Milliseconds every datagram takes at least.
    pub latency: u64,
    /// Most extra milliseconds a datagram takes, drawn uniformly per copy; this reorders datagrams.
    pub jitter: u64,
}

/// An in-memory network for one deterministic simulation; clones are handles to the same network.
///
/// Its clock moves only by [`advance_to`](Self::advance_to), so a [`Sim`] endpoint's `recv`
/// returns only datagrams that have already arrived and never waits. The same seed, chaos and
/// calls give the same deliveries.
#[derive(Debug, Clone)]
pub struct SimNet(Rc<RefCell<Network>>);

/// One endpoint of a [`SimNet`].
#[derive(Debug)]
pub struct Sim {
    net: SimNet,
    addr: SocketAddr,
}

/// Datagrams on their way to one endpoint, with their senders, by arrival time and copy number.
type Inbox = BTreeMap<(Time, u64), (SocketAddr, Vec<u8>)>;

#[derive(Debug)]
struct Network {
    rng: ChaCha8Rng,
    chaos: Chaos,
    now: Time,
    /// Datagram copies queued so far; numbers each copy to break ties between equal arrivals.
    queued: u64,
    inboxes: BTreeMap<SocketAddr, Inbox>,
    /// One side of the partition; empty when the network is whole.
    cut_off: BTreeSet<SocketAddr>,
}

impl SimNet {
    /// A network at time 0 with no endpoints, whose chaos is drawn from `seed`.
    #[must_use]
    pub fn new(seed: u64, chaos: Chaos) -> Self {
        Self(Rc::new(RefCell::new(Network {
            rng: ChaCha8Rng::seed_from_u64(seed),
            chaos,
            now: Time(0),
            queued: 0,
            inboxes: BTreeMap::new(),
            cut_off: BTreeSet::new(),
        })))
    }

    /// Adds an endpoint at a fresh address.
    #[must_use]
    pub fn endpoint(&self) -> Sim {
        let mut net = self.0.borrow_mut();
        let index = u32::try_from(net.inboxes.len()).unwrap_or(u32::MAX);
        let ip = Ipv4Addr::from_bits(SIM_FIRST_ADDR.to_bits().saturating_add(index));
        let addr = SocketAddr::new(ip.into(), SIM_PORT);
        net.inboxes.insert(addr, BTreeMap::new());
        Sim {
            net: self.clone(),
            addr,
        }
    }

    /// Moves the clock to `now`; it never moves back.
    pub fn advance_to(&self, now: Time) {
        let mut net = self.0.borrow_mut();
        net.now = net.now.max(now);
    }

    /// Splits the network in two, `side` and everyone else; datagrams sent across are lost until
    /// [`heal`](Self::heal). Replaces any earlier partition.
    pub fn partition(&self, side: &[SocketAddr]) {
        self.0.borrow_mut().cut_off = side.iter().copied().collect();
    }

    /// Joins the network back together.
    pub fn heal(&self) {
        self.0.borrow_mut().cut_off.clear();
    }
}

impl Sim {
    /// This endpoint's address on its network.
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Transport for Sim {
    /// A datagram to an address with no endpoint is lost.
    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), TransportError> {
        check_size(datagram)?;
        self.net.0.borrow_mut().send(self.addr, to, datagram);
        Ok(())
    }

    /// Never waits: time passes only by [`SimNet::advance_to`], so `timeout` is ignored.
    fn recv(
        &mut self,
        _timeout: Duration,
    ) -> Result<Option<(SocketAddr, Vec<u8>)>, TransportError> {
        Ok(self.net.0.borrow_mut().take_arrived(self.addr))
    }

    fn max_datagram(&self) -> usize {
        MAX_DATAGRAM
    }
}

impl Network {
    fn send(&mut self, from: SocketAddr, to: SocketAddr, datagram: &[u8]) {
        let across = self.cut_off.contains(&from) != self.cut_off.contains(&to);
        if across || !self.inboxes.contains_key(&to) || self.chance(self.chaos.loss) {
            return;
        }
        let copies = if self.chance(self.chaos.duplicate) {
            2
        } else {
            1
        };
        for _ in 0..copies {
            let jitter = self.rng.random_range(0..=self.chaos.jitter);
            let arrival = Time(
                self.now
                    .0
                    .saturating_add(self.chaos.latency)
                    .saturating_add(jitter),
            );
            self.queued += 1;
            let key = (arrival, self.queued);
            if let Some(inbox) = self.inboxes.get_mut(&to) {
                inbox.insert(key, (from, datagram.to_vec()));
            }
        }
    }

    fn take_arrived(&mut self, addr: SocketAddr) -> Option<(SocketAddr, Vec<u8>)> {
        let inbox = self.inboxes.get_mut(&addr)?;
        let next = inbox.first_entry()?;
        (next.key().0 <= self.now).then(|| next.remove())
    }

    /// True with probability `p`; a `p` outside 0..=1 acts as 0 or 1.
    fn chance(&mut self, p: f64) -> bool {
        self.rng.random::<f64>() < p
    }
}
