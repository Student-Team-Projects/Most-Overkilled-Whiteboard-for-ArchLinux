//! Conformance suite for `Transport`.

use wb_net::{MAX_DATAGRAM, Transport, TransportError};

use crate::support::transport::{Peer, is_quiet, recv_up_to, sim_pair, udp_pair};

/// Generates a module of tests that run every law on a pair of connected transports from `$pair`.
macro_rules! transport_conformance {
    ($impl:ident, $pair:expr) => {
        mod $impl {
            use super::*;

            #[test]
            fn whole_or_nothing() -> Result<(), TransportError> {
                let (a, b) = $pair()?;
                super::whole_or_nothing(a, b)
            }

            #[test]
            fn oversize_is_err() -> Result<(), TransportError> {
                let (a, b) = $pair()?;
                super::oversize_is_err(a, b)
            }

            #[test]
            fn max_datagram_is_1200() -> Result<(), TransportError> {
                let (a, _) = $pair()?;
                assert_eq!(a.transport.max_datagram(), 1200);
                assert_eq!(MAX_DATAGRAM, 1200);
                Ok(())
            }
        }
    };
}

transport_conformance!(udp, udp_pair);
transport_conformance!(sim, sim_pair);

/// Datagrams from empty to the maximum arrive with their sender and their exact bytes, one each.
fn whole_or_nothing(
    mut a: Peer<impl Transport>,
    mut b: Peer<impl Transport>,
) -> Result<(), TransportError> {
    let mut sent: Vec<Vec<u8>> = [0, 1, MAX_DATAGRAM / 2, MAX_DATAGRAM]
        .into_iter()
        .zip(0u8..)
        .map(|(len, fill)| vec![fill; len])
        .collect();
    for datagram in &sent {
        a.transport.send(b.addr, datagram)?;
    }
    let mut received = Vec::new();
    for (from, datagram) in recv_up_to(&mut b.transport, sent.len())? {
        assert_eq!(from, a.addr);
        received.push(datagram);
    }
    assert!(
        is_quiet(&mut b.transport)?,
        "nothing arrives twice or split"
    );
    sent.sort();
    received.sort();
    assert_eq!(received, sent);
    Ok(())
}

fn oversize_is_err(
    mut a: Peer<impl Transport>,
    mut b: Peer<impl Transport>,
) -> Result<(), TransportError> {
    let too_long = vec![0; MAX_DATAGRAM + 1];
    let result = a.transport.send(b.addr, &too_long);
    assert!(
        matches!(result, Err(TransportError::Oversize { len, max }) if len == MAX_DATAGRAM + 1 && max == MAX_DATAGRAM),
        "{result:?}"
    );
    assert!(is_quiet(&mut b.transport)?, "nothing of it is sent");
    Ok(())
}
