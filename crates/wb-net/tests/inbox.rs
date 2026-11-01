//! The inbox: our board and discovery pass, everything else is dropped and counted exactly.

use proptest::prelude::*;
use wb_net::config::{MAGIC, VERSION};
use wb_net::{Body, Inbox, Reason, Reject};

use crate::support::config::{MAX_DATAGRAMS, OTHER, OURS};
use crate::support::envelope::{board_datagram, datagram, discovery_datagram};

type Accepted = Result<Body<Vec<u8>, Vec<u8>>, Reject>;

fn counts(inbox: &Inbox) -> [u64; 4] {
    Reason::ALL.map(|reason| inbox.dropped(reason))
}

#[test]
fn accepts_our_board_and_any_discovery() {
    let mut inbox = Inbox::new(OURS);
    let ours: Accepted = inbox.accept(&board_datagram(OURS));
    let discovery: Accepted = inbox.accept(&discovery_datagram());
    assert!(ours.is_ok(), "{ours:?}");
    assert!(discovery.is_ok(), "{discovery:?}");
    assert_eq!(counts(&inbox), [0; 4]);
}

#[test]
fn drops_a_foreign_board_and_names_it() {
    let mut inbox = Inbox::new(OURS);
    let foreign: Accepted = inbox.accept(&board_datagram(OTHER));
    assert_eq!(foreign, Err(Reject::ForeignBoard { got: OTHER }));
}

/// A different number of drops per reason, so counts swapped between reasons show up.
#[test]
fn counts_each_reason_exactly() {
    let mut wrong_version = board_datagram(OURS);
    wrong_version[MAGIC.len()] = VERSION + 1;
    let mut truncated = board_datagram(OURS);
    truncated.pop();
    let feeds = [
        (b"nope".to_vec(), 1),
        (wrong_version, 2),
        (truncated, 3),
        (board_datagram(OTHER), 4),
        (board_datagram(OURS), 5),
    ];

    let mut inbox = Inbox::new(OURS);
    for (datagram, times) in feeds {
        for _ in 0..times {
            let _: Accepted = inbox.accept(&datagram);
        }
    }
    assert_eq!(counts(&inbox), [1, 2, 3, 4]);
}

proptest! {
    /// Each datagram is either accepted or adds exactly one to its own reason's count.
    #[test]
    fn every_datagram_is_accepted_or_counted_once(
        datagrams in prop::collection::vec(datagram(), 0..=MAX_DATAGRAMS),
    ) {
        let mut inbox = Inbox::new(OURS);
        for datagram in &datagrams {
            let mut expected = counts(&inbox);
            let result: Accepted = inbox.accept(datagram);
            if let Err(reject) = result {
                expected[reject.reason() as usize] += 1;
            }
            prop_assert_eq!(counts(&inbox), expected);
        }
    }
}
