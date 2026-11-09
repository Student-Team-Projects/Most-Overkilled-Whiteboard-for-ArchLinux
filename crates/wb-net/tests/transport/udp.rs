//! `Udp`'s own tests: how `recv` waits.

use std::time::Duration;

use wb_net::{Transport, TransportError};

use crate::support::config::{ARRIVES, QUIET};
use crate::support::transport::udp_pair;

#[test]
fn recv_with_nothing_sent_times_out_with_none() -> Result<(), TransportError> {
    let (_, mut b) = udp_pair()?;
    assert!(b.transport.recv(QUIET)?.is_none());
    assert!(b.transport.recv(Duration::ZERO)?.is_none());
    Ok(())
}

/// A zero timeout switches the socket to non-blocking; the next real timeout must wait again.
#[test]
fn zero_timeout_takes_a_waiting_datagram_and_then_waits_again() -> Result<(), TransportError> {
    let (mut a, mut b) = udp_pair()?;
    a.transport.send(b.addr, b"first")?;
    assert_eq!(
        b.transport.recv(ARRIVES)?.map(|(_, d)| d),
        Some(b"first".to_vec())
    );
    a.transport.send(b.addr, b"second")?;
    std::thread::sleep(QUIET);
    assert_eq!(
        b.transport.recv(Duration::ZERO)?.map(|(_, d)| d),
        Some(b"second".to_vec())
    );

    a.transport.send(b.addr, b"third")?;
    assert_eq!(
        b.transport.recv(ARRIVES)?.map(|(_, d)| d),
        Some(b"third".to_vec())
    );
    Ok(())
}
