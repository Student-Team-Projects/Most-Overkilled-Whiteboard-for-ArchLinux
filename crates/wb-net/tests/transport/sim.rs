//! `Sim`'s own tests: the clock, seeded chaos and partitions.

use std::time::Duration;

use wb_core::Time;
use wb_net::{Chaos, Sim, SimNet, Transport, TransportError};

use crate::support::config::{DOUBLED, DUPLICATE, JITTER, LATENCY, LOSS, LOST, MANY, NOBODY, SEED};
use crate::support::transport::drain;

fn net(seed: u64, chaos: Chaos) -> (SimNet, Sim, Sim) {
    let net = SimNet::new(seed, chaos);
    let (a, b) = (net.endpoint(), net.endpoint());
    (net, a, b)
}

/// Sends `MANY` numbered datagrams from `a` to `b` at time 0, then moves the clock one
/// millisecond at a time until every copy is due; returns each arrival as (time, number).
fn arrivals(seed: u64, chaos: Chaos) -> Result<Vec<(u64, usize)>, TransportError> {
    let (net, mut a, mut b) = net(seed, chaos);
    for i in 0..MANY {
        a.send(b.local_addr(), &i.to_le_bytes())?;
    }
    let mut arrivals = Vec::new();
    for ms in 0..=chaos.latency + chaos.jitter {
        net.advance_to(Time(ms));
        for (from, datagram) in drain(&mut b)? {
            assert_eq!(from, a.local_addr());
            let bytes = datagram
                .try_into()
                .map_err(|_| std::io::Error::other("not a number"))?;
            arrivals.push((ms, usize::from_le_bytes(bytes)));
        }
    }
    Ok(arrivals)
}

/// How many copies of each of the `MANY` datagrams arrived.
fn copies(chaos: Chaos) -> Result<Vec<usize>, TransportError> {
    let mut copies = vec![0; MANY];
    for (_, i) in arrivals(SEED, chaos)? {
        copies[i] += 1;
    }
    Ok(copies)
}

#[test]
fn endpoints_get_distinct_addresses() {
    let (_, a, b) = net(SEED, Chaos::default());
    assert_ne!(a.local_addr(), b.local_addr());
}

#[test]
fn a_perfect_network_delivers_each_datagram_once_in_order_at_once() -> Result<(), TransportError> {
    let expected: Vec<_> = (0..MANY).map(|i| (0, i)).collect();
    assert_eq!(arrivals(SEED, Chaos::default())?, expected);
    Ok(())
}

#[test]
fn full_loss_delivers_nothing() -> Result<(), TransportError> {
    let lossy = Chaos {
        loss: 1.0,
        ..Chaos::default()
    };
    assert_eq!(copies(lossy)?, vec![0; MANY]);
    Ok(())
}

#[test]
fn full_duplication_delivers_each_datagram_twice() -> Result<(), TransportError> {
    let doubling = Chaos {
        duplicate: 1.0,
        ..Chaos::default()
    };
    assert_eq!(copies(doubling)?, vec![2; MANY]);
    Ok(())
}

#[test]
fn partial_loss_and_duplication_happen_at_about_their_rates() -> Result<(), TransportError> {
    let copies = copies(Chaos {
        loss: LOSS,
        duplicate: DUPLICATE,
        ..Chaos::default()
    })?;
    let lost = copies.iter().filter(|&&n| n == 0).count();
    let doubled = copies.iter().filter(|&&n| n == 2).count();
    assert!(LOST.contains(&lost), "{lost} of {MANY} lost");
    assert!(DOUBLED.contains(&doubled), "{doubled} of {MANY} doubled");
    assert!(copies.iter().all(|&n| n <= 2));
    Ok(())
}

/// Both copies of every datagram take the latency, and jitter spreads and reorders them.
#[test]
fn every_copy_arrives_after_latency_within_jitter() -> Result<(), TransportError> {
    let arrivals = arrivals(
        SEED,
        Chaos {
            duplicate: 1.0,
            latency: LATENCY,
            jitter: JITTER,
            ..Chaos::default()
        },
    )?;
    assert_eq!(arrivals.len(), 2 * MANY);
    let window = LATENCY..=LATENCY + JITTER;
    assert!(arrivals.iter().all(|(ms, _)| window.contains(ms)));
    for ms in window {
        assert!(arrivals.iter().any(|&(at, _)| at == ms), "none at {ms}");
    }
    let in_send_order = arrivals.windows(2).all(|pair| pair[0].1 <= pair[1].1);
    assert!(!in_send_order, "jitter reorders");
    Ok(())
}

#[test]
fn the_clock_never_moves_back() -> Result<(), TransportError> {
    let (net, mut a, mut b) = net(
        SEED,
        Chaos {
            latency: LATENCY,
            ..Chaos::default()
        },
    );
    a.send(b.local_addr(), b"x")?;
    net.advance_to(Time(LATENCY));
    net.advance_to(Time(0));
    assert!(b.recv(Duration::ZERO)?.is_some());
    Ok(())
}

#[test]
fn the_same_seed_gives_the_same_deliveries() -> Result<(), TransportError> {
    let chaos = Chaos {
        loss: LOSS,
        duplicate: DUPLICATE,
        latency: LATENCY,
        jitter: JITTER,
    };
    assert_eq!(arrivals(SEED, chaos)?, arrivals(SEED, chaos)?);
    assert_ne!(arrivals(SEED, chaos)?, arrivals(SEED + 1, chaos)?);
    Ok(())
}

#[test]
fn a_partition_drops_datagrams_across_it_until_healed() -> Result<(), TransportError> {
    let net = SimNet::new(SEED, Chaos::default());
    let [mut a, mut b, mut c] = [net.endpoint(), net.endpoint(), net.endpoint()];
    let [a_addr, b_addr, c_addr] = [a.local_addr(), b.local_addr(), c.local_addr()];
    net.partition(&[a_addr]);
    a.send(b_addr, b"a to b")?;
    b.send(a_addr, b"b to a")?;
    c.send(b_addr, b"c to b")?;
    assert!(drain(&mut a)?.is_empty());
    assert_eq!(drain(&mut b)?, [(c_addr, b"c to b".to_vec())]);

    net.heal();
    a.send(b_addr, b"a to b")?;
    assert_eq!(drain(&mut b)?, [(a_addr, b"a to b".to_vec())]);
    assert!(drain(&mut c)?.is_empty());
    Ok(())
}

/// Only the moment of sending counts: what was in flight still arrives, and what was sent across
/// stays lost after a heal.
#[test]
fn a_partition_applies_when_a_datagram_is_sent() -> Result<(), TransportError> {
    let delayed = Chaos {
        latency: LATENCY,
        ..Chaos::default()
    };
    let (net, mut a, mut b) = net(SEED, delayed);
    a.send(b.local_addr(), b"before")?;
    net.partition(&[a.local_addr()]);
    a.send(b.local_addr(), b"during")?;
    net.heal();
    net.advance_to(Time(LATENCY));
    assert_eq!(drain(&mut b)?, [(a.local_addr(), b"before".to_vec())]);
    Ok(())
}

#[test]
fn a_datagram_to_no_endpoint_is_lost() -> Result<(), TransportError> {
    let (_, mut a, mut b) = net(SEED, Chaos::default());
    a.send(NOBODY, b"lost")?;
    assert!(drain(&mut a)?.is_empty() && drain(&mut b)?.is_empty());
    Ok(())
}
