//! Connected transport pairs and helpers to drain them.

use std::net::SocketAddr;

use wb_net::{Chaos, Sim, SimNet, Transport, TransportError, Udp};

use super::config::{ARRIVES, QUIET, SEED};

pub type Datagram = (SocketAddr, Vec<u8>);

/// A transport and its own address.
pub struct Peer<T> {
    pub transport: T,
    pub addr: SocketAddr,
}

/// Two UDP sockets on loopback.
pub fn udp_pair() -> Result<(Peer<Udp>, Peer<Udp>), TransportError> {
    let peer = || {
        let transport = Udp::bind("127.0.0.1:0")?;
        let addr = transport.local_addr()?;
        Ok::<_, TransportError>(Peer { transport, addr })
    };
    Ok((peer()?, peer()?))
}

/// Two endpoints of a perfect simulated network, so datagrams arrive at once.
#[allow(clippy::unnecessary_wraps)] // the conformance suite takes it in place of `udp_pair`
pub fn sim_pair() -> Result<(Peer<Sim>, Peer<Sim>), TransportError> {
    let net = SimNet::new(SEED, Chaos::default());
    let peer = || {
        let transport = net.endpoint();
        let addr = transport.local_addr();
        Peer { transport, addr }
    };
    Ok((peer(), peer()))
}

/// Receives until `count` datagrams came or one wait ran out.
pub fn recv_up_to(
    transport: &mut impl Transport,
    count: usize,
) -> Result<Vec<Datagram>, TransportError> {
    let mut received = Vec::new();
    while received.len() < count {
        match transport.recv(ARRIVES)? {
            Some(datagram) => received.push(datagram),
            None => break,
        }
    }
    Ok(received)
}

/// Whether nothing more arrives within a short wait.
pub fn is_quiet(transport: &mut impl Transport) -> Result<bool, TransportError> {
    Ok(transport.recv(QUIET)?.is_none())
}

/// Everything that has arrived at `sim` by now.
pub fn drain(sim: &mut Sim) -> Result<Vec<Datagram>, TransportError> {
    let mut arrived = Vec::new();
    while let Some(datagram) = sim.recv(QUIET)? {
        arrived.push(datagram);
    }
    Ok(arrived)
}
