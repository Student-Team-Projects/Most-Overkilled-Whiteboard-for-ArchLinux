//! Protocol constants and simulator settings.

use std::net::Ipv4Addr;

/// The largest datagram every transport sends, in bytes: small enough to cross the internet
/// without IP fragmentation.
pub const MAX_DATAGRAM: usize = 1200;

/// Simulated endpoints get consecutive addresses from here up.
pub const SIM_FIRST_ADDR: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
/// Port of every simulated endpoint.
pub const SIM_PORT: u16 = 47_000;
