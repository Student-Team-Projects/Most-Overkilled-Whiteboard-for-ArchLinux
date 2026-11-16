//! Throttled log lines for dropped datagrams: one per reason at most every 10 s.

use wb_core::Time;

use crate::config::REJECT_LOG_EVERY_MS;
use crate::envelope::{Reason, Reject};

/// Decides which dropped datagrams get a log line, so a flood of garbage cannot flood the log.
///
/// Each reason is throttled on its own. The first drop of a reason is logged at once; later
/// drops of that reason within `REJECT_LOG_EVERY_MS` of its last line are skipped, and the next
/// line after that says how many were skipped. Skipped drops are still counted exactly by
/// `Inbox`; only the lines are thinned out. A clock that goes backwards counts as no time passed.
#[derive(Debug, Default)]
pub struct RejectLog {
    reasons: [Throttle; Reason::ALL.len()],
}

/// The state of one reason: when it last got a line, and how many drops it skipped since.
#[derive(Debug, Default, Clone, Copy)]
struct Throttle {
    last: Option<Time>,
    skipped: u64,
}

impl Throttle {
    /// Never logged, nothing skipped.
    const NEW: Self = Self {
        last: None,
        skipped: 0,
    };
}

impl RejectLog {
    /// A log where no reason has been logged yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reasons: [Throttle::NEW; Reason::ALL.len()],
        }
    }

    /// The line to log for `reject`, dropped at `now`, or `None` if it is skipped.
    ///
    /// The line is `dropped datagram: <reject>`, plus ` (+N more)` when N drops of the same
    /// reason were skipped since its last line. An unknown version names both versions.
    pub fn record(&mut self, reject: &Reject, now: Time) -> Option<String> {
        let throttle = &mut self.reasons[reject.reason() as usize];
        let quiet = throttle
            .last
            .is_some_and(|last| now.0.saturating_sub(last.0) < REJECT_LOG_EVERY_MS);
        if quiet {
            throttle.skipped = throttle.skipped.saturating_add(1);
            return None;
        }
        let line = match throttle.skipped {
            0 => format!("dropped datagram: {reject}"),
            skipped => format!("dropped datagram: {reject} (+{skipped} more)"),
        };
        *throttle = Throttle {
            last: Some(now),
            skipped: 0,
        };
        Some(line)
    }
}
