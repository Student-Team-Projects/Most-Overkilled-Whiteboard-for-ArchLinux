//! Helpers for inbox tests.

use wb_net::{Inbox, Reason};

/// Returns the dropped datagram count for every rejection reason.
pub fn counts(inbox: &Inbox) -> [u64; 4] {
    Reason::ALL.map(|reason| inbox.dropped(reason))
}
