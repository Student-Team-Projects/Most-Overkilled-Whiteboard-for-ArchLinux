//! Named test parameters.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::ops::Range;
use std::time::Duration;

/// How long a `recv` waits for a datagram that should come.
pub const ARRIVES: Duration = Duration::from_secs(1);
/// How long a `recv` waits before deciding that nothing more will come.
pub const QUIET: Duration = Duration::from_millis(50);

/// Seed of every simulated network unless a test compares seeds.
pub const SEED: u64 = 7;
/// Datagrams sent in the tests that count losses and copies.
pub const MANY: usize = 1000;

/// Loss chance in the chaos tests; of `MANY` datagrams, `LOST` are lost.
pub const LOSS: f64 = 0.25;
/// How many of `MANY` datagrams `LOSS` loses with `SEED`: about a quarter.
pub const LOST: Range<usize> = 200..300;
/// Duplication chance in the chaos tests.
pub const DUPLICATE: f64 = 0.5;
/// How many of the about 750 datagrams that arrive `DUPLICATE` doubles with `SEED`: about half.
pub const DOUBLED: Range<usize> = 325..425;
/// Milliseconds every simulated datagram takes in the latency tests.
pub const LATENCY: u64 = 10;
/// Most extra milliseconds in the latency tests.
pub const JITTER: u64 = 5;

/// An address no simulated endpoint has (TEST-NET-1, reserved for documentation).
pub const NOBODY: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 1);
